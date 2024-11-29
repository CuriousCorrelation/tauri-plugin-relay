use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum InterceptorError {
    #[error("Unsupported feature '{feature}' in interceptor '{interceptor}': {message}")]
    #[serde(rename = "unsupported_feature")]
    UnsupportedFeature {
        feature: String,
        message: String,
        interceptor: String,
    },

    #[error("Network error: {message}")]
    #[serde(rename = "network")]
    Network {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    #[error("Request timed out during {}", .phase.as_ref().map_or("execution", |p| p.as_str()))]
    #[serde(rename = "timeout")]
    Timeout {
        message: String,
        phase: Option<TimeoutPhase>,
    },

    #[error("Certificate error: {message}")]
    #[serde(rename = "certificate")]
    Certificate {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    #[error("Failed to parse response: {message}")]
    #[serde(rename = "parse")]
    Parse {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<String>,
    },

    #[error("Request aborted: {message}")]
    #[serde(rename = "abort")]
    Abort { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TimeoutPhase {
    #[serde(rename = "connect")]
    Connect,
    #[serde(rename = "tls")]
    Tls,
    #[serde(rename = "response")]
    Response,
}

impl TimeoutPhase {
    fn as_str(&self) -> &'static str {
        match self {
            TimeoutPhase::Connect => "connection establishment",
            TimeoutPhase::Tls => "TLS handshake",
            TimeoutPhase::Response => "response waiting",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum InterceptorResult<Response> {
    #[serde(rename = "success")]
    Success { response: Response },
    #[serde(rename = "error")]
    Error { error: InterceptorError },
}

pub type RequestResult<T> = std::result::Result<T, InterceptorError>;
