use clap::Parser;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    platform::unix::EventLoopBuilderExtUnix,
    window::{WindowAttributes, WindowBuilder},
};
use url::Url;
use wry::{
    WebContext, WebViewBuilder,
    dpi::{LogicalSize, Size},
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

    let mut browser = match webapps::browser::Browser::from_appid(&args.id) {
        Some(b) => b,
        None => {
            eprintln!("Failed to load web app configuration for '{}'", args.id);
            std::process::exit(1);
        }
    };

    // Override private mode if --private CLI flag was passed
    if args.private {
        browser.private_mode = Some(true);
    }

    // Validate URL scheme before loading
    let url = browser.url.unwrap_or_default();
    if !url.is_empty() && !is_url_safe(&url) {
        eprintln!("Refusing to load unsafe URL scheme: {url}");
        std::process::exit(1);
    }

    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

    // Clone title before window builder consumes it (needed for notification forwarding)
    let app_title_for_notifications = browser
        .window_title
        .clone()
        .unwrap_or_else(|| "Web App".to_string());

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

    // Issue #46: WM_CLASS is set via gtk::glib::set_program_name() above (line 29),
    // which GTK uses as the WM_CLASS res_name on X11. This matches StartupWMClass
    // in the generated .desktop entry.

    // #54: Set proxy environment variables if configured
    if let Some(ref proxy) = browser.proxy_url {
        if !proxy.trim().is_empty() {
            // SAFETY: Called before any threads are spawned
            unsafe {
                std::env::set_var("http_proxy", proxy);
                std::env::set_var("https_proxy", proxy);
            }
        }
    }

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
        let block_audio = if !perms.allow_microphone {
            "true"
        } else {
            "false"
        };
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

    // #53: Content blocking (ads/trackers)
    if let Some(true) = browser.content_blocking {
        builder = builder.with_initialization_script(
            r#"(function(){
                var adSelectors = [
                    'iframe[src*="ads"]', 'iframe[src*="doubleclick"]',
                    'div[class*="ad-"]', 'div[class*="advert"]',
                    'div[id*="google_ads"]', 'ins.adsbygoogle',
                    '[data-ad]', '[data-ads]', '[data-ad-slot]'
                ];
                function removeAds() {
                    adSelectors.forEach(function(sel) {
                        document.querySelectorAll(sel).forEach(function(el) { el.remove(); });
                    });
                }
                removeAds();
                new MutationObserver(removeAds).observe(
                    document.body || document.documentElement,
                    { childList: true, subtree: true }
                );
            })()"#,
        );
    }

    // #60: Block third-party cookies
    if let Some(true) = browser.block_third_party_cookies {
        builder = builder.with_initialization_script(
            r#"(function(){
                try {
                    Object.defineProperty(document, 'cookie', {
                        get: function() {
                            return document._firstPartyCookies || '';
                        },
                        set: function(val) {
                            // Only allow first-party cookie setting
                            if (!val.includes('domain=') || val.includes(window.location.hostname)) {
                                document._firstPartyCookies = val;
                            }
                        }
                    });
                } catch(e) {}
            })()"#,
        );
    }

    // #61: Block WebRTC IP leak
    if let Some(true) = browser.block_webrtc {
        builder = builder.with_initialization_script(
            r#"(function(){
                window.RTCPeerConnection = undefined;
                window.webkitRTCPeerConnection = undefined;
                window.mozRTCPeerConnection = undefined;
                if (navigator.mediaDevices) {
                    navigator.mediaDevices.enumerateDevices = function() {
                        return Promise.resolve([]);
                    };
                }
            })()"#,
        );
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
            })()"#,
        );
    }

    // Issue #43: Media session integration (always inject)
    builder = builder.with_initialization_script(
        r#"(function(){
            // Auto-wire media session to first video/audio element
            function wireMediaSession() {
                var media = document.querySelector('video, audio');
                if (!media) return;

                if ('mediaSession' in navigator) {
                    navigator.mediaSession.setActionHandler('play', function() { media.play(); });
                    navigator.mediaSession.setActionHandler('pause', function() { media.pause(); });
                    navigator.mediaSession.setActionHandler('seekbackward', function() { media.currentTime = Math.max(0, media.currentTime - 10); });
                    navigator.mediaSession.setActionHandler('seekforward', function() { media.currentTime += 10; });

                    // Report media state changes via IPC
                    media.addEventListener('play', function() {
                        window.ipc.postMessage(JSON.stringify({type:'media', state:'playing'}));
                    });
                    media.addEventListener('pause', function() {
                        window.ipc.postMessage(JSON.stringify({type:'media', state:'paused'}));
                    });
                }
            }

            // Wire up on load and on DOM changes
            wireMediaSession();
            var observer = new MutationObserver(function() { wireMediaSession(); });
            observer.observe(document.body || document.documentElement, { childList: true, subtree: true });
        })()"#,
    );

    // Issue #44: Badge count detection (always inject)
    builder = builder.with_initialization_script(
        r#"(function(){
            var lastBadge = 0;
            function checkBadge() {
                var match = document.title.match(/[\(\[](\d+)[\)\]]/);
                var count = match ? parseInt(match[1]) : 0;
                if (count !== lastBadge) {
                    lastBadge = count;
                    window.ipc.postMessage(JSON.stringify({type:'badge', count: count}));
                }
            }

            // Also intercept Badging API if available
            if (navigator.setAppBadge) {
                var origSetBadge = navigator.setAppBadge.bind(navigator);
                navigator.setAppBadge = function(count) {
                    window.ipc.postMessage(JSON.stringify({type:'badge', count: count || 0}));
                    return origSetBadge(count);
                };
            }
            if (navigator.clearAppBadge) {
                var origClearBadge = navigator.clearAppBadge.bind(navigator);
                navigator.clearAppBadge = function() {
                    window.ipc.postMessage(JSON.stringify({type:'badge', count: 0}));
                    return origClearBadge();
                };
            }

            // Check periodically and on title changes
            checkBadge();
            var titleEl = document.querySelector('title');
            if (titleEl) {
                new MutationObserver(checkBadge).observe(titleEl, { childList: true });
            }
            setInterval(checkBadge, 5000);
        })()"#,
    );

    // Always set up IPC handler for media controls, badges, session URL, and optionally notifications
    let forward_notifications = perms.allow_notifications;
    let app_title = app_title_for_notifications.clone();
    let restore_session_enabled = browser.restore_session.unwrap_or(false);
    let ipc_app_id = browser.app_id.as_ref().to_string();
    builder = builder.with_ipc_handler(move |req| {
        let msg = req.body();
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(msg) {
            match parsed.get("type").and_then(|t| t.as_str()) {
                Some("notification") if forward_notifications => {
                    let title = parsed
                        .get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("Notification");
                    let body = parsed.get("body").and_then(|b| b.as_str()).unwrap_or("");
                    let _ = notify_rust::Notification::new()
                        .summary(&format!("{} — {}", app_title, title))
                        .body(body)
                        .appname("dev.heppen.webapps")
                        .show();
                }
                Some("media") => {
                    if let Some(state) = parsed.get("state").and_then(|s| s.as_str()) {
                        tracing::debug!("Media state: {state}");
                    }
                }
                Some("badge") => {
                    if let Some(count) = parsed.get("count").and_then(|c| c.as_u64()) {
                        tracing::debug!("Badge count: {count}");
                    }
                }
                Some("save_url") if restore_session_enabled => {
                    if let Some(new_url) = parsed.get("url").and_then(|u| u.as_str()) {
                        if !new_url.is_empty() {
                            // Update last_url in the RON database
                            let safe_id = webapps::browser::sanitize_app_id(&ipc_app_id);
                            if let Some(db_path) = webapps::database_path(&format!("{safe_id}.ron")) {
                                if let Ok(content) = std::fs::read_to_string(&db_path) {
                                    if let Ok(mut launcher) = ron::from_str::<webapps::launcher::WebAppLauncher>(&content) {
                                        launcher.browser.last_url = Some(new_url.to_string());
                                        if let Ok(serialized) = ron::ser::to_string_pretty(&launcher, ron::ser::PrettyConfig::default()) {
                                            let _ = std::fs::write(&db_path, serialized);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

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

    // #55: Zoom level via CSS transform
    if let Some(zoom) = browser.zoom_level {
        if (zoom - 1.0).abs() > f64::EPSILON {
            let zoom_clamped = zoom.clamp(0.25, 5.0);
            builder = builder.with_initialization_script(&format!(
                "(function(){{document.body.style.zoom='{}';}})()",
                zoom_clamped
            ));
        }
    }

    // #56: Session restore — navigate to last URL if enabled
    if let Some(true) = browser.restore_session {
        if let Some(ref last) = browser.last_url {
            if !last.is_empty() && is_url_safe(last) && last != &url {
                builder = builder.with_url(last);
            }
        }
    }

    // #56: Session URL saving — periodically report current URL via IPC
    if let Some(true) = browser.restore_session {
        builder = builder.with_initialization_script(
            r#"(function(){
                setInterval(function() {
                    window.ipc.postMessage(JSON.stringify({
                        type: 'save_url',
                        url: window.location.href
                    }));
                }, 30000);
                // Also save on page unload
                window.addEventListener('beforeunload', function() {
                    window.ipc.postMessage(JSON.stringify({
                        type: 'save_url',
                        url: window.location.href
                    }));
                });
            })()"#,
        );
    }

    // #62: Auto dark mode CSS injection based on system preference
    if let Some(true) = browser.auto_dark_mode {
        builder = builder.with_initialization_script(
            r#"(function(){
                var style = document.createElement('style');
                style.textContent = '@media (prefers-color-scheme: dark) { html { filter: invert(1) hue-rotate(180deg); } img, video, canvas, svg { filter: invert(1) hue-rotate(180deg); } }';
                document.head.appendChild(style);
                // Also try to set color-scheme meta
                var meta = document.querySelector('meta[name="color-scheme"]');
                if (!meta) {
                    meta = document.createElement('meta');
                    meta.name = 'color-scheme';
                    document.head.appendChild(meta);
                }
                meta.content = 'dark light';
            })()"#,
        );
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

    // #59: Minimize to background on close
    let minimize_on_close = browser.minimize_to_background.unwrap_or(false);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            if minimize_on_close {
                window.set_visible(false);
            } else {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}
