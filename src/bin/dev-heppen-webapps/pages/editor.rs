use cosmic::{
    Element, Task,
    action::Action,
    iced::{Length, alignment::Vertical},
    style, task,
    widget::{self},
};
use rand::{Rng, rng};
use strum::IntoEnumIterator as _;
use webapps::fl;

use crate::pages;

/// Filter a string to only contain digits and dots (for numeric input fields).
fn filter_numeric(input: String) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect()
}

#[derive(Debug, Clone)]
pub struct AppEditor {
    pub app_browser: Option<webapps::browser::Browser>,
    pub app_title: String,
    pub app_url: String,
    pub app_icon: String,
    pub app_category: webapps::Category,
    pub app_persistent: bool,
    pub app_window_width: String,
    pub app_window_height: String,
    pub app_window_size: webapps::WindowSize,
    pub app_window_decorations: bool,
    pub app_private_mode: bool,
    pub app_simulate_mobile: bool,
    pub app_custom_css: String,
    pub app_custom_js: String,
    pub selected_icon: Option<webapps::Icon>,
    pub categories: Vec<String>,
    pub category_idx: Option<usize>,
    pub is_installed: bool,
    pub app_user_agent: usize,
    pub app_custom_ua: String,
    pub user_agent_options: Vec<String>,
    pub app_allow_camera: bool,
    pub app_allow_microphone: bool,
    pub app_allow_geolocation: bool,
    pub app_allow_notifications: bool,
    pub app_url_schemes: String,
    pub show_advanced: bool,
    pub thumbnail_handle: Option<widget::image::Handle>,
    pub thumbnail_loading: bool,
    // #53, #60, #61: Privacy features
    pub app_content_blocking: bool,
    pub app_block_cookies: bool,
    pub app_block_webrtc: bool,
    // #54: Proxy
    pub app_proxy_url: String,
    // #55: Zoom
    pub app_zoom_level: String,
    // #56: Session restore
    pub app_restore_session: bool,
    // #57: Usage statistics (read-only display)
    pub app_launch_count: u64,
    pub app_last_launched: Option<u64>,
    // #59: Minimize to background
    pub app_minimize_to_background: bool,
    // #62: Auto dark mode
    pub app_auto_dark_mode: bool,
}

