use clap::Parser;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    platform::unix::EventLoopBuilderExtUnix,
    window::{WindowAttributes, WindowBuilder},
};
use url::Url;
use wry::{
    dpi::{LogicalSize, Size},
    WebContext, WebViewBuilder,
};

fn is_url_safe(url_str: &str) -> bool {
    match Url::parse(url_str) {
        Ok(url) => matches!(url.scheme(), "http" | "https"),
        Err(_) => false,
    }
}

fn main() -> wry::Result<()> {
    let args = webapps::WebviewArgs::parse();

    if let Err(e) = gtk::init() {
        eprintln!("Failed to initialize GTK: {e}");
        std::process::exit(1);
    }

    gtk::glib::set_program_name(args.id.clone().into());
    gtk::glib::set_application_name(&args.id);

    let browser = match webapps::browser::Browser::from_appid(&args.id) {
        Some(b) => b,
        None => {
            eprintln!("Failed to load web app configuration for '{}'", args.id);
            std::process::exit(1);
        }
    };

    // Validate URL scheme before loading
    let url = browser.url.unwrap_or_default();
    if !url.is_empty() && !is_url_safe(&url) {
        eprintln!("Refusing to load unsafe URL scheme: {url}");
        std::process::exit(1);
    }

    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

    // Clone title before window builder consumes it (needed for notification forwarding)
    let app_title_for_notifications = browser.window_title.clone().unwrap_or_else(|| "Web App".to_string());

    let mut attrs = WindowAttributes::default();
    if let Some(size) = browser.window_size {
        attrs.inner_size = Some(Size::new(LogicalSize::new(size.0, size.1)));
    }

    let mut window_builder = WindowBuilder::new();
    window_builder.window = attrs;

    let window = match window_builder
        .with_title(browser.window_title.unwrap_or(webapps::fl!("app")))
        .with_decorations(browser.window_decorations.unwrap_or(true))
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to create window: {e}");
            std::process::exit(1);
        }
    };

    let mut context = WebContext::new(browser.profile);

    let mut builder = WebViewBuilder::new_with_web_context(&mut context)
        .with_url(&url)
        .with_incognito(browser.private_mode.unwrap_or(false))
        .with_devtools(false)
        .with_navigation_handler(|nav_url| {
            if is_url_safe(&nav_url) {
                true
            } else {
                eprintln!("Blocked navigation to unsafe URL: {nav_url}");
                false
            }
        })
        .with_new_window_req_handler(|new_url, _features| {
            if is_url_safe(&new_url) {
                wry::NewWindowResponse::Allow
            } else {
                eprintln!("Blocked new window with unsafe URL: {new_url}");
                wry::NewWindowResponse::Deny
            }
        })
        .with_download_started_handler(|url, dest_path| {
            if !is_url_safe(&url) {
                eprintln!("Blocked download from unsafe URL: {url}");
                return false;
            }
            // Redirect downloads to XDG download directory
            if let Some(download_dir) = dirs::download_dir() {
                match dest_path.file_name() {
                    Some(filename) => *dest_path = download_dir.join(filename),
                    None => {
                        eprintln!("Blocked download with no valid filename");
                        return false;
                    }
                }
            }
            true
        });

    // Issue #38: Apply user agent (try_simulate_mobile takes precedence for backwards compat)
    if let Some(true) = browser.try_simulate_mobile {
        builder = builder.with_user_agent(webapps::MOBILE_UA);
    } else if let Some(ref ua) = browser.user_agent {
        match ua {
            webapps::browser::UserAgent::Default => {}
            webapps::browser::UserAgent::Mobile => {
                builder = builder.with_user_agent(webapps::MOBILE_UA);
            }
            webapps::browser::UserAgent::Custom(custom_ua) => {
                if !custom_ua.trim().is_empty() {
                    builder = builder.with_user_agent(custom_ua);
                }
            }
        }
    }

    // Issue #35: Enforce permission policies via JavaScript injection
    let perms = browser.permissions.clone().unwrap_or_default();
    let mut permission_overrides = Vec::new();

    if !perms.allow_camera || !perms.allow_microphone {
        // Override getUserMedia to block camera/mic
        let block_video = if !perms.allow_camera { "true" } else { "false" };
        let block_audio = if !perms.allow_microphone { "true" } else { "false" };
        permission_overrides.push(format!(
            r#"(function(){{
                var origGetUserMedia = navigator.mediaDevices && navigator.mediaDevices.getUserMedia;
                if (origGetUserMedia) {{
                    navigator.mediaDevices.getUserMedia = function(constraints) {{
                        if ({block_video} && constraints && constraints.video) {{
                            return Promise.reject(new DOMException('Camera access denied by app settings', 'NotAllowedError'));
                        }}
                        if ({block_audio} && constraints && constraints.audio) {{
                            return Promise.reject(new DOMException('Microphone access denied by app settings', 'NotAllowedError'));
                        }}
                        return origGetUserMedia.call(navigator.mediaDevices, constraints);
                    }};
                }}
            }})()"#
        ));
    }

    if !perms.allow_geolocation {
        permission_overrides.push(
            r#"(function(){
                navigator.geolocation.getCurrentPosition = function(s, e) {
                    if (e) e({ code: 1, message: 'Geolocation denied by app settings', PERMISSION_DENIED: 1 });
                };
                navigator.geolocation.watchPosition = function(s, e) {
                    if (e) e({ code: 1, message: 'Geolocation denied by app settings', PERMISSION_DENIED: 1 });
                    return 0;
                };
            })()"#.to_string()
        );
    }

    if !perms.allow_notifications {
        permission_overrides.push(
            r#"(function(){
                window.Notification = class {
                    constructor() { throw new DOMException('Notifications denied by app settings', 'NotAllowedError'); }
                    static get permission() { return 'denied'; }
                    static requestPermission() { return Promise.resolve('denied'); }
                };
            })()"#.to_string()
        );
    }

    for script in &permission_overrides {
        builder = builder.with_initialization_script(script);
    }

    // Issue #39: Forward web notifications to COSMIC desktop notifications
    if perms.allow_notifications {
        builder = builder.with_initialization_script(
            r#"(function(){
                window.Notification = class extends EventTarget {
                    constructor(title, options) {
                        super();
                        window.ipc.postMessage(JSON.stringify({
                            type: 'notification',
                            title: title || '',
                            body: (options && options.body) || ''
                        }));
                    }
                    static get permission() { return 'granted'; }
                    static requestPermission() { return Promise.resolve('granted'); }
                };
            })()"#
        );

        let app_title = app_title_for_notifications.clone();
        builder = builder.with_ipc_handler(move |req| {
            let msg = req.body();
            // Parse JSON to forward notifications
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(msg) {
                if parsed.get("type").and_then(|t| t.as_str()) == Some("notification") {
                    let title = parsed.get("title").and_then(|t| t.as_str()).unwrap_or("Notification");
                    let body = parsed.get("body").and_then(|b| b.as_str()).unwrap_or("");
                    let _ = notify_rust::Notification::new()
                        .summary(&format!("{} — {}", app_title, title))
                        .body(body)
                        .appname("dev.heppen.webapps")
                        .show();
                }
            }
        });
    }

    // Inject custom CSS if configured
    if let Some(ref css) = browser.custom_css {
        if !css.trim().is_empty() {
            let css_escaped = css.replace('\\', "\\\\").replace('`', "\\`");
            builder = builder.with_initialization_script(&format!(
                "(function(){{var s=document.createElement('style');s.textContent=`{css_escaped}`;document.head.appendChild(s)}})()"
            ));
        }
    }

    // Inject custom JavaScript if configured
    if let Some(ref js) = browser.custom_js {
        if !js.trim().is_empty() {
            builder = builder.with_initialization_script(js);
        }
    }

    let _webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = match window.default_vbox() {
            Some(vbox) => vbox,
            None => {
                eprintln!("Failed to get GTK vbox from window");
                std::process::exit(1);
            }
        };
        builder.build_gtk(vbox)?
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
