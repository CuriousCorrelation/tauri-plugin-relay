use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::SystemTime;

use curl::easy::{Easy, List};
use lazy_static::lazy_static;
use openssl::{pkcs12::Pkcs12, ssl::SslContextBuilder, x509::X509};
use openssl_sys::SSL_CTX;
use tokio_util::sync::CancellationToken;

use crate::{
    error::{RelayError, RelayResult},
    interop::{
        BodyDef, ClientCertDef, FormDataValue, KeyValuePair, RequestWithMetadata,
        ResponseWithMetadata,
    },
    util::get_status_text,
};

lazy_static! {
    static ref CANCELLED_REQUESTS: Arc<Mutex<HashMap<usize, Arc<AtomicBool>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub(crate) fn run(req: RequestWithMetadata) -> RelayResult<ResponseWithMetadata> {
    let req_id = req.req_id;
    let cancelled = Arc::new(AtomicBool::new(false));

    {
        let mut cancelled_requests = CANCELLED_REQUESTS.lock().unwrap();
        cancelled_requests.insert(req_id, Arc::clone(&cancelled));
    }

    let cancel_token = CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();
    let cancelled_clone = Arc::clone(&cancelled);

    let handle = std::thread::spawn(move || {
        let result = run_request_task(req, cancel_token);

        if cancel_token_clone.is_cancelled() {
            cancelled_clone.store(true, Ordering::SeqCst);
        }

        result
    });

    let result = match handle.join() {
        Ok(result) => {
            if cancelled.load(Ordering::SeqCst) {
                Err(RelayError::RequestCancelled)
            } else {
                result
            }
        }
        Err(_) => Err(RelayError::RequestRunError(
            "Request thread panicked".to_string(),
        )),
    };

    {
        let mut cancelled_requests = CANCELLED_REQUESTS.lock().unwrap();
        cancelled_requests.remove(&req_id);
    }

    result
}

pub(crate) fn cancel(req_id: usize) {
    if let Some(cancelled) = CANCELLED_REQUESTS.lock().unwrap().get(&req_id) {
        cancelled.store(true, Ordering::SeqCst);
    }
}

fn run_request_task(
    req: RequestWithMetadata,
    cancel_token: CancellationToken,
) -> Result<ResponseWithMetadata, RelayError> {
    let mut curl_handle = Easy::new();

    curl_handle
        .progress(true)
        .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

    curl_handle
        .custom_request(&req.method)
        .map_err(|_| RelayError::InvalidMethod)?;

    curl_handle
        .url(&req.endpoint)
        .map_err(|_| RelayError::InvalidUrl)?;

    let headers = get_headers_list(&req)?;
    curl_handle
        .http_headers(headers)
        .map_err(|_| RelayError::InvalidHeaders)?;

    apply_body_to_curl_handle(&mut curl_handle, &req)?;
    apply_ssl_config_to_curl_handle(&mut curl_handle, &req)?;
    apply_client_cert_to_curl_handle(&mut curl_handle, &req)?;
    apply_proxy_config_to_curl_handle(&mut curl_handle, &req)?;

    let mut response_body = Vec::new();
    let mut response_headers = Vec::new();
    let (start_time_ms, end_time_ms) = {
        let mut transfer = curl_handle.transfer();

        transfer
            .ssl_ctx_function(|ssl_ctx_ptr| {
                let cert_list = get_x509_certs_from_root_cert_bundle_safe(&req).unwrap_or_default();

                if !cert_list.is_empty() {
                    let mut ssl_ctx_builder =
                        unsafe { SslContextBuilder::from_ptr(ssl_ctx_ptr as *mut SSL_CTX) };

                    let cert_store = ssl_ctx_builder.cert_store_mut();

                    for cert in cert_list.iter() {
                        let _ = cert_store.add_cert(cert.clone());
                    }

                    // SAFETY: We need to prevent Rust from dropping the `SslContextBuilder` because
                    // the underlying `SSL_CTX` pointer is owned and managed by curl, not us.
                    // From curl docs: "libcurl does not guarantee the lifetime of the passed in
                    // object once this callback function has returned"
                    // and `SslContextBuilder` is just a safe wrapper around curl's `SSL_CTX` from
                    // `openssl_sys::SSL_CTX`.
                    // If dropped, Rust would try to free the `SSL_CTX` which curl still needs.
                    //
                    // This intentional "leak" is safe because:
                    // - We're only leaking the thin Rust wrapper
                    // - Curl manages the actual `SSL_CTX` memory
                    // - Curl will free the `SSL_CTX` during connection cleanup
                    //
                    // See: https://curl.se/libcurl/c/CURLOPT_SSL_CTX_FUNCTION.html
                    std::mem::forget(ssl_ctx_builder);
                }

                Ok(())
            })
            .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

        transfer
            .progress_function(|_dltotal, _dlnow, _ultotal, _ulnow| {
                let cancelled = CANCELLED_REQUESTS
                    .lock()
                    .unwrap()
                    .get(&req.req_id)
                    .map(|flag| flag.load(Ordering::SeqCst))
                    .unwrap_or(false);

                if cancelled {
                    cancel_token.cancel();
                }
                !cancelled
            })
            .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

        transfer
            .header_function(|header| {
                let header = String::from_utf8_lossy(header).into_owned();
                if let Some((key, value)) = header.split_once(':') {
                    response_headers.push(KeyValuePair {
                        key: key.trim().to_string(),
                        value: value.trim().to_string(),
                    });
                }
                true
            })
            .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

        transfer
            .write_function(|data| {
                response_body.extend_from_slice(data);
                Ok(data.len())
            })
            .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

        let start_time_ms = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        transfer
            .perform()
            .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

        let end_time_ms = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        (start_time_ms, end_time_ms)
    };

    let response_status = curl_handle
        .response_code()
        .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?
        as u16;

    Ok(ResponseWithMetadata {
        status: response_status,
        status_text: get_status_text(response_status).to_string(),
        headers: response_headers,
        data: response_body,
        time_start_ms: start_time_ms,
        time_end_ms: end_time_ms,
    })
}

fn get_headers_list(req: &RequestWithMetadata) -> Result<List, RelayError> {
    let mut result = List::new();

    for KeyValuePair { key, value } in &req.headers {
        result
            .append(&format!("{}: {}", key, value))
            .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;
    }

    Ok(result)
}

fn apply_body_to_curl_handle(
    curl_handle: &mut Easy,
    req: &RequestWithMetadata,
) -> Result<(), RelayError> {
    match &req.body {
        Some(BodyDef::Text(text)) => {
            curl_handle
                .post_fields_copy(text.as_bytes())
                .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;
        }
        Some(BodyDef::FormData(entries)) => {
            let mut form = curl::easy::Form::new();

            for entry in entries {
                let mut part = form.part(&entry.key);

                match &entry.value {
                    FormDataValue::Text(data) => {
                        part.contents(data.as_bytes());
                    }
                    FormDataValue::File {
                        filename,
                        data,
                        mime,
                    } => {
                        part.buffer(filename, data.clone()).content_type(mime);
                    }
                };

                part.add()
                    .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;
            }

            curl_handle
                .httppost(form)
                .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;
        }
        Some(BodyDef::URLEncoded(entries)) => {
            let data = entries
                .iter()
                .map(|KeyValuePair { key, value }| {
                    format!(
                        "{}={}",
                        &url_escape::encode_www_form_urlencoded(key),
                        url_escape::encode_www_form_urlencoded(value)
                    )
                })
                .collect::<Vec<String>>()
                .join("&");

            curl_handle
                .post_fields_copy(data.as_bytes())
                .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;
        }
        None => {}
    };

    Ok(())
}

fn apply_ssl_config_to_curl_handle(
    curl_handle: &mut Easy,
    req: &RequestWithMetadata,
) -> Result<(), RelayError> {
    curl_handle
        .ssl_verify_peer(req.validate_certs)
        .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

    curl_handle
        .ssl_verify_host(req.validate_certs)
        .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

    Ok(())
}

fn apply_client_cert_to_curl_handle(
    handle: &mut Easy,
    req: &RequestWithMetadata,
) -> Result<(), RelayError> {
    match &req.client_cert {
        Some(ClientCertDef::PEMCert {
            certificate_pem,
            key_pem,
        }) => {
            handle
                .ssl_cert_type("PEM")
                .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

            handle
                .ssl_cert_blob(certificate_pem)
                .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

            handle
                .ssl_key_type("PEM")
                .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

            handle
                .ssl_key_blob(key_pem)
                .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;
        }
        Some(ClientCertDef::PFXCert {
            certificate_pfx,
            password,
        }) => {
            let pkcs12 = Pkcs12::from_der(&certificate_pfx).map_err(|err| {
                RelayError::RequestRunError(format!(
                    "Failed to parse PFX certificate from DER: {}",
                    err
                ))
            })?;

            let parsed = pkcs12.parse2(password).map_err(|err| {
                RelayError::RequestRunError(format!(
                    "Failed to parse PFX certificate with provided password: {}",
                    err
                ))
            })?;

            if let (Some(cert), Some(key)) = (parsed.cert, parsed.pkey) {
                let certificate_pem = cert.to_pem().map_err(|err| {
                    RelayError::RequestRunError(format!(
                        "Failed to convert PFX certificate to PEM format: {}",
                        err
                    ))
                })?;

                let key_pem = key.private_key_to_pem_pkcs8().map_err(|err| {
                    RelayError::RequestRunError(format!(
                        "Failed to convert PFX private key to PEM format: {}",
                        err
                    ))
                })?;

                handle
                    .ssl_cert_type("PEM")
                    .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

                handle
                    .ssl_cert_blob(&certificate_pem)
                    .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

                handle
                    .ssl_key_type("PEM")
                    .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

                handle
                    .ssl_key_blob(&key_pem)
                    .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;
            } else {
                return Err(RelayError::RequestRunError(
                    "PFX certificate parsing succeeded, but either cert or private key is missing"
                        .to_string(),
                ));
            }
        }
        None => {}
    };

    Ok(())
}

fn get_x509_certs_from_root_cert_bundle_safe(
    req: &RequestWithMetadata,
) -> Result<Vec<X509>, openssl::error::ErrorStack> {
    let mut certs = Vec::new();

    for pem_bundle in &req.root_cert_bundle_files {
        if let Ok(mut bundle_certs) = X509::stack_from_pem(pem_bundle) {
            certs.append(&mut bundle_certs);
        }
    }

    Ok(certs)
}

fn apply_proxy_config_to_curl_handle(
    handle: &mut Easy,
    req: &RequestWithMetadata,
) -> Result<(), RelayError> {
    if let Some(proxy_config) = &req.proxy {
        handle
            .proxy_auth(curl::easy::Auth::new().auto(true))
            .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;

        handle
            .proxy(&proxy_config.url)
            .map_err(|err| RelayError::RequestRunError(err.description().to_string()))?;
    }

    Ok(())
}
