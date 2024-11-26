use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::{models::*, Result};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> Result<HoppscotchRelay<R>> {
    Ok(HoppscotchRelay(app.clone()))
}

/// Access to the hoppscotch-relay APIs.
pub struct HoppscotchRelay<R: Runtime>(AppHandle<R>);

impl<R: Runtime> HoppscotchRelay<R> {
    pub fn run(&self, payload: RunRequest) -> Result<RunResponse> {
        Ok(RunResponse {
            value: hoppscotch_relay::run(payload.req),
        })
    }

    pub fn cancel(&self, payload: CancelRequest) -> Result<CancelResponse> {
        hoppscotch_relay::cancel(payload.req_id);
        Ok(CancelResponse {})
    }
}
