use crate::commands::config::get_config;
use crate::log_info;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::{Result, UpdaterExt};

pub async fn update_app(app: AppHandle) -> Result<()> {
  let config = get_config().unwrap_or_default();
  if !config.auto_update {
    log_info!("Auto-update is disabled in config, skipping update check");
    return Ok(());
  }

  if let Some(update) = app.updater()?.check().await? {
    app.emit("update-available", ()).unwrap();
    let mut downloaded = 0;
    update
      .download_and_install(
        |chunk_length, content_length| {
          downloaded += chunk_length;
          app
            .emit(
              "update-progress",
              serde_json::json!({
                "downloaded": downloaded,
                "total": content_length.unwrap_or(0)
              }),
            )
            .unwrap();
        },
        || {
          app.emit("update-downloaded", ()).unwrap();
        },
      )
      .await?;
    app.emit("update-ready", ()).unwrap();
  }

  Ok(())
}
