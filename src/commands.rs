use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::HoppscotchRelayExt;
use crate::Result;

#[command]
pub(crate) async fn run<R: Runtime>(app: AppHandle<R>, payload: RunRequest) -> Result<RunResponse> {
    app.hoppscotch_relay().run(payload)
}

#[command]
pub(crate) async fn cancel<R: Runtime>(
    app: AppHandle<R>,
    payload: CancelRequest,
) -> Result<CancelResponse> {
    app.hoppscotch_relay().cancel(payload)
}
