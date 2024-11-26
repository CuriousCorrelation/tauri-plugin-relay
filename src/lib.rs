use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::HoppscotchRelay;
#[cfg(mobile)]
use mobile::HoppscotchRelay;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the hoppscotch-relay APIs.
pub trait HoppscotchRelayExt<R: Runtime> {
    fn hoppscotch_relay(&self) -> &HoppscotchRelay<R>;
}

impl<R: Runtime, T: Manager<R>> crate::HoppscotchRelayExt<R> for T {
    fn hoppscotch_relay(&self) -> &HoppscotchRelay<R> {
        self.state::<HoppscotchRelay<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("hoppscotch-relay")
        .invoke_handler(tauri::generate_handler![commands::run, commands::cancel])
        .setup(|app, api| {
            #[cfg(mobile)]
            let hoppscotch_relay = mobile::init(app, api)?;
            #[cfg(desktop)]
            let hoppscotch_relay = desktop::init(app, api)?;
            app.manage(hoppscotch_relay);
            Ok(())
        })
        .build()
}
