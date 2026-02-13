pub mod editor;
mod iconpicker;

use crate::{config::AppConfig, pages::iconpicker::IconPicker, themes::Theme};
use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use cosmic::{
    Application, Element,
    app::{Core, Task, context_drawer},
    command::set_theme,
    cosmic_theme,
    iced::{
        Alignment, Length, Subscription,
        alignment::Horizontal,
        futures::{SinkExt as _, future},
    },
    surface, task, theme,
    widget::{
        self,
        menu::{self, ItemHeight, ItemWidth},
        nav_bar, responsive_menu_bar,
    },
};
use editor::AppEditor;
use ron::ser::to_string_pretty;
use std::{
    collections::HashMap,
    fs::read_dir,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::ExitStatus,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::oneshot,
};
use webapps::{APP_ICON, APP_ID, REPOSITORY, fl};

static MENU_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("responsive-menu"));

static SEARCH_ID: LazyLock<cosmic::widget::Id> =
    LazyLock::new(|| cosmic::widget::Id::new("search-input"));

#[derive(Debug, Clone)]
pub enum Message {
    ChangeUserTheme(usize),
    CloseDialog,
    CloseToast(widget::toaster::ToastId),
    Editor(editor::Message),
    Delete(widget::segmented_button::Entity),
    DeletionDone(widget::segmented_button::Entity),
    DuplicateApp(Box<editor::AppEditor>),
    DownloaderDone,
    DownloaderStarted,
    DownloaderStream(String),
    DownloaderStreamFinished,
    IconPicker(iconpicker::Message),
    IconsResult(Vec<String>),
    ImportThemeFilePicker,
    Launch(webapps::WebviewArgs),
    LaunchUrl(String),
    LoadThemes,
    OpenFileResult(Vec<String>),
    OpenIconPicker,
    OpenRepositoryUrl,
    OpenThemeResult(String),
    ConfirmDeletion(widget::segmented_button::Entity),
    PushIcon(webapps::Icon),
    ReloadNavbarItems,
    ResetSettings,
    SaveLauncher(webapps::launcher::WebAppLauncher),
    SetIcon(Option<webapps::Icon>),
    Surface(surface::Action),
    DownloaderStop,
    ExportApps,
    ExportAppsResult(Result<(), String>),
    ImportApps,
    ImportAppsFilePicked(Vec<String>),
    SearchApps(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(AppConfig),
    UpdateTheme(Box<Theme>),
    ClearAppData(String),
    ClearAppDataDone(Result<(), String>),
    FocusSearch,
    LaunchCurrentApp,
    DuplicateCurrentApp,
    OpenCurrentAppUrl,
    ToggleViewMode,
    SelectAppFromGrid(String),
    UpdateRunningApps(std::collections::HashSet<String>),
    ToggleBulkMode,
    ToggleBulkSelect(String),
    BulkDelete,
    BulkDeleteDone(usize),
    BulkExport,
    BulkExportResult(Result<(), String>),
    // empty message
    None,
}

#[derive(Debug, Clone)]
pub enum Page {
    Editor(AppEditor),
}

#[derive(Debug, Clone)]
pub enum Dialogs {
    IconPicker(IconPicker),
    Confirmation((widget::segmented_button::Entity, String)),
    IconsDownloader,
}

pub struct QuickWebApps {
    core: Core,
    context_page: ContextPage,
    nav: nav_bar::Model,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    config: AppConfig,
    page: Page,
    dialogs: Option<Dialogs>,
    downloader_started: bool,
    downloader_id: usize,
    downloader_output: String,
    search_query: String,
    cached_apps: Vec<webapps::launcher::WebAppLauncher>,
    themes_list: Vec<Theme>,
    theme_idx: Option<usize>,
    toasts: widget::toaster::Toasts<Message>,
    running_app_ids: std::collections::HashSet<String>,
    bulk_mode: bool,
    selected_app_ids: std::collections::HashSet<String>,
}

impl Application for QuickWebApps {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config = AppConfig::config();
        let add_page = Page::Editor(AppEditor::default());
        let nav = nav_bar::Model::default();

        let themes_list = Vec::new();

        let mut key_binds = HashMap::new();
        key_binds.insert(
            menu::KeyBind {
                modifiers: vec![menu::key_bind::Modifier::Ctrl],
                key: cosmic::iced_core::keyboard::Key::Character("n".into()),
            },
            MenuAction::NewApp,
        );
        key_binds.insert(
            menu::KeyBind {
                modifiers: vec![menu::key_bind::Modifier::Ctrl],
                key: cosmic::iced_core::keyboard::Key::Character("s".into()),
            },
            MenuAction::Save,
        );
        key_binds.insert(
            menu::KeyBind {
                modifiers: vec![menu::key_bind::Modifier::Ctrl],
                key: cosmic::iced_core::keyboard::Key::Character("f".into()),
            },
            MenuAction::FocusSearch,
        );
        key_binds.insert(
            menu::KeyBind {
                modifiers: vec![menu::key_bind::Modifier::Ctrl],
                key: cosmic::iced_core::keyboard::Key::Character("e".into()),
            },
            MenuAction::ExportApps,
        );
        key_binds.insert(
            menu::KeyBind {
                modifiers: vec![menu::key_bind::Modifier::Ctrl],
                key: cosmic::iced_core::keyboard::Key::Character("i".into()),
            },
            MenuAction::ImportApps,
        );
        key_binds.insert(
            menu::KeyBind {
                modifiers: vec![menu::key_bind::Modifier::Ctrl],
                key: cosmic::iced_core::keyboard::Key::Character("l".into()),
            },
            MenuAction::LaunchApp,
        );
        key_binds.insert(
            menu::KeyBind {
                modifiers: vec![menu::key_bind::Modifier::Ctrl],
                key: cosmic::iced_core::keyboard::Key::Character("d".into()),
            },
            MenuAction::DuplicateApp,
        );