impl Default for AppEditor {
    fn default() -> Self {
        let categories = webapps::Category::iter()
            .map(|c| c.name())
            .collect::<Vec<String>>();

        AppEditor {
            app_browser: None,
            app_title: String::new(),
            app_url: String::new(),
            app_icon: String::new(),
            app_category: webapps::Category::default(),
            app_persistent: false,
            app_window_width: webapps::DEFAULT_WINDOW_WIDTH.to_string(),
            app_window_height: webapps::DEFAULT_WINDOW_HEIGHT.to_string(),
            app_window_size: webapps::WindowSize::default(),
            app_window_decorations: true,
            app_private_mode: false,
            app_simulate_mobile: false,
            app_custom_css: String::new(),
            app_custom_js: String::new(),
            selected_icon: None,
            categories,
            category_idx: Some(0),
            is_installed: false,
            app_user_agent: 0,
            app_custom_ua: String::new(),
            user_agent_options: vec![
                fl!("user-agent-default"),
                fl!("user-agent-mobile"),
                fl!("user-agent-custom"),
            ],
            app_allow_camera: false,
            app_allow_microphone: false,
            app_allow_geolocation: false,
            app_allow_notifications: false,
            app_url_schemes: String::new(),
            show_advanced: false,
            thumbnail_handle: None,
            thumbnail_loading: false,
            app_content_blocking: false,
            app_block_cookies: false,
            app_block_webrtc: false,
            app_proxy_url: String::new(),
            app_zoom_level: String::from("1.0"),
            app_restore_session: false,
            app_launch_count: 0,
            app_last_launched: None,
            app_minimize_to_background: false,
            app_auto_dark_mode: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Category(usize),
    Done,
    DownloadFavicon,
    Duplicate,
    FaviconResult(Option<String>),
    PersistentProfile(bool),
    LaunchApp,
    OpenIconPicker,
    Title(String),
    Url(String),
    WindowWidth(String),
    WindowHeight(String),
    WindowDecorations(bool),
    AppIncognito(bool),
    AppSimulateMobile(bool),
    CustomCss(String),
    CustomJs(String),
    UserAgentSelect(usize),
    CustomUserAgent(String),
    AllowCamera(bool),
    AllowMicrophone(bool),
    AllowGeolocation(bool),
    AllowNotifications(bool),
    ClearAppData,
    UrlSchemes(String),
    SiteTitleResult(Option<String>),
    ToggleAdvanced(bool),
    FetchThumbnail,
    ThumbnailResult(Option<String>),
    ThumbnailLoaded(Option<widget::image::Handle>),
    ContentBlocking(bool),
    BlockThirdPartyCookies(bool),
    BlockWebRTC(bool),
    ProxyUrl(String),
    ZoomLevel(String),
    RestoreSession(bool),
    MinimizeToBackground(bool),
    AutoDarkMode(bool),
}

impl AppEditor {
    pub fn from(launcher: webapps::launcher::WebAppLauncher) -> Self {
        let window_size = launcher.browser.window_size.clone().unwrap_or_default();
        let window_decorations = launcher.browser.window_decorations.unwrap_or_default();
        let incognito = launcher.browser.private_mode.unwrap_or_default();
        let simulate_mobile = launcher.browser.try_simulate_mobile.unwrap_or_default();

        let mut editor = AppEditor::default();

        editor.app_browser = Some(launcher.browser.clone());
        editor.app_title = launcher.name.clone();
        editor.app_url = launcher.browser.url.clone().unwrap_or_default();
        editor.app_icon = launcher.icon.clone();
        editor.app_category = launcher.category.clone();
        editor.app_persistent = launcher.browser.profile.is_some();
        editor.app_window_width = window_size.0.to_string();
        editor.app_window_height = window_size.1.to_string();
        editor.app_window_size = window_size;
        editor.app_window_decorations = window_decorations;
        editor.app_private_mode = incognito;
        editor.app_simulate_mobile = simulate_mobile;
        editor.app_custom_css = launcher.browser.custom_css.clone().unwrap_or_default();
        editor.app_custom_js = launcher.browser.custom_js.clone().unwrap_or_default();
        editor.category_idx = editor
            .categories
            .iter()
            .position(|c| c == &launcher.category.name());
        editor.is_installed = true;

        editor.app_user_agent = match &launcher.browser.user_agent {
            Some(webapps::browser::UserAgent::Default) | None => 0,
            Some(webapps::browser::UserAgent::Mobile) => 1,
            Some(webapps::browser::UserAgent::Custom(_)) => 2,
        };
        editor.app_custom_ua = match &launcher.browser.user_agent {
            Some(webapps::browser::UserAgent::Custom(ua)) => ua.clone(),
            _ => String::new(),
        };

        let perms = launcher.browser.permissions.clone().unwrap_or_default();
        editor.app_allow_camera = perms.allow_camera;
        editor.app_allow_microphone = perms.allow_microphone;
        editor.app_allow_geolocation = perms.allow_geolocation;
        editor.app_allow_notifications = perms.allow_notifications;

        editor.app_url_schemes = launcher.browser.url_schemes
            .as_ref()
            .map(|schemes| schemes.join(", "))
            .unwrap_or_default();

        editor.app_content_blocking = launcher.browser.content_blocking.unwrap_or(false);
        editor.app_block_cookies = launcher.browser.block_third_party_cookies.unwrap_or(false);
        editor.app_block_webrtc = launcher.browser.block_webrtc.unwrap_or(false);
        editor.app_proxy_url = launcher.browser.proxy_url.clone().unwrap_or_default();
        editor.app_zoom_level = launcher.browser.zoom_level.unwrap_or(1.0).to_string();
        editor.app_restore_session = launcher.browser.restore_session.unwrap_or(false);
        editor.app_launch_count = launcher.browser.launch_count.unwrap_or(0);
        editor.app_last_launched = launcher.browser.last_launched;
        editor.app_minimize_to_background = launcher.browser.minimize_to_background.unwrap_or(false);
        editor.app_auto_dark_mode = launcher.browser.auto_dark_mode.unwrap_or(false);

        editor
    }

    pub fn update(&mut self, message: Message) -> Task<Action<crate::pages::Message>> {
        match message {
            Message::AppIncognito(flag) => {
                self.app_private_mode = flag;
            }
            Message::AppSimulateMobile(flag) => {
                self.app_simulate_mobile = flag;
            }
            Message::CustomCss(css) => {
                self.app_custom_css = css;
            }
            Message::CustomJs(js) => {
                self.app_custom_js = js;
            }
            Message::Category(idx) => {
                self.app_category = webapps::Category::from_index(idx as u8);
                self.category_idx = Some(idx);
            }
            Message::DownloadFavicon => {
                let url = self.app_url.clone();
                if webapps::url_valid(&url) {
                    let url2 = url.clone();
                    let favicon_task = Task::perform(
                        async move { webapps::download_favicon(&url).await },
                        |result| {
                            cosmic::Action::App(crate::pages::Message::Editor(
                                Message::FaviconResult(result),
                            ))
                        },
                    );
                    // Also fetch site title if title field is empty
                    if self.app_title.is_empty() {
                        let title_task = Task::perform(
                            async move { webapps::fetch_site_title(&url2).await },
                            |result| {
                                cosmic::Action::App(crate::pages::Message::Editor(
                                    Message::SiteTitleResult(result),
                                ))
                            },
                        );
                        return Task::batch([favicon_task, title_task]);
                    }
                    return favicon_task;
                }
            }
            Message::FaviconResult(result) => {
                if let Some(path) = result {
                    return Task::perform(
                        async move { webapps::image_handle(path).await },
                        |icon| cosmic::Action::App(crate::pages::Message::SetIcon(icon)),
                    );
                }
            }
            Message::Duplicate => {
                let mut duplicate = self.clone();
                duplicate.app_title = format!("Copy of {}", self.app_title);
                // Keep browser config but clear app_id so a new one is generated on save
                duplicate.app_browser = None;
                duplicate.is_installed = false;
                // Preserve window settings from the original app
                if let Some(browser) = &self.app_browser {
                    duplicate.app_window_decorations = browser.window_decorations.unwrap_or(true);
                    duplicate.app_private_mode = browser.private_mode.unwrap_or(false);
                    duplicate.app_simulate_mobile = browser.try_simulate_mobile.unwrap_or(false);
                    duplicate.app_custom_css = browser.custom_css.clone().unwrap_or_default();
                    duplicate.app_custom_js = browser.custom_js.clone().unwrap_or_default();
                    if let Some(ref size) = browser.window_size {
                        duplicate.app_window_width = size.0.to_string();
                        duplicate.app_window_height = size.1.to_string();
                        duplicate.app_window_size = size.clone();
                    }
                    duplicate.app_persistent = browser.profile.is_some();
                    duplicate.app_user_agent = match &browser.user_agent {
                        Some(webapps::browser::UserAgent::Default) | None => 0,
                        Some(webapps::browser::UserAgent::Mobile) => 1,
                        Some(webapps::browser::UserAgent::Custom(_)) => 2,
                    };
                    duplicate.app_custom_ua = match &browser.user_agent {
                        Some(webapps::browser::UserAgent::Custom(ua)) => ua.clone(),
                        _ => String::new(),
                    };
                    let perms = browser.permissions.clone().unwrap_or_default();
                    duplicate.app_allow_camera = perms.allow_camera;
                    duplicate.app_allow_microphone = perms.allow_microphone;
                    duplicate.app_allow_geolocation = perms.allow_geolocation;
                    duplicate.app_allow_notifications = perms.allow_notifications;
                    duplicate.app_url_schemes = browser.url_schemes
                        .as_ref()
                        .map(|schemes| schemes.join(", "))
                        .unwrap_or_default();
                    duplicate.app_content_blocking = browser.content_blocking.unwrap_or(false);
                    duplicate.app_block_cookies = browser.block_third_party_cookies.unwrap_or(false);
                    duplicate.app_block_webrtc = browser.block_webrtc.unwrap_or(false);
                    duplicate.app_proxy_url = browser.proxy_url.clone().unwrap_or_default();
                    duplicate.app_zoom_level = browser.zoom_level.unwrap_or(1.0).to_string();
                    duplicate.app_restore_session = browser.restore_session.unwrap_or(false);
                    duplicate.app_minimize_to_background = browser.minimize_to_background.unwrap_or(false);
                    duplicate.app_auto_dark_mode = browser.auto_dark_mode.unwrap_or(false);
                }
                return task::future(async move {
                    crate::pages::Message::DuplicateApp(Box::new(duplicate))
                });
            }
            Message::Done => {
                let browser = if let Some(browser) = &self.app_browser {
                    browser.clone()
                } else {
                    let app_id = format!(
                        "{}{}",
                        self.app_title.replace(' ', ""),
                        rng().random_range(1000..10000)
                    );

                    let mut browser = webapps::browser::Browser::new(&app_id, self.app_persistent);
                    browser.window_title = Some(self.app_title.clone());
                    browser.url = Some(self.app_url.clone());
                    browser.window_size = Some(self.app_window_size.clone());
                    browser.window_decorations = Some(self.app_window_decorations);
                    browser.private_mode = Some(self.app_private_mode);
                    browser.try_simulate_mobile = Some(self.app_simulate_mobile);
                    if !self.app_custom_css.is_empty() {
                        browser.custom_css = Some(self.app_custom_css.clone());
                    }
                    if !self.app_custom_js.is_empty() {
                        browser.custom_js = Some(self.app_custom_js.clone());
                    }
                    browser.user_agent = Some(match self.app_user_agent {
                        1 => webapps::browser::UserAgent::Mobile,
                        2 => webapps::browser::UserAgent::Custom(self.app_custom_ua.clone()),
                        _ => webapps::browser::UserAgent::Default,
                    });
                    browser.permissions = Some(webapps::browser::PermissionPolicy {
                        allow_camera: self.app_allow_camera,
                        allow_microphone: self.app_allow_microphone,
                        allow_geolocation: self.app_allow_geolocation,
                        allow_notifications: self.app_allow_notifications,
                    });
                    // Parse URL schemes
                    let schemes: Vec<String> = self.app_url_schemes
                        .split(',')
                        .map(|s| s.trim().to_lowercase())
                        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '+' || c == '.'))
                        .collect();
                    if !schemes.is_empty() {
                        browser.url_schemes = Some(schemes);
                    }
                    browser.content_blocking = Some(self.app_content_blocking);
                    browser.block_third_party_cookies = Some(self.app_block_cookies);
                    browser.block_webrtc = Some(self.app_block_webrtc);
                    if !self.app_proxy_url.is_empty() {
                        browser.proxy_url = Some(self.app_proxy_url.clone());
                    }
                    let zoom: f64 = self.app_zoom_level.parse().unwrap_or(1.0);
                    browser.zoom_level = Some(zoom.clamp(0.25, 5.0));
                    browser.restore_session = Some(self.app_restore_session);
                    browser.minimize_to_background = Some(self.app_minimize_to_background);
                    browser.auto_dark_mode = Some(self.app_auto_dark_mode);
                    browser
                };

                if webapps::launcher::webapplauncher_is_valid(
                    &self.app_icon,
                    &self.app_title,
                    &browser.url,
                    &self.app_category,
                ) {
                    let launcher = webapps::launcher::WebAppLauncher {
                        browser: browser.clone(),
                        name: self.app_title.clone(),
                        icon: self.app_icon.clone(),
                        category: self.app_category.clone(),
                    };

                    return task::future(async move {
                        if launcher.create().await.is_ok() {
                            crate::pages::Message::SaveLauncher(launcher)
                        } else {
                            crate::pages::Message::None
                        }
                    });
                } else {
                    return Task::none();
                }
            }
            Message::PersistentProfile(flag) => {
                self.app_persistent = flag;
            }
            Message::LaunchApp => {
                if let Some(browser) = &self.app_browser {
                    let arg_id = browser.app_id.clone();

                    return task::future(async { crate::pages::Message::Launch(arg_id) });
                }
            }
            Message::OpenIconPicker => {
                return task::future(async { pages::Message::OpenIconPicker });
            }
            Message::Title(title) => {
                self.app_title = title;
            }
            Message::Url(url) => {
                self.app_url = url;
            }
            Message::WindowDecorations(decorations) => {
                self.app_window_decorations = decorations;
            }
            Message::WindowWidth(width) => {
                self.app_window_width = filter_numeric(width);
                let parsed: f64 = self
                    .app_window_width
                    .parse()
                    .unwrap_or(webapps::DEFAULT_WINDOW_WIDTH);
                self.app_window_size.0 = parsed.clamp(200.0, 8192.0);
            }
            Message::WindowHeight(height) => {
                self.app_window_height = filter_numeric(height);
                let parsed: f64 = self
                    .app_window_height
                    .parse()
                    .unwrap_or(webapps::DEFAULT_WINDOW_HEIGHT);
                self.app_window_size.1 = parsed.clamp(200.0, 8192.0);
            }
            Message::UserAgentSelect(idx) => {
                self.app_user_agent = idx;
            }
            Message::CustomUserAgent(ua) => {
                self.app_custom_ua = ua;
            }
            Message::AllowCamera(v) => {
                self.app_allow_camera = v;
            }
            Message::AllowMicrophone(v) => {
                self.app_allow_microphone = v;
            }
            Message::AllowGeolocation(v) => {
                self.app_allow_geolocation = v;
            }
            Message::AllowNotifications(v) => {
                self.app_allow_notifications = v;
            }
            Message::ClearAppData => {
                if let Some(browser) = &self.app_browser {
                    let app_id = browser.app_id.as_ref().to_string();
                    return task::future(
                        async move { crate::pages::Message::ClearAppData(app_id) },
                    );
                }
            }
            Message::UrlSchemes(schemes) => {
                self.app_url_schemes = schemes;
            }
            Message::ContentBlocking(flag) => {
                self.app_content_blocking = flag;
            }
            Message::BlockThirdPartyCookies(flag) => {
                self.app_block_cookies = flag;
            }
            Message::BlockWebRTC(flag) => {
                self.app_block_webrtc = flag;
            }
            Message::ProxyUrl(url) => {
                self.app_proxy_url = url;
            }
            Message::ZoomLevel(level) => {
                self.app_zoom_level = filter_numeric(level);
            }
            Message::RestoreSession(flag) => {
                self.app_restore_session = flag;
            }
            Message::MinimizeToBackground(flag) => {
                self.app_minimize_to_background = flag;
            }
            Message::AutoDarkMode(flag) => {
                self.app_auto_dark_mode = flag;
            }
            Message::ToggleAdvanced(flag) => {
                self.show_advanced = flag;
            }
            Message::FetchThumbnail => {
                if !self.thumbnail_loading && webapps::url_valid(&self.app_url) {
                    self.thumbnail_loading = true;
                    let url = self.app_url.clone();
                    return Task::perform(
                        async move { webapps::download_thumbnail(&url).await },
                        |result| {
                            cosmic::Action::App(crate::pages::Message::Editor(
                                Message::ThumbnailResult(result),
                            ))
                        },
                    );
                }
            }
            Message::ThumbnailResult(result) => {
                self.thumbnail_loading = false;
                if let Some(path) = result {
                    return Task::perform(
                        async move {
                            let data = tokio::task::spawn_blocking(move || {
                                std::fs::read(&path).ok()
                            })
                            .await
                            .ok()?;
                            data.map(widget::image::Handle::from_bytes)
                        },
                        |handle| {
                            cosmic::Action::App(crate::pages::Message::Editor(
                                Message::ThumbnailLoaded(handle),
                            ))
                        },
                    );
                }
            }
            Message::ThumbnailLoaded(handle) => {
                self.thumbnail_handle = handle;
            }
            Message::SiteTitleResult(result) => {
                // Only auto-fill if the title is still empty (user hasn't typed anything)
                if let Some(title) = result {
                    if self.app_title.is_empty() {
                        self.app_title = title;
                    }
                }
            }
        }
        Task::none()
    }

