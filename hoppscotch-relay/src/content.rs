use std::collections::HashMap;

use curl::easy::Easy;

use crate::{
    error::{InterceptorError, RequestResult},
    header::HeadersBuilder,
    interop::ContentType,
};

pub(crate) struct ContentHandler<'a> {
    handle: &'a mut Easy,
}

impl<'a> ContentHandler<'a> {
    pub(crate) fn new(handle: &'a mut Easy) -> Self {
        Self { handle }
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub(crate) fn set_content(&mut self, content: &ContentType) -> RequestResult<()> {
        match content {
            ContentType::Text { content } => {
                tracing::trace!(content_length = content.len(), "Setting text content");
                self.set_text_content(content)
            }
            ContentType::Json { content } => {
                tracing::trace!("Setting JSON content");
                self.set_json_content(content)
            }
            ContentType::Form { content } => {
                tracing::trace!(field_count = content.len(), "Setting form content");
                self.set_form_content(content)
            }
            ContentType::UrlEncoded { content } => {
                tracing::trace!(field_count = content.len(), "Setting URL-encoded content");
                self.set_urlencoded_content(content)
            }
        }
    }

    fn set_text_content(&mut self, content: &str) -> RequestResult<()> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), vec!["text/plain".to_string()]);
        HeadersBuilder::new(self.handle).add_headers(Some(&headers))?;

        self.handle
            .post_fields_copy(content.as_bytes())
            .map_err(|e| InterceptorError::Network {
                message: "Failed to set text content".into(),
                cause: Some(e.to_string()),
            })
    }

    fn set_json_content(&mut self, content: &serde_json::Value) -> RequestResult<()> {
        let mut headers = HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            vec!["application/json".to_string()],
        );
        HeadersBuilder::new(self.handle).add_headers(Some(&headers))?;

        let json_str = serde_json::to_string(content).map_err(|e| InterceptorError::Parse {
            message: "Failed to serialize JSON".into(),
            cause: Some(e.to_string()),
        })?;

        self.set_text_content(&json_str)
    }

    fn set_form_content(&mut self, content: &HashMap<String, Vec<u8>>) -> RequestResult<()> {
        let mut form = curl::easy::Form::new();
        for (key, value) in content {
            match form.part(key).contents(value).add() {
                Ok(_) => tracing::trace!(key = %key, "Added form field"),
                Err(e) => {
                    tracing::error!(error = %e, key = %key, "Failed to add form field");
                    return Err(InterceptorError::Network {
                        message: "Failed to add form field".into(),
                        cause: Some(e.to_string()),
                    });
                }
            }
        }

        self.handle.httppost(form).map_err(|e| {
            tracing::error!(error = %e, "Failed to set form data");
            InterceptorError::Network {
                message: "Failed to set form data".into(),
                cause: Some(e.to_string()),
            }
        })
    }

    fn set_urlencoded_content(&mut self, content: &HashMap<String, String>) -> RequestResult<()> {
        let mut headers = HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            vec!["application/x-www-form-urlencoded".to_string()],
        );
        HeadersBuilder::new(self.handle).add_headers(Some(&headers))?;

        let encoded: String = content
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        self.set_text_content(&encoded)
    }
}
