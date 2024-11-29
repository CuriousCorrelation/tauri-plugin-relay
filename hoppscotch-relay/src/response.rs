use std::{collections::HashMap, time::SystemTime};

use crate::{
    error::{InterceptorError, RequestResult},
    interop::{ContentType, Response, ResponseMeta, SizeInfo, TimingInfo},
    util::get_status_text,
};

pub(crate) struct ResponseHandler {
    id: i64,
    headers: HashMap<String, Vec<String>>,
    body: Vec<u8>,
    status: u16,
    header_size: u64,
    start_time: SystemTime,
    end_time: SystemTime,
}

impl ResponseHandler {
    pub(crate) fn new(
        id: i64,
        headers: HashMap<String, Vec<String>>,
        body: Vec<u8>,
        status: u16,
        header_size: u64,
        start_time: SystemTime,
        end_time: SystemTime,
    ) -> Self {
        Self {
            id,
            headers,
            body,
            status,
            header_size,
            start_time,
            end_time,
        }
    }

    #[tracing::instrument(skip(self), fields(request_id = self.id), level = "debug")]
    pub(crate) fn build(self) -> RequestResult<Response> {
        tracing::debug!(status = self.status, "Building response");
        let content = self.determine_content()?;
        let timing = self.calculate_timing()?;
        let size = SizeInfo {
            headers: self.header_size,
            body: self.body.len() as u64,
            total: self.header_size + self.body.len() as u64,
        };

        tracing::debug!(
            status = self.status,
            content_type = ?content,
            body_size = size.body,
            total_size = size.total,
            "Response built successfully"
        );

        Ok(Response {
            id: self.id,
            status: self.status,
            status_text: get_status_text(self.status).to_string(),
            content,
            headers: self.headers,
            meta: ResponseMeta { timing, size },
        })
    }

    fn determine_content(&self) -> RequestResult<ContentType> {
        tracing::trace!("Determining response content type");
        Ok(
            if let Some(content_types) = self.headers.get("Content-Type") {
                if let Some(content_type) = content_types.first() {
                    if content_type.starts_with("application/json") {
                        match serde_json::from_slice(&self.body) {
                            Ok(json) => {
                                tracing::trace!("Parsed JSON response");
                                ContentType::Json { content: json }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    "Failed to parse JSON response, falling back to text"
                                );
                                ContentType::Text {
                                    content: String::from_utf8_lossy(&self.body).into_owned(),
                                }
                            }
                        }
                    } else {
                        tracing::trace!(content_type = ?content_type, "Non-JSON response");
                        ContentType::Text {
                            content: String::from_utf8_lossy(&self.body).into_owned(),
                        }
                    }
                } else {
                    tracing::trace!("Empty content type header, treating as text");
                    ContentType::Text {
                        content: String::from_utf8_lossy(&self.body).into_owned(),
                    }
                }
            } else {
                tracing::trace!("No content type header, treating as text");
                ContentType::Text {
                    content: String::from_utf8_lossy(&self.body).into_owned(),
                }
            },
        )
    }

    fn calculate_timing(&self) -> RequestResult<TimingInfo> {
        let start_ms = self
            .start_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to get start time");
                InterceptorError::Parse {
                    message: "Failed to get start time".into(),
                    cause: Some(e.to_string()),
                }
            })?
            .as_millis() as u64;

        let end_ms = self
            .end_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to get end time");
                InterceptorError::Parse {
                    message: "Failed to get end time".into(),
                    cause: Some(e.to_string()),
                }
            })?
            .as_millis() as u64;

        tracing::trace!(
            start_ms = start_ms,
            end_ms = end_ms,
            duration_ms = end_ms - start_ms,
            "Calculated request timing"
        );

        Ok(TimingInfo {
            start: start_ms,
            end: end_ms,
        })
    }
}