    pub fn update_icon(&mut self, icon: Option<webapps::Icon>) {
        if let Some(icon) = icon {
            self.app_icon = icon.path.clone();
            self.selected_icon = Some(icon);
        }
    }

    fn icon_element(&self, icon: Option<webapps::Icon>) -> Element<'_, Message> {
        let ico = if let Some(ico) = icon {
            match ico.icon {
                webapps::IconType::Raster(data) => widget::button::custom(widget::image(data))
                    .width(Length::Fixed(92.0))
                    .height(Length::Fixed(92.0))
                    .class(style::Button::Icon)
                    .on_press(Message::OpenIconPicker),

                webapps::IconType::Svg(data) => widget::button::custom(widget::svg(data))
                    .width(Length::Fixed(92.0))
                    .height(Length::Fixed(92.0))
                    .class(style::Button::Icon)
                    .on_press(Message::OpenIconPicker),
            }
        } else {
            widget::button::custom(widget::icon::from_name("folder-pictures-symbolic"))
                .width(Length::Fixed(92.0))
                .height(Length::Fixed(92.0))
                .class(style::Button::Suggested)
                .on_press(Message::OpenIconPicker)
        };

        widget::tooltip(
            widget::container(ico),
            widget::text(fl!("icon-selector")),
            widget::tooltip::Position::Bottom,
        )
        .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        widget::container(
            widget::column()
                .spacing(24)
                .push(
                    widget::container(
                        widget::row()
                            .spacing(12)
                            .push(
                                widget::container(self.icon_element(self.selected_icon.clone()))
                                    .width(96.)
                                    .height(96.)
                                    .align_y(Vertical::Center),
                            )
                            .push(
                                widget::container(
                                    widget::column()
                                        .spacing(12)
                                        .push(widget::text::title3(format!(
                                            "{}: {}",
                                            fl!("title"),
                                            if self.app_title.is_empty() {
                                                fl!("new-webapp-title")
                                            } else {
                                                self.app_title.clone()
                                            }
                                        )))
                                        .push(widget::text::title4(format!(
                                            "{}: {}",
                                            fl!("category"),
                                            self.app_category.name()
                                        ))),
                                )
                                .height(Length::Fixed(96.))
                                .align_y(Vertical::Center),
                            ),
                    )
                    .padding(12)
                    .width(Length::Fill)
                    .class(style::Container::Card),
                )
                // Thumbnail preview
                .push_maybe(if let Some(handle) = &self.thumbnail_handle {
                    Some(
                        widget::container(
                            widget::image(handle.clone())
                                .width(Length::Fill)
                                .height(Length::Fixed(200.0)),
                        )
                        .width(Length::Fill)
                        .class(cosmic::style::Container::Card),
                    )
                } else if self.thumbnail_loading {
                    Some(
                        widget::container(
                            widget::text::body(fl!("loading")),
                        )
                        .width(Length::Fill)
                        .padding(12)
                        .class(cosmic::style::Container::Card),
                    )
                } else if self.is_installed && webapps::url_valid(&self.app_url) {
                    Some(
                        widget::container(
                            widget::button::standard(fl!("fetch-thumbnail"))
                                .on_press(Message::FetchThumbnail),
                        )
                        .width(Length::Fill)
                        .padding(12),
                    )
                } else {
                    None
                })
                .push(widget::text_input(fl!("title"), &self.app_title).on_input(Message::Title))
                .push_maybe(if !self.app_title.is_empty() && self.app_title.len() < 3 {
                    Some(widget::text::caption(fl!("warning-app-name")).class(style::Text::Accent))
                } else {
                    None
                })
                .push(
                    widget::row()
                        .spacing(8)
                        .push(widget::text_input(fl!("url"), &self.app_url).on_input(Message::Url))
                        .push(
                            widget::button::standard(fl!("download-favicon")).on_press_maybe(
                                if webapps::url_valid(&self.app_url) {
                                    Some(Message::DownloadFavicon)
                                } else {
                                    None
                                },
                            ),
                        ),
                )
                .push_maybe(
                    if !self.app_url.is_empty() && !webapps::url_valid(&self.app_url) {
                        Some(
                            widget::text::caption(fl!("warning-app-url"))
                                .class(style::Text::Accent),
                        )
                    } else {
                        None
                    },
                )
                // Basic settings section
                .push(
                    widget::settings::section()
                        .title(fl!("basic-settings"))
                        .add(widget::settings::item(
                            fl!("select-category"),
                            widget::dropdown(
                                &self.categories,
                                self.category_idx,
                                Message::Category,
                            ),
                        ))
                        .add(widget::settings::item(
                            fl!("persistent-profile"),
                            widget::toggler(self.app_persistent)
                                .on_toggle(Message::PersistentProfile),
                        ))
                        .add(widget::settings::item(
                            fl!("window-size"),
                            widget::row()
                                .spacing(8)
                                .push(
                                    widget::text_input(
                                        format!("{}", webapps::DEFAULT_WINDOW_WIDTH),
                                        &self.app_window_width,
                                    )
                                    .on_input(Message::WindowWidth),
                                )
                                .push(
                                    widget::text_input(
                                        format!("{}", webapps::DEFAULT_WINDOW_HEIGHT),
                                        &self.app_window_height,
                                    )
                                    .on_input(Message::WindowHeight),
                                ),
                        ))
                        .add(widget::settings::item(
                            fl!("decorations"),
                            widget::toggler(self.app_window_decorations)
                                .on_toggle(Message::WindowDecorations),
                        )),
                )
                // Advanced settings toggle
                .push(
                    widget::settings::item(
                        fl!("advanced-settings"),
                        widget::toggler(self.show_advanced)
                            .on_toggle(Message::ToggleAdvanced),
                    ),
                )
                // Advanced settings section (conditional)
                .push_maybe(if self.show_advanced {
                    let mut advanced = widget::settings::section()
                        .title(fl!("advanced-settings"))
                        .add(widget::settings::item(
                            fl!("private-mode"),
                            widget::toggler(self.app_private_mode).on_toggle(Message::AppIncognito),
                        ))
                        .add(widget::settings::item(
                            fl!("simulate-mobile"),
                            widget::toggler(self.app_simulate_mobile)
                                .on_toggle(Message::AppSimulateMobile),
                        ))
                        .add(widget::settings::item(
                            fl!("user-agent"),
                            widget::dropdown(
                                &self.user_agent_options,
                                Some(self.app_user_agent),
                                Message::UserAgentSelect,
                            ),
                        ));

                    if self.app_user_agent == 2 {
                        advanced = advanced.add(widget::settings::item(
                            fl!("user-agent-custom-label"),
                            widget::text_input(
                                fl!("user-agent-custom-placeholder"),
                                &self.app_custom_ua,
                            )
                            .on_input(Message::CustomUserAgent),
                        ));
                    }

                    advanced = advanced
                        .add(widget::settings::item(
                            fl!("permission-camera"),
                            widget::toggler(self.app_allow_camera).on_toggle(Message::AllowCamera),
                        ))
                        .add(widget::settings::item(
                            fl!("permission-microphone"),
                            widget::toggler(self.app_allow_microphone)
                                .on_toggle(Message::AllowMicrophone),
                        ))
                        .add(widget::settings::item(
                            fl!("permission-geolocation"),
                            widget::toggler(self.app_allow_geolocation)
                                .on_toggle(Message::AllowGeolocation),
                        ))
                        .add(widget::settings::item(
                            fl!("permission-notifications"),
                            widget::toggler(self.app_allow_notifications)
                                .on_toggle(Message::AllowNotifications),
                        ))
                        .add(widget::settings::item(
                            fl!("custom-css"),
                            widget::text_input(fl!("custom-css-placeholder"), &self.app_custom_css)
                                .on_input(Message::CustomCss),
                        ))
                        .add(widget::settings::item(
                            fl!("custom-js"),
                            widget::column()
                                .spacing(4)
                                .push(
                                    widget::text_input(
                                        fl!("custom-js-placeholder"),
                                        &self.app_custom_js,
                                    )
                                    .on_input(Message::CustomJs),
                                )
                                .push(
                                    widget::text::caption(fl!("custom-js-warning"))
                                        .class(style::Text::Accent),
                                ),
                        ))
                        .add(widget::settings::item(
                            fl!("url-schemes"),
                            widget::text_input(
                                fl!("url-schemes-placeholder"),
                                &self.app_url_schemes,
                            )
                            .on_input(Message::UrlSchemes),
                        ))
                        .add(widget::settings::item(
                            fl!("content-blocking"),
                            widget::toggler(self.app_content_blocking)
                                .on_toggle(Message::ContentBlocking),
                        ))
                        .add(widget::settings::item(
                            fl!("block-third-party-cookies"),
                            widget::toggler(self.app_block_cookies)
                                .on_toggle(Message::BlockThirdPartyCookies),
                        ))
                        .add(widget::settings::item(
                            fl!("block-webrtc"),
                            widget::toggler(self.app_block_webrtc)
                                .on_toggle(Message::BlockWebRTC),
                        ))
                        .add(widget::settings::item(
                            fl!("proxy-url"),
                            widget::text_input(
                                fl!("proxy-url-placeholder"),
                                &self.app_proxy_url,
                            )
                            .on_input(Message::ProxyUrl),
                        ))
                        .add(widget::settings::item(
                            fl!("zoom-level"),
                            widget::text_input(
                                fl!("zoom-level-placeholder"),
                                &self.app_zoom_level,
                            )
                            .on_input(Message::ZoomLevel),
                        ))
                        .add(widget::settings::item(
                            fl!("restore-session"),
                            widget::toggler(self.app_restore_session)
                                .on_toggle(Message::RestoreSession),
                        ))
                        .add(widget::settings::item(
                            fl!("minimize-to-background"),
                            widget::toggler(self.app_minimize_to_background)
                                .on_toggle(Message::MinimizeToBackground),
                        ))
                        .add(widget::settings::item(
                            fl!("auto-dark-mode"),
                            widget::toggler(self.app_auto_dark_mode)
                                .on_toggle(Message::AutoDarkMode),
                        ));

                    // Show usage stats for installed apps (read-only)
                    if self.is_installed {
                        advanced = advanced
                            .add(widget::settings::item(
                                fl!("launch-count"),
                                widget::text::body(format!("{}", self.app_launch_count)),
                            ))
                            .add(widget::settings::item(
                                fl!("last-launched"),
                                widget::text::body(
                                    self.app_last_launched
                                        .map(webapps::format_timestamp)
                                        .unwrap_or_else(|| fl!("never-launched")),
                                ),
                            ));
                    }

                    Some(advanced)
                } else {
                    None
                })
                .push(
                    widget::row()
                        .spacing(8)
                        .push(widget::horizontal_space())
                        .push_maybe(if self.is_installed && self.app_persistent {
                            Some(
                                widget::button::destructive(fl!("clear-data"))
                                    .on_press(Message::ClearAppData),
                            )
                        } else {
                            None
                        })
                        .push_maybe(if !self.is_installed {
                            None
                        } else {
                            Some(
                                widget::button::standard(fl!("duplicate"))
                                    .on_press(Message::Duplicate),
                            )
                        })
                        .push_maybe(if !self.is_installed {
                            None
                        } else {
                            Some(
                                widget::button::standard(fl!("run-app"))
                                    .on_press(Message::LaunchApp),
                            )
                        })
                        .push(widget::button::suggested(fl!("create")).on_press_maybe(
                            if webapps::launcher::webapplauncher_is_valid(
                                &self.app_icon,
                                &self.app_title,
                                &Some(self.app_url.clone()),
                                &self.app_category,
                            ) {
                                Some(Message::Done)
                            } else {
                                None
                            },
                        )),
                ),
        )
        .padding(cosmic::iced::Padding::new(0.).left(30.0).right(30.0))
        .max_width(1000)
        .into()
    }
}
