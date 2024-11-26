use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_hoppscotch_relay);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<HoppscotchRelay<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin("", "ExamplePlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_hoppscotch_relay)?;
    Ok(HoppscotchRelay(handle))
}

/// Access to the hoppscotch-relay APIs.
pub struct HoppscotchRelay<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> HoppscotchRelay<R> {
    pub fn run(&self, payload: RunRequest) -> crate::Result<RunResponse> {
        self.0.run_mobile_plugin("run", payload).map_err(Into::into)
    }
}