        let windows = QuickWebApps {
            core,
            context_page: ContextPage::About,
            nav,
            key_binds,
            config,
            page: add_page,
            dialogs: None,
            downloader_started: false,
            downloader_id: 1,
            downloader_output: String::new(),
            search_query: String::new(),
            cached_apps: Vec::new(),
            themes_list,
            theme_idx: Some(0),
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
            running_app_ids: std::collections::HashSet::new(),
            bulk_mode: false,
            selected_app_ids: std::collections::HashSet::new(),
        };

        let tasks = vec![
            task::message(Message::ReloadNavbarItems),
            task::message(Message::LoadThemes),
            task::message(Message::UpdateTheme(Box::new(Theme::Light))),
        ];

        (windows, Task::batch(tasks))
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = Vec::new();

        subscriptions.push(
            self.core()
                .watch_config::<AppConfig>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        );

        // Poll for running webview processes every 5 seconds
        subscriptions.push(Subscription::run_with_id(
            "running-apps-poll",
            cosmic::iced::stream::channel(1, |mut channel| async move {
                loop {
                    let ids = tokio::task::spawn_blocking(webapps::running_webview_app_ids)
                        .await
                        .unwrap_or_default();
                    let _ = channel.send(Message::UpdateRunningApps(ids)).await;
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }),
        ));

        if self.downloader_started {
            subscriptions.push(Subscription::run_with_id(
                self.downloader_id,
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    let script = match webapps::add_icon_packs_install_script().await {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("Failed to create install script: {e}");
                            let _ = channel.send(Message::DownloaderStreamFinished).await;
                            return future::pending().await;
                        }
                    };
                    let mut child = match webapps::execute_script(script).await {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!("Failed to execute install script: {e}");
                            let _ = channel.send(Message::DownloaderStreamFinished).await;
                            return future::pending().await;
                        }
                    };
                    let stdout = match child.stdout.take() {
                        Some(s) => s,
                        None => {
                            tracing::error!("Child process has no stdout handle");
                            let _ = channel.send(Message::DownloaderStreamFinished).await;
                            return future::pending().await;
                        }
                    };

                    let mut reader = BufReader::new(stdout).lines();
                    let (tx, rx) = oneshot::channel::<ExitStatus>();

                    tokio::spawn(async move {
                        match child.wait().await {
                            Ok(status) => {
                                let _ = tx.send(status);
                            }
                            Err(e) => tracing::error!("Child process error: {e}"),
                        }
                    });

                    while let Ok(Some(line)) = reader.next_line().await {
                        _ = channel.send(Message::DownloaderStream(line)).await;
                    }

                    match rx.await {
                        Ok(es) => {
                            if es.success() {
                                let _ = channel.send(Message::DownloaderStreamFinished).await;
                            }
                        }
                        Err(_) => tracing::error!("the sender dropped"),
                    }

                    future::pending().await
                }),
            ));
        }

        Subscription::batch(subscriptions)
    }

    fn update(&mut self, message: Message) -> cosmic::Task<cosmic::Action<Message>> {
        let mut tasks: Vec<cosmic::Task<cosmic::Action<Message>>> = Vec::new();

        match message {
            Message::ChangeUserTheme(idx) => {
                if let Some(selected) = self.themes_list.get(idx) {
                    self.theme_idx = Some(idx);
                    tasks.push(task::message(cosmic::action::app(Message::UpdateTheme(
                        Box::new(selected.clone()),
                    ))));
                }
            }
            Message::CloseDialog => self.dialogs = None,
            Message::CloseToast(id) => {
                self.toasts.remove(id);
            }
            Message::ConfirmDeletion(id) => {
                let data = self.nav.data::<Page>(id);

                if let Some(page) = data {
                    let Page::Editor(app_editor) = page;
                    self.dialogs = Some(Dialogs::Confirmation((id, app_editor.app_title.clone())))
                };
            }
            Message::Editor(msg) => match &mut self.page {
                Page::Editor(app_editor) => tasks.push(app_editor.update(msg)),
            },
            Message::Delete(id) => {
                let data = self.nav.data::<Page>(id);

                if let Some(page) = data {
                    let Page::Editor(app_editor) = page;

                    if let Some(browser) = &app_editor.app_browser {
                        let launcher = webapps::launcher::WebAppLauncher {
                            browser: browser.clone(),
                            name: app_editor.app_title.clone(),
                            icon: app_editor.app_icon.clone(),
                            category: app_editor.app_category.clone(),
                        };

                        self.dialogs = None;
                        return task::future(async move {
                            if let Err(e) = launcher.delete().await {
                                tracing::error!("Failed to delete web app: {e}");
                                return cosmic::action::app(Message::CloseDialog);
                            }
                            cosmic::action::app(Message::DeletionDone(id))
                        });
                    }
                }
            }
            Message::DeletionDone(id) => {
                self.nav.remove(id);
                self.dialogs = None;
                self.page = Page::Editor(AppEditor::default());
                tasks.push(
                    self.toasts
                        .push(widget::toaster::Toast::new(fl!("toast-app-deleted")))
                        .map(cosmic::Action::App),
                );
            }
            Message::DuplicateApp(editor) => {
                self.page = Page::Editor(*editor);
                // Select the "Create new" entry so the user can save the duplicate
                let first_entity = { self.nav.iter().next() };
                if let Some(first) = first_entity {
                    self.nav.activate(first);
                }
            }
            Message::DownloaderDone => {
                self.downloader_started = false;
                return task::message(cosmic::action::app(Message::CloseDialog));
            }
            Message::DownloaderStarted => {
                self.dialogs = None;
                self.downloader_started = true;
                self.dialogs = Some(Dialogs::IconsDownloader)
            }
            Message::DownloaderStream(buffer) => {
                const MAX_OUTPUT_LEN: usize = 32_768;
                self.downloader_output.push_str(&format!("{buffer}\n"));
                if self.downloader_output.len() > MAX_OUTPUT_LEN {
                    let trim_at = self.downloader_output.len() - MAX_OUTPUT_LEN;
                    if let Some(newline) = self.downloader_output[trim_at..].find('\n') {
                        self.downloader_output =
                            self.downloader_output[trim_at + newline + 1..].to_string();
                    }
                }
            }
            Message::SearchApps(query) => {
                self.search_query = query;
                self.rebuild_nav_from_cache(None);
            }
            Message::ExportApps => {
                return task::future(async {
                    let title = fl!("file-dialog-export-title");
                    let label = fl!("file-dialog-save");
                    let response = match SelectedFiles::save_file()
                        .title(title.as_str())
                        .accept_label(label.as_str())
                        .modal(true)
                        .current_name("webapps-export.ron")
                        .send()
                        .await
                    {
                        Ok(r) => r.response(),
                        Err(e) => {
                            tracing::error!("Failed to open save dialog: {e}");
                            return cosmic::action::app(Message::ExportAppsResult(Err(fl!(
                                "toast-export-error"
                            ))));
                        }
                    };

                    if let Ok(result) = response {
                        let uris = result.uris();
                        if let Some(uri) = uris.first() {
                            let path = std::path::PathBuf::from(uri.path());
                            match webapps::launcher::export_all(&path) {
                                Ok(()) => {
                                    return cosmic::action::app(Message::ExportAppsResult(Ok(())));
                                }
                                Err(e) => {
                                    tracing::error!("Export failed: {e}");
                                    return cosmic::action::app(Message::ExportAppsResult(Err(
                                        fl!("toast-export-error"),
                                    )));
                                }
                            }
                        }
                    }
                    cosmic::action::none()
                });
            }
            Message::ExportAppsResult(result) => match result {
                Ok(()) => {
                    tasks.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!("toast-export-success")))
                            .map(cosmic::Action::App),
                    );
                }
                Err(msg) => {
                    tasks.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(msg))
                            .map(cosmic::Action::App),
                    );
                }
            },
            Message::ImportApps => {
                return task::future(async {
                    let title = fl!("file-dialog-import-title");
                    let label = fl!("file-dialog-import");
                    let filter_name = fl!("file-filter-ron");
                    let response = match SelectedFiles::open_file()
                        .title(title.as_str())
                        .accept_label(label.as_str())
                        .modal(true)
                        .multiple(false)
                        .filter(FileFilter::new(&filter_name).glob("*.ron"))
                        .send()
                        .await
                    {
                        Ok(r) => r.response(),
                        Err(e) => {
                            tracing::error!("Failed to open import dialog: {e}");
                            return cosmic::action::none();
                        }
                    };

                    if let Ok(result) = response {
                        let files: Vec<String> = result
                            .uris()
                            .iter()
                            .map(|file| file.path().to_string())
                            .collect();
                        if !files.is_empty() {
                            return cosmic::action::app(Message::ImportAppsFilePicked(files));
                        }
                    }
                    cosmic::action::none()
                });
            }
            Message::ImportAppsFilePicked(files) => {
                if let Some(file) = files.first() {
                    let decoded = match urlencoding::decode(file) {
                        Ok(d) => d.to_string(),
                        Err(e) => {
                            tracing::error!("Failed to decode import file path: {e}");
                            tasks.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(fl!("toast-import-error")))
                                    .map(cosmic::Action::App),
                            );
                            return Task::batch(tasks);
                        }
                    };
                    let path = std::path::PathBuf::from(&decoded);

                    match webapps::launcher::import_all(&path) {
                        Ok(apps) => {
                            let (saved, total) = webapps::launcher::save_imported(&apps);
                            let msg = if saved == total {
                                fl!("toast-import-success")
                            } else {
                                format!("{} ({}/{})", fl!("toast-import-success"), saved, total)
                            };
                            tasks.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(msg))
                                    .map(cosmic::Action::App),
                            );
                            return Task::batch(tasks.into_iter().chain(std::iter::once(
                                task::message(cosmic::action::app(Message::ReloadNavbarItems)),
                            )));
                        }
                        Err(e) => {
                            tracing::error!("Import failed: {e}");
                            tasks.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(fl!("toast-import-error")))
                                    .map(cosmic::Action::App),
                            );
                        }
                    }
                }
            }
            Message::DownloaderStop => {
                self.downloader_started = false;
                self.downloader_id += 1;
                self.downloader_output
                    .push_str(&fl!("downloader-canceled").to_string());
            }
            Message::DownloaderStreamFinished => {
                self.downloader_output
                    .push_str(&fl!("icons-installer-finished-waiting").to_string());

                return task::future(async {
                    tokio::time::sleep(Duration::from_secs_f32(3.0)).await;

                    cosmic::action::app(Message::DownloaderDone)
                });
            }
            Message::IconPicker(msg) => {
                if let Some(Dialogs::IconPicker(icon_picker)) = &mut self.dialogs {
                    tasks.push(icon_picker.update(msg));
                };
            }
            Message::IconsResult(result) => {
                if let Some(Dialogs::IconPicker(_icon_picker)) = &mut self.dialogs {
                    for path in result {
                        tasks.push(Task::perform(webapps::image_handle(path), |icon| {
                            if let Some(icon) = icon {
                                cosmic::Action::App(Message::PushIcon(icon))
                            } else {
                                cosmic::Action::None
                            }
                        }))
                    }
                };
            }
            Message::ImportThemeFilePicker => {
                return task::future(async {
                    let title = fl!("file-dialog-open-theme");
                    let label = fl!("open");
                    let filter_name = fl!("file-filter-ron-theme");
                    let response = match SelectedFiles::open_file()
                        .title(title.as_str())
                        .accept_label(label.as_str())
                        .modal(true)
                        .multiple(false)
                        .filter(FileFilter::new(&filter_name).glob("*.ron"))
                        .send()
                        .await
                    {
                        Ok(r) => r.response(),
                        Err(e) => {
                            tracing::error!("Failed to open theme file picker: {e}");
                            return cosmic::action::none();
                        }
                    };

                    if let Ok(result) = response {
                        let files = result
                            .uris()
                            .iter()
                            .map(|file| file.path().to_string())
                            .collect::<Vec<String>>();

                        if !files.is_empty() {
                            let decoded = match urlencoding::decode(&files[0]) {
                                Ok(d) => d.to_string(),
                                Err(e) => {
                                    tracing::error!("Failed to decode theme path: {e}");
                                    return cosmic::action::none();
                                }
                            };
                            return cosmic::action::app(Message::OpenThemeResult(decoded));
                        }
                        cosmic::action::none()
                    } else {
                        cosmic::action::none()
                    }
                });
            }
            Message::Launch(args) => {
                // #57: Update usage statistics
                let app_id_str = args.as_ref().to_string();
                let safe_id = webapps::browser::sanitize_app_id(&app_id_str);
                if let Some(db_path) = webapps::database_path(&format!("{safe_id}.ron")) {
                    if let Ok(content) = std::fs::read_to_string(&db_path) {
                        if let Ok(mut launcher) = ron::from_str::<webapps::launcher::WebAppLauncher>(&content) {
                            let count = launcher.browser.launch_count.unwrap_or(0);
                            launcher.browser.launch_count = Some(count + 1);
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            launcher.browser.last_launched = Some(now);
                            if let Ok(serialized) = to_string_pretty(&launcher, ron::ser::PrettyConfig::default()) {
                                let _ = std::fs::write(&db_path, serialized);
                            }
                        }
                    }
                }
                return Task::perform(
                    async move {
                        if let Err(e) = Command::new("dev.heppen.webapps.webview")
                            .args(args)
                            .spawn()
                        {
                            tracing::error!("Failed to spawn webview: {e}");
                        }
                    },
                    |_| cosmic::Action::App(Message::None),
                );
            }
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    tracing::error!("Failed to open {url:?}: {err}");
                }
            },
            Message::LoadThemes => {
                self.themes_list.clear();

                if std::env::var("XDG_CURRENT_DESKTOP") != Ok("COSMIC".to_string()) {
                    self.themes_list.push(Theme::Light);
                    self.themes_list.push(Theme::Dark);
                }

                let Some(folder) = webapps::themes_path("") else {
                    return Task::none();
                };
                let dir = read_dir(folder);

                if let Ok(files) = dir {
                    for path in files {
                        let Ok(dir_entry) = path else { continue };
                        let file_name = dir_entry.file_name();
                        let Some(name_str) = file_name.to_str() else {
                            continue;
                        };
                        let theme_name = name_str.replace(".ron", "");
                        let metadata = std::fs::metadata(dir_entry.path());

                        if let Ok(meta) = metadata {
                            if meta.is_file() {
                                let mut content: String = String::new();

                                let Ok(mut file) = std::fs::File::open(dir_entry.path()) else {
                                    continue;
                                };
                                let _ = file.read_to_string(&mut content);

                                let theme = Theme::build(theme_name.to_string(), content);

                                if theme_name == self.config.app_theme {
                                    tasks.push(task::message(cosmic::action::app(
                                        Message::UpdateTheme(Box::new(theme.clone())),
                                    )));
                                }

                                self.themes_list.push(theme);
                            }
                        }
                    }
                }

                self.theme_idx = self.themes_list.iter().position(|c| match c {
                    Theme::Light => self.config.app_theme == "COSMIC Light",
                    Theme::Dark => self.config.app_theme == "COSMIC Dark",
                    Theme::Custom(theme) => self.config.app_theme == theme.0,
                })
            }
            Message::OpenFileResult(file_paths) => {
                let mut moved: Vec<String> = Vec::new();

                for path in file_paths {
                    let buf = PathBuf::from(&path);
                    let icon_name = buf.file_stem();

                    if let Some(file_stem) = icon_name {
                        let stem_str = file_stem.to_str().unwrap_or("icon");
                        let ext_str = buf.extension().and_then(|e| e.to_str()).unwrap_or("png");
                        if let Some(final_path) = webapps::move_icon(&path, stem_str, ext_str) {
                            moved.push(final_path.display().to_string());
                        }
                    };
                }

                return task::message(cosmic::action::app(Message::IconsResult(moved)));
            }
            Message::OpenIconPicker => {
                self.dialogs = Some(Dialogs::IconPicker(IconPicker::default()));
            }
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }
            Message::OpenThemeResult(theme) => {
                if !theme.is_empty() {
                    let from_path = Path::new(&theme);
                    if let Some(file_name) = from_path.file_name() {
                        let file_name = file_name.to_string_lossy();

                        if let Some(dest) = webapps::themes_path(&file_name) {
                            if !dest.exists() {
                                let _ = std::fs::copy(from_path, dest);
                            }
                        }
                    }
                }

                tasks.push(task::message(cosmic::action::app(Message::LoadThemes)));
            }
            Message::PushIcon(icon) => {
                if let Some(Dialogs::IconPicker(icon_picker)) = &mut self.dialogs {
                    icon_picker.push_icon(icon);
                }
            }
            Message::ReloadNavbarItems => {
                // Read from disk and cache
                self.cached_apps = webapps::launcher::installed_webapps();
                self.cached_apps.sort_by(|a, b| {
                    let cat_cmp = a.category.name().cmp(&b.category.name());
                    cat_cmp.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });
                self.rebuild_nav_from_cache(None);
            }
            Message::ResetSettings => {
                if let Some(handler) = AppConfig::config_handler() {
                    let _ = self.config.set_app_theme(&handler, String::new());
                };

                return cosmic::command::set_theme(cosmic::Theme::light());
            }
            Message::SaveLauncher(launcher) => {
                if let Some(location) =
                    webapps::database_path(&format!("{}.ron", launcher.browser.app_id.as_ref()))
                {
                    let save_result = (|| -> Result<(), Box<dyn std::error::Error>> {
                        let content =
                            to_string_pretty(&launcher, ron::ser::PrettyConfig::default())?;
                        let mut f = std::fs::File::create(&location)?;
                        f.write_all(content.as_bytes())?;
                        Ok(())
                    })();

                    match save_result {
                        Ok(()) => {
                            tasks.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(fl!("toast-app-saved")))
                                    .map(cosmic::Action::App),
                            );
                        }
                        Err(e) => {
                            tracing::error!("Failed to save web app: {e}");
                            tasks.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(fl!("toast-save-error")))
                                    .map(cosmic::Action::App),
                            );
                            return Task::batch(tasks);
                        }
                    }

                    // Reload and re-select saved app
                    self.cached_apps = webapps::launcher::installed_webapps();
                    self.cached_apps.sort_by(|a, b| {
                        let cat_cmp = a.category.name().cmp(&b.category.name());
                        cat_cmp.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                    });
                    let app_id = launcher.browser.app_id.as_ref().to_string();
                    self.rebuild_nav_from_cache(Some(&app_id));
                    return Task::batch(tasks);
                }
            }
            Message::SetIcon(icon) => {
                let Page::Editor(app_editor) = &mut self.page;
                app_editor.update_icon(icon);
                self.dialogs = None;
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::UpdateTheme(theme) => {
                if let Theme::Custom(theme) = *theme {
                    if let Some(handler) = AppConfig::config_handler() {
                        let _ = self.config.set_app_theme(&handler, theme.0);
                    };
                    return set_theme(cosmic::Theme::custom(Arc::new(*theme.1)));
                };

                let theme_selector = match *theme {
                    Theme::Light => {
                        if let Some(handler) = AppConfig::config_handler() {
                            let _ = self.config.set_app_theme(&handler, "COSMIC Light".into());
                        };
                        set_theme(cosmic::theme::Theme::light())
                    }
                    Theme::Dark => {
                        if let Some(handler) = AppConfig::config_handler() {
                            let _ = self.config.set_app_theme(&handler, "COSMIC Dark".into());
                        };
                        set_theme(cosmic::theme::Theme::dark())
                    }
                    _ => Task::none(),
                };

                if std::env::var("XDG_CURRENT_DESKTOP") != Ok("COSMIC".to_string()) {
                    tasks.push(theme_selector);
                }
            }
            Message::ClearAppData(app_id) => {
                return task::future(async move {
                    match tokio::task::spawn_blocking(move || webapps::clear_profile_data(&app_id))
                        .await
                    {
                        Ok(Ok(())) => cosmic::action::app(Message::ClearAppDataDone(Ok(()))),
                        Ok(Err(e)) => {
                            cosmic::action::app(Message::ClearAppDataDone(Err(e.to_string())))
                        }
                        Err(e) => {
                            cosmic::action::app(Message::ClearAppDataDone(Err(e.to_string())))
                        }
                    }
                });
            }
            Message::ClearAppDataDone(result) => match result {
                Ok(()) => {
                    tasks.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!("toast-data-cleared")))
                            .map(cosmic::Action::App),
                    );
                }
                Err(msg) => {
                    tracing::error!("Failed to clear app data: {msg}");
                    tasks.push(
                        self.toasts
                            .push(widget::toaster::Toast::new(fl!("toast-data-clear-error")))
                            .map(cosmic::Action::App),
                    );
                }
            },
            Message::FocusSearch => {
                return widget::text_input::focus(SEARCH_ID.clone());
            }
            Message::LaunchCurrentApp => {
                let Page::Editor(editor) = &self.page;
                if let Some(browser) = &editor.app_browser {
                    let arg_id = browser.app_id.clone();
                    return task::future(async { crate::pages::Message::Launch(arg_id) });
                }
            }
            Message::DuplicateCurrentApp => {
                return task::message(cosmic::action::app(Message::Editor(
                    editor::Message::Duplicate,
                )));
            }
            Message::ToggleViewMode => {
                use crate::config::ViewMode;
                self.config.view_mode = match self.config.view_mode {
                    ViewMode::List => ViewMode::Grid,
                    ViewMode::Grid => ViewMode::List,
                };
                if let Some(handler) = AppConfig::config_handler() {
                    let _ = self.config.set_view_mode(&handler, self.config.view_mode);
                }
            }
            Message::SelectAppFromGrid(app_id) => {
                // Find the target entity first (immutable borrow)
                let target = self.nav.iter().find(|&entity| {
                    self.nav.data::<Page>(entity).is_some_and(|page| {
                        let Page::Editor(editor) = page;
                        editor
                            .app_browser
                            .as_ref()
                            .is_some_and(|b| b.app_id.as_ref() == app_id)
                    })
                });
                // Then activate it (mutable borrow)
                if let Some(entity) = target {
                    if let Some(page) = self.nav.data::<Page>(entity) {
                        self.page = page.clone();
                    }
                    self.nav.activate(entity);
                    // Switch to list view for editing
                    use crate::config::ViewMode;
                    self.config.view_mode = ViewMode::List;
                    if let Some(handler) = AppConfig::config_handler() {
                        let _ = self.config.set_view_mode(&handler, ViewMode::List);
                    }
                }
            }
            Message::UpdateRunningApps(ids) => {
                if ids != self.running_app_ids {
                    self.running_app_ids = ids;
                    // Rebuild nav to update running indicators
                    let active_app_id = match &self.page {
                        Page::Editor(editor) => editor
                            .app_browser
                            .as_ref()
                            .map(|b| b.app_id.as_ref().to_string()),
                    };
                    self.rebuild_nav_from_cache(active_app_id.as_deref());
                }
            }
            Message::OpenCurrentAppUrl => {
                let Page::Editor(editor) = &self.page;
                if !editor.app_url.is_empty() {
                    match open::that_detached(&editor.app_url) {
                        Ok(()) => {}
                        Err(err) => {
                            tracing::error!("Failed to open URL {:?}: {err}", editor.app_url);
                        }
                    }
                }
            }
            Message::ToggleBulkMode => {
                self.bulk_mode = !self.bulk_mode;
                if !self.bulk_mode {
                    self.selected_app_ids.clear();
                }
            }
            Message::ToggleBulkSelect(app_id) => {
                if self.selected_app_ids.contains(&app_id) {
                    self.selected_app_ids.remove(&app_id);
                } else {
                    self.selected_app_ids.insert(app_id);
                }
            }
            Message::BulkDelete => {
                let ids_to_delete: Vec<String> = self.selected_app_ids.iter().cloned().collect();
                let apps_to_delete: Vec<webapps::launcher::WebAppLauncher> = self
                    .cached_apps
                    .iter()
                    .filter(|app| ids_to_delete.contains(&app.browser.app_id.as_ref().to_string()))
                    .cloned()
                    .collect();

                let count = apps_to_delete.len();
                return task::future(async move {
                    for launcher in apps_to_delete {
                        if let Err(e) = launcher.delete().await {
                            tracing::error!("Failed to delete web app {}: {e}", launcher.name);
                        }
                    }
                    cosmic::action::app(Message::BulkDeleteDone(count))
                });
            }
            Message::BulkDeleteDone(count) => {
                self.bulk_mode = false;
                self.selected_app_ids.clear();
                tasks.push(
                    self.toasts
                        .push(widget::toaster::Toast::new(fl!("toast-bulk-deleted")))
                        .map(cosmic::Action::App),
                );
                tasks.push(task::message(cosmic::action::app(Message::ReloadNavbarItems)));
                tracing::info!("Bulk deleted {count} apps");
            }
            Message::BulkExport => {
                let ids_to_export: Vec<String> = self.selected_app_ids.iter().cloned().collect();
                let apps_to_export: Vec<webapps::launcher::WebAppLauncher> = self
                    .cached_apps
                    .iter()
                    .filter(|app| ids_to_export.contains(&app.browser.app_id.as_ref().to_string()))
                    .cloned()
                    .collect();

                return task::future(async move {
                    let title = fl!("file-dialog-export-title");
                    let label = fl!("file-dialog-save");
                    let response = match SelectedFiles::save_file()
                        .title(title.as_str())
                        .accept_label(label.as_str())
                        .modal(true)
                        .current_name("webapps-selected-export.ron")
                        .send()
                        .await
                    {
                        Ok(r) => r.response(),
                        Err(e) => {
                            tracing::error!("Failed to open save dialog: {e}");
                            return cosmic::action::app(Message::BulkExportResult(Err(
                                fl!("toast-export-error"),
                            )));
                        }
                    };

                    if let Ok(result) = response {
                        let uris = result.uris();
                        if let Some(uri) = uris.first() {
                            let path = std::path::PathBuf::from(uri.path());
                            let pretty = ron::ser::PrettyConfig::default();
                            match ron::ser::to_string_pretty(&apps_to_export, pretty) {
                                Ok(content) => match std::fs::write(&path, content) {
                                    Ok(()) => {
                                        return cosmic::action::app(Message::BulkExportResult(
                                            Ok(()),
                                        ));
                                    }
                                    Err(e) => {
                                        tracing::error!("Bulk export write failed: {e}");
                                        return cosmic::action::app(Message::BulkExportResult(
                                            Err(fl!("toast-export-error")),
                                        ));
                                    }
                                },
                                Err(e) => {
                                    tracing::error!("Bulk export serialize failed: {e}");
                                    return cosmic::action::app(Message::BulkExportResult(Err(
                                        fl!("toast-export-error"),
                                    )));
                                }
                            }
                        }
                    }
                    cosmic::action::none()
                });
            }
            Message::BulkExportResult(result) => {
                self.bulk_mode = false;
                self.selected_app_ids.clear();
                match result {
                    Ok(()) => {
                        tasks.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(fl!("toast-bulk-exported")))
                                .map(cosmic::Action::App),
                        );
                    }
                    Err(msg) => {
                        tasks.push(
                            self.toasts
                                .push(widget::toaster::Toast::new(msg))
                                .map(cosmic::Action::App),
                        );
                    }
                }
            }
            Message::None => (),
        };

        Task::batch(tasks)
    }

    fn header_start(&self) -> Vec<Element<'_, Message>> {
        vec![
            responsive_menu_bar()
                .item_height(ItemHeight::Dynamic(40))
                .item_width(ItemWidth::Uniform(240))
                .spacing(4.0)
                .into_element(
                    &self.core,
                    &self.key_binds,
                    MENU_ID.clone(),
                    Message::Surface,
                    vec![(
                        fl!("app"),
                        vec![
                            menu::Item::Button(fl!("new-app"), None, MenuAction::NewApp),
                            menu::Item::Divider,
                            menu::Item::Button(fl!("export-apps"), None, MenuAction::ExportApps),
                            menu::Item::Button(fl!("import-apps"), None, MenuAction::ImportApps),
                            menu::Item::Divider,
                            menu::Item::Button(fl!("settings"), None, MenuAction::Settings),
                            menu::Item::Button(fl!("about"), None, MenuAction::About),
                        ],
                    )],
                ),
            widget::text_input(fl!("search"), &self.search_query)
                .on_input(Message::SearchApps)
                .width(Length::Fixed(200.0))
                .id(SEARCH_ID.clone())
                .into(),
            widget::tooltip(
                widget::button::icon(match self.config.view_mode {
                    crate::config::ViewMode::List => widget::icon::from_name("view-grid-symbolic"),
                    crate::config::ViewMode::Grid => widget::icon::from_name("view-list-symbolic"),
                })
                .on_press(Message::ToggleViewMode),
                widget::text(fl!("toggle-view")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
            widget::button::standard(if self.bulk_mode {
                fl!("bulk-done")
            } else {
                fl!("bulk-select")
            })
            .on_press(Message::ToggleBulkMode)
            .into(),
        ]
    }

    fn nav_bar(&self) -> Option<Element<'_, cosmic::Action<Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = widget::nav_bar(nav_model, |id| {
            cosmic::Action::Cosmic(cosmic::app::Action::NavBar(id))
        })
        .on_close(|id| cosmic::action::app(Message::ConfirmDeletion(id)))
        .into_container()
        .width(Length::Shrink)
        .height(Length::Shrink);

        if !self.core().is_condensed() {
            nav = nav.max_width(280);
        }

        Some(Element::from(
            nav.width(Length::Shrink).height(Length::Shrink),
        ))
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Message> {
        self.nav.activate(id);
        if let Some(page) = self.nav.data::<Page>(id) {
            self.page = page.clone();

            // Auto-trigger thumbnail fetch for installed apps with URLs
            let Page::Editor(editor) = &self.page;
            if editor.is_installed
                && editor.thumbnail_handle.is_none()
                && !editor.thumbnail_loading
                && webapps::url_valid(&editor.app_url)
            {
                return task::message(Message::Editor(editor::Message::FetchThumbnail));
            }
        }
        Task::none()
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
            ContextPage::Settings => context_drawer::context_drawer(
                self.settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("settings")),
        })
    }

    fn on_escape(&mut self) -> Task<Message> {
        self.dialogs = None;
        self.core.window.show_context = false;

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let Page::Editor(content) = &self.page;

        // nav has 1 entry (the "Create new" placeholder) when no apps are installed
        let has_installed_apps = self.nav.iter().count() > 1;

        let main_content = match self.config.view_mode {
            crate::config::ViewMode::Grid if has_installed_apps => {
                // Grid/card view of all apps
                let query = self.search_query.to_lowercase();
                let card_elements: Vec<Element<'_, Message>> = self
                    .cached_apps
                    .iter()
                    .filter(|app| {
                        query.is_empty() || app.name.to_lowercase().contains(&query)
                    })
                    .map(|app| {
                        let is_running = self
                            .running_app_ids
                            .contains(app.browser.app_id.as_ref());

                        let display_name = if is_running {
                            format!("{} {}", fl!("running-indicator"), app.name)
                        } else {
                            app.name.clone()
                        };

                        let app_id = app.browser.app_id.as_ref().to_string();
                        let is_selected = self.selected_app_ids.contains(&app_id);
                        let btn_class = if self.bulk_mode && is_selected {
                            cosmic::style::Button::Suggested
                        } else {
                            cosmic::style::Button::Image
                        };
                        let press_msg = if self.bulk_mode {
                            Message::ToggleBulkSelect(app_id)
                        } else {
                            Message::SelectAppFromGrid(app_id)
                        };
                        widget::button::custom(
                            widget::column()
                                .spacing(8)
                                .push(
                                    widget::icon::from_name(app.icon.clone())
                                        .size(64),
                                )
                                .push(
                                    widget::text::body(display_name)
                                        .width(Length::Fixed(100.0))
                                        .align_x(Horizontal::Center),
                                )
                                .align_x(Alignment::Center),
                        )
                        .width(Length::Fixed(120.0))
                        .height(Length::Fixed(120.0))
                        .on_press(press_msg)
                        .class(btn_class)
                        .into()
                    })
                    .collect();

                let cards = widget::row::with_children(card_elements)
                    .spacing(12)
                    .wrap();

                let mut grid_col = widget::column().spacing(12);

                // Bulk action toolbar
                if self.bulk_mode && !self.selected_app_ids.is_empty() {
                    grid_col = grid_col.push(
                        widget::container(
                            widget::row()
                                .spacing(8)
                                .push(
                                    widget::button::destructive(fl!("bulk-delete"))
                                        .on_press(Message::BulkDelete),
                                )
                                .push(
                                    widget::button::standard(fl!("bulk-export"))
                                        .on_press(Message::BulkExport),
                                ),
                        )
                        .padding([0, 24]),
                    );
                }

                grid_col = grid_col.push(
                    widget::container(cards)
                        .padding(24)
                        .width(Length::Fill),
                );

                widget::container(grid_col)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .center_x(Length::Fill)
            }
            _ => {
                // List/editor view (default)
                let mut col = widget::column().spacing(12);

                if !has_installed_apps && !content.is_installed {
                    col = col.push(
                        widget::container(
                            widget::column()
                                .spacing(8)
                                .push(widget::text::title3(fl!("create-new-webapp")))
                                .push(widget::text::body(fl!("not-installed-header")))
                                .align_x(Alignment::Center),
                        )
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                        .padding([24, 0, 0, 0]),
                    );
                }

                // Quick-actions toolbar for installed apps
                if content.is_installed {
                    col = col.push(
                        widget::container(
                            widget::row()
                                .spacing(8)
                                .push(
                                    widget::button::standard(fl!("run-app"))
                                        .on_press(Message::LaunchCurrentApp),
                                )
                                .push(
                                    widget::button::standard(fl!("duplicate"))
                                        .on_press(Message::DuplicateCurrentApp),
                                )
                                .push(
                                    widget::button::standard(fl!("open-in-browser"))
                                        .on_press(Message::OpenCurrentAppUrl),
                                )
                                .push(widget::horizontal_space())
                                .push(
                                    widget::button::destructive(fl!("delete"))
                                        .on_press_maybe(
                                            self.nav.active_data::<Page>().and_then(|_| {
                                                Some(Message::ConfirmDeletion(self.nav.active()))
                                            }),
                                        ),
                                ),
                        )
                        .padding([0, 30]),
                    );
                }

                col = col.push(content.view().map(Message::Editor));

                widget::container(col)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .center_x(Length::Fill)
            }
        };

        widget::toaster::toaster(&self.toasts, main_content).into()
    }

    fn dialog(&self) -> Option<Element<'_, Message>> {
        if let Some(dialog) = &self.dialogs {
            let element = match dialog {
                Dialogs::IconPicker(icon_picker) => widget::dialog()
                    .primary_action(
                        widget::button::standard(fl!("close")).on_press(Message::CloseDialog),
                    )
                    .control(icon_picker.view().map(Message::IconPicker)),
                Dialogs::Confirmation((entity, title)) => widget::dialog()
                    .title(fl!("delete"))
                    .primary_action(
                        widget::button::destructive(fl!("yes"))
                            .on_press(Message::Delete(entity.to_owned())),
                    )
                    .secondary_action(
                        widget::button::suggested(fl!("no")).on_press(Message::CloseDialog),
                    )
                    .body(fl!(
                        "confirm-delete",
                        HashMap::from([("app", title.as_str())])
                    )),
                Dialogs::IconsDownloader => widget::dialog()
                    .title(fl!("icons-installer-header"))
                    .body(self.downloader_output.clone())
                    .primary_action(
                        widget::button::destructive(fl!("cancel"))
                            .on_press(Message::DownloaderStop),
                    )
                    .secondary_action(
                        widget::button::suggested(fl!("close")).on_press(Message::CloseDialog),
                    ),
            };

            return Some(element.into());
        };

        None
    }
}

