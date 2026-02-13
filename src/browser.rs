use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Maximum length for a sanitized app ID.
const MAX_APP_ID_LEN: usize = 128;

/// User agent configuration for web apps.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub enum UserAgent {
    #[default]
    Default,
    Mobile,
    Custom(String),
}

/// Permission policy for web app capabilities.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PermissionPolicy {
    pub allow_camera: bool,
    pub allow_microphone: bool,
    pub allow_geolocation: bool,
    pub allow_notifications: bool,
}

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
    pub custom_css: Option<String>,
    pub custom_js: Option<String>,
    pub user_agent: Option<UserAgent>,
    pub permissions: Option<PermissionPolicy>,
    pub url_schemes: Option<Vec<String>>,
    // #53: Content blocking (ads/trackers)
    pub content_blocking: Option<bool>,
    // #60: Block third-party cookies
    pub block_third_party_cookies: Option<bool>,
    // #61: Block WebRTC IP leak
    pub block_webrtc: Option<bool>,
    // #54: HTTP/HTTPS proxy
    pub proxy_url: Option<String>,
    // #55: Page zoom level
    pub zoom_level: Option<f64>,
    // #56: Session restore
    pub restore_session: Option<bool>,
    pub last_url: Option<String>,
    // #57: Usage statistics
    pub launch_count: Option<u64>,
    pub last_launched: Option<u64>,
    // #59: Minimize to background on close
    pub minimize_to_background: Option<bool>,
    // #62: Auto dark mode
    pub auto_dark_mode: Option<bool>,
}

impl Browser {
    pub fn new(app_id: &str, with_profile: bool) -> Self {
        let safe_id = sanitize_app_id(app_id);
        let mut browser = Self {
            app_id: crate::WebviewArgs {
                id: safe_id.clone(),
                private: false,
            },
            window_title: None,
            url: None,
            profile: None,
            window_size: None,
            window_decorations: None,
            private_mode: None,
            try_simulate_mobile: None,
            custom_css: None,
            custom_js: None,
            user_agent: None,
            permissions: None,
            url_schemes: None,
            content_blocking: None,
            block_third_party_cookies: None,
            block_webrtc: None,
            proxy_url: None,
            zoom_level: None,
            restore_session: None,
            last_url: None,
            launch_count: None,
            last_launched: None,
            minimize_to_background: None,
            auto_dark_mode: None,
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
