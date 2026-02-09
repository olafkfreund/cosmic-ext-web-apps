use ashpd::desktop::{
    dynamic_launcher::{DynamicLauncherProxy, PrepareInstallOptions},
    Icon,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    io::Read,
};
use tokio::fs::remove_file;

use crate::APP_ID;

/// Sanitize a string for use in a desktop entry field.
/// Strips newlines and carriage returns to prevent key injection.
fn sanitize_desktop_field(s: &str) -> String {
    s.chars().filter(|c| *c != '\n' && *c != '\r').collect()
}

pub fn webapplauncher_is_valid(
    icon: &str,
    name: &str,
    url: &Option<String>,
    category: &crate::Category,
) -> bool {
    if let Some(url) = url {
        if crate::url_valid(url)
            && !name.is_empty()
            && !icon.is_empty()
            && !url.is_empty()
            && category != &crate::Category::None
        {
            return true;
        }
    }

    false
}

/// Maximum size for a single RON database file (64 KB).
const MAX_RON_FILE_SIZE: u64 = 64 * 1024;

pub fn installed_webapps() -> Vec<WebAppLauncher> {
    let mut webapps = Vec::new();

    if let Some(data_dir) = dirs::data_dir() {
        if let Ok(entries) = fs::read_dir(data_dir.join(APP_ID).join("database")) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    if metadata.len() > MAX_RON_FILE_SIZE {
                        tracing::warn!("Skipping oversized file {:?} ({} bytes)", entry.path(), metadata.len());
                        continue;
                    }

                    let file = std::fs::File::open(entry.path());
                    let mut content = String::new();

                    if let Ok(mut f) = file {
                        if let Err(e) = f.read_to_string(&mut content) {
                            tracing::warn!("Failed to read {:?}: {e}", entry.path());
                            continue;
                        }
                        if let Ok(launcher) = ron::from_str::<WebAppLauncher>(&content) {
                            webapps.push(launcher);
                        }
                    }
                }
            }
        }
    }

    webapps
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebAppLauncher {
    pub browser: crate::browser::Browser,
    pub name: String,
    pub icon: String,
    pub category: crate::Category,
}

impl WebAppLauncher {
    pub async fn create(&self) -> Result<(), Box<dyn std::error::Error>> {
        let safe_name = sanitize_desktop_field(&self.name);
        let safe_wm_class = sanitize_desktop_field(&self.browser.app_id.id);
        let safe_exec = sanitize_desktop_field(&self.browser.get_exec());

        let mut desktop_entry = String::new();

        desktop_entry.push_str("[Desktop Entry]\n");
        desktop_entry.push_str("Version=1.0\n");
        desktop_entry.push_str("Type=Application\n");
        desktop_entry.push_str(&format!("Name={safe_name}\n"));
        desktop_entry.push_str("Comment=Quick WebApp\n");
        desktop_entry.push_str(&format!("Exec={safe_exec}\n"));
        desktop_entry.push_str(&format!("StartupWMClass={safe_wm_class}\n"));
        desktop_entry.push_str(&format!("Categories={}\n", self.category.as_ref()));
        desktop_entry.push_str("Actions=new-window;\n");
        desktop_entry.push_str("\n[Desktop Action new-window]\n");
        desktop_entry.push_str("Name=New Window\n");
        desktop_entry.push_str(&format!("Exec={safe_exec}\n"));

        let proxy = DynamicLauncherProxy::new().await?;

        let mut f = std::fs::File::open(&self.icon)?;
        let metadata = std::fs::metadata(&self.icon)?;
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer)?;

        let icon = Icon::Bytes(buffer);
        let response = proxy
            .prepare_install(None, &self.name, icon, PrepareInstallOptions::default())
            .await?
            .response()?;

        let token = response.token();

        tracing::debug!("Installing desktop entry:\n{desktop_entry}");

        proxy
            .install(
                &token,
                &format!("{}.{}.desktop", &APP_ID, self.browser.app_id.id),
                &desktop_entry,
            )
            .await?;

        Ok(())
    }

    pub async fn delete(&self) -> Result<(), Box<dyn std::error::Error>> {
        let proxy = DynamicLauncherProxy::new().await?;

        proxy
            .uninstall(&format!("{}.{}.desktop", &APP_ID, self.browser.app_id.id))
            .await?;

        if let Some(path) = crate::database_path(&format!("{}.ron", self.browser.app_id.as_ref())) {
            remove_file(path).await?;
        }

        self.browser.delete();

        Ok(())
    }
}

/// Export all installed web apps to a RON file.
pub fn export_all(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let apps = installed_webapps();
    let config = ron::ser::PrettyConfig::default();
    let content = ron::ser::to_string_pretty(&apps, config)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Import web apps from a RON file. Returns the number of apps imported.
pub fn import_all(path: &std::path::Path) -> Result<Vec<WebAppLauncher>, Box<dyn std::error::Error>> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_RON_FILE_SIZE * 100 {
        return Err("Import file too large".into());
    }
    let content = std::fs::read_to_string(path)?;
    let apps: Vec<WebAppLauncher> = ron::from_str(&content)?;
    Ok(apps)
}
