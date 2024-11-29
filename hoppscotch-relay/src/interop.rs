use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method = match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        };
        write!(f, "{}", method)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "camelCase")]
pub enum ContentType {
    #[serde(rename = "text")]
    Text { content: String },
    #[serde(rename = "json")]
    Json { content: serde_json::Value },
    #[serde(rename = "form")]
    Form { content: HashMap<String, Vec<u8>> },
    #[serde(rename = "urlencoded")]
    UrlEncoded { content: HashMap<String, String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "camelCase")]
pub enum AuthType {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "basic")]
    Basic { username: String, password: String },
    #[serde(rename = "bearer")]
    Bearer { token: String },
    #[serde(rename = "digest")]
    Digest {
        username: String,
        password: String,
        realm: Option<String>,
        nonce: Option<String>,
        opaque: Option<String>,
        algorithm: Option<DigestAlgorithm>,
        qop: Option<DigestQop>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DigestAlgorithm {
    #[serde(rename = "MD5")]
    Md5,
    #[serde(rename = "SHA-256")]
    Sha256,
    #[serde(rename = "SHA-512")]
    Sha512,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DigestQop {
    #[serde(rename = "auth")]
    Auth,
    #[serde(rename = "auth-int")]
    AuthInt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "camelCase")]
pub enum CertificateType {
    #[serde(rename = "pem")]
    Pem { cert: Vec<u8>, key: Vec<u8> },
    #[serde(rename = "pfx")]
    Pfx { data: Vec<u8>, password: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Security {
    pub certificates: Option<SecurityCertificates>,
    pub validate_certificates: bool,
    pub verify_host: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityCertificates {
    pub client: Option<CertificateType>,
    pub ca: Option<Vec<Vec<u8>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub id: i64,
    pub url: String,
    pub method: Method,
    pub headers: Option<HashMap<String, Vec<String>>>,
    pub params: Option<HashMap<String, String>>,
    pub content: Option<ContentType>,
    pub auth: Option<AuthType>,
    pub security: Option<Security>,
    pub proxy: Option<Proxy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub id: i64,
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, Vec<String>>,
    pub content: ContentType,
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMeta {
    pub timing: TimingInfo,
    pub size: SizeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimingInfo {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SizeInfo {
    pub headers: u64,
    pub body: u64,
    pub total: u64,
}
