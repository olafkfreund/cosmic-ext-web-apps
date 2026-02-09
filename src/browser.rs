use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Maximum length for a sanitized app ID.
const MAX_APP_ID_LEN: usize = 128;

/// Sanitize an app ID for safe use in filesystem paths and desktop entry filenames.
/// Removes path separators, traversal sequences, and enforces length limits.
/// Returns an empty string if the input is empty after sanitization.
pub fn sanitize_app_id(id: &str) -> String {
    let mut sanitized: String = id
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect();

    while sanitized.contains("..") {
        sanitized = sanitized.replace("..", "");
    }

    if sanitized.len() > MAX_APP_ID_LEN {
        sanitized[..MAX_APP_ID_LEN].to_string()
    } else {
        sanitized
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Browser {
    pub app_id: crate::WebviewArgs,
    pub window_title: Option<String>,
    pub url: Option<String>,
    pub profile: Option<PathBuf>,
    pub window_size: Option<crate::WindowSize>,
    pub window_decorations: Option<bool>,
    pub private_mode: Option<bool>,
    pub try_simulate_mobile: Option<bool>,
}

impl Browser {
    pub fn new(app_id: &str, with_profile: bool) -> Self {
        let safe_id = sanitize_app_id(app_id);
        let mut browser = Self {
            app_id: crate::WebviewArgs {
                id: safe_id.clone(),
            },
            window_title: None,
            url: None,
            profile: None,
            window_size: None,
            window_decorations: None,
            private_mode: None,
            try_simulate_mobile: None,
        };

        if with_profile {
            if let Some(xdg_data) = dirs::data_dir() {
                let path = xdg_data.join(crate::APP_ID).join("profiles").join(&safe_id);
                browser.profile = Some(path);
            }
        };

        browser
    }

    pub fn from_appid(id: &str) -> Option<Self> {
        let safe_id = sanitize_app_id(id);
        let db_path = crate::database_path(&format!("{safe_id}.ron"))?;

        let content = std::fs::read_to_string(&db_path).ok()?;

        // Same 64KB safety limit used in launcher::installed_webapps()
        const MAX_RON_SIZE: usize = 64 * 1024;
        if content.len() > MAX_RON_SIZE {
            tracing::warn!("RON file too large: {}", db_path.display());
            return None;
        }

        let launcher: crate::launcher::WebAppLauncher = ron::from_str(&content).ok()?;
        Some(launcher.browser)
    }

    pub fn get_exec(&self) -> String {
        format!("{}.webview {}", crate::APP_ID, self.app_id.as_ref())
    }

    pub fn delete(&self) {
        if self.profile.is_some() {
            if let Some(xdg_data) = dirs::data_dir() {
                let path = xdg_data
                    .join(crate::APP_ID)
                    .join("profiles")
                    .join(self.app_id.as_ref());
                if let Err(e) = std::fs::remove_dir_all(&path) {
                    tracing::error!("Failed to delete profile directory: {e}");
                }
            }
        }
    }
}
