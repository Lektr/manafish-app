use crate::config::get_config_from_file;
use crate::log_info;
use crate::toast::{toast_info, toast_loading};
use tauri::AppHandle;
use tauri_plugin_updater::{Result, UpdaterExt};

pub async fn update_app(app: AppHandle) -> Result<()> {
  let config = get_config_from_file();
  if !config.auto_update {
    log_info!("Auto-update is disabled in config, skipping update check");
    return Ok(());
  }

  if let Some(update) = app.updater()?.check().await? {
    toast_info(
      None,
      "Update available".to_string(),
      Some("Downloading update...".to_string()),
      None,
    );
    let mut downloaded = 0;
    update
      .download_and_install(
        |chunk_length, content_length_opt| {
          let actual_content_length = content_length_opt.unwrap_or(0);
          downloaded += chunk_length;
          if actual_content_length > 0 {
            let progress_percent =
              (downloaded as f64 / actual_content_length as f64 * 100.0).round() as u64;
            toast_loading(
              Some("update-progress".to_string()),
              format!("Downloading: {}%", progress_percent),
              None,
              None,
            );
          }
        },
        || {
          toast_info(
            Some("update-progress".to_string()),
            "Update downloaded".to_string(),
            Some("Installing...".to_string()),
            None,
          );
        },
      )
      .await?;
    toast_info(
      Some("update-progress".to_string()),
      "Update ready".to_string(),
      Some("Restart the app to apply the update.".to_string()),
      None,
    );
  }

  Ok(())
}
