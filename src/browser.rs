use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Sanitize an app ID for safe use in filesystem paths and desktop entry filenames.
/// Removes path separators and traversal sequences.
pub fn sanitize_app_id(id: &str) -> String {
    id.replace(['/', '\\', '\0'], "")
        .replace("..", "")
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
        if let Some(launcher) = crate::launcher::installed_webapps()
            .iter()
            .find(|launcher| launcher.browser.app_id.as_ref() == id)
        {
            return Some(launcher.browser.clone());
        };

        None
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
                    eprintln!("Failed to delete profile directory: {}", e);
                }
            }
        }
    }
}