impl QuickWebApps {
    /// Rebuild the nav bar from the in-memory app cache, applying the current search filter.
    /// If select_app_id is provided, attempts to re-select that app after rebuilding.
    fn rebuild_nav_from_cache(&mut self, select_app_id: Option<&str>) {
        self.nav.clear();

        self.nav
            .insert()
            .icon(widget::icon::from_name("list-add-symbolic"))
            .text(fl!("new-app"))
            .data::<Page>(Page::Editor(AppEditor::default()))
            .activate();

        let query = self.search_query.to_lowercase();
        let mut selected_entity = None;

        for app in &self.cached_apps {
            if !query.is_empty() && !app.name.to_lowercase().contains(&query) {
                continue;
            }
            let is_running = self
                .running_app_ids
                .contains(app.browser.app_id.as_ref());
            let display_name = if is_running {
                format!("{} {}", fl!("running-indicator"), app.name)
            } else {
                app.name.clone()
            };
            self.nav
                .insert()
                .icon(widget::icon::from_name(app.icon.clone()))
                .text(display_name)
                .data::<Page>(Page::Editor(editor::AppEditor::from(app.clone())))
                .closable();

            // Check if this is the app we want to re-select
            if let Some(target_id) = select_app_id {
                if app.browser.app_id.as_ref() == target_id {
                    // Find the entity we just inserted by iterating and comparing data
                    for entity in self.nav.iter() {
                        if let Some(Page::Editor(editor)) = self.nav.data::<Page>(entity) {
                            if let Some(browser) = &editor.app_browser {
                                if browser.app_id.as_ref() == target_id {
                                    selected_entity = Some((entity, app.clone()));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some((entity, app)) = selected_entity {
            self.nav.activate(entity);
            self.page = Page::Editor(editor::AppEditor::from(app));
        } else {
            self.page = Page::Editor(AppEditor::default());
        }
    }

    fn about(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        widget::column()
            .push(widget::image(widget::image::Handle::from_bytes(APP_ICON)))
            .push(widget::text::title3(fl!("app")))
            .push(
                widget::button::link(REPOSITORY)
                    .on_press(Message::OpenRepositoryUrl)
                    .padding(0),
            )
            .push(
                widget::column()
                    .push(widget::text::title3(fl!("support-me")))
                    .push(widget::text::body(fl!("support-body")))
                    .push(widget::button::link("github.com/sponsors/hepp3n").on_press(
                        Message::LaunchUrl("https://github.com/sponsors/hepp3n".to_string()),
                    ))
                    .push(widget::button::link("paypal.me/elevenhsoft").on_press(
                        Message::LaunchUrl("https://paypal.me/elevenhsoft".to_string()),
                    ))
                    .push(widget::button::link("ko-fi.com/elevenhsoft").on_press(
                        Message::LaunchUrl("https://ko-fi.com/elevenhsoft".to_string()),
                    ))
                    .align_x(Alignment::Center)
                    .spacing(space_xxs),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    fn settings(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        widget::column()
            .push(
                widget::settings::section()
                    .add(widget::settings::item(
                        fl!("import-theme"),
                        widget::button::standard(fl!("open"))
                            .on_press(Message::ImportThemeFilePicker),
                    ))
                    .add(widget::settings::item(
                        fl!("imported-themes"),
                        widget::dropdown(
                            &self.themes_list,
                            self.theme_idx,
                            Message::ChangeUserTheme,
                        ),
                    ))
                    .add(widget::settings::item(
                        fl!("reset-settings"),
                        widget::button::standard(fl!("reset")).on_press(Message::ResetSettings),
                    )),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    DuplicateApp,
    ExportApps,
    FocusSearch,
    ImportApps,
    LaunchApp,
    NewApp,
    Save,
    Settings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::DuplicateApp => Message::Editor(editor::Message::Duplicate),
            MenuAction::ExportApps => Message::ExportApps,
            MenuAction::FocusSearch => Message::FocusSearch,
            MenuAction::ImportApps => Message::ImportApps,
            MenuAction::LaunchApp => Message::Editor(editor::Message::LaunchApp),
            MenuAction::NewApp => Message::ReloadNavbarItems,
            MenuAction::Save => Message::Editor(editor::Message::Done),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
        }
    }
}
