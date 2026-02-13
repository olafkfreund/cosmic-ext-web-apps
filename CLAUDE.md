# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Quick Web Apps** (`dev.heppen.webapps`) is a web app manager for the COSMIC desktop environment. Users create, manage, and launch web applications that run in isolated WebKitGTK webview windows. Built with Rust using `libcosmic` (iced-based GUI framework) and `wry`/`tao` for the webview runtime.

## Build Commands

The project uses `just` as the task runner:

```bash
just                    # Build release (default)
just build-debug        # Debug build
just build-release      # Release build
just check              # Clippy with -W clippy::pedantic
just run                # Run with RUST_BACKTRACE=full
just dev                # cargo fmt + run
just clean              # cargo clean
sudo just install       # Install binaries, desktop entry, icons to /app/
```

Flatpak build (primary distribution method):
```bash
flatpak run --command=flathub-build org.flatpak.Builder --install dev.heppen.webapps.json
```

There is no test suite. A `flake.nix` is provided for reproducible Nix builds (`nix build`), development shells (`nix develop`), and CI checks (`nix flake check`). Alternatively, use the Containerfile in `.devcontainer/` or system-installed deps.

### System Dependencies

Building requires: `pkg-config`, `libssl-dev`, `libxkbcommon-dev`, `libwebkit2gtk-4.1-dev`, `just`

## Architecture

### Two Binaries

The crate produces two binaries (see `Cargo.toml [[bin]]` sections):

1. **`dev-heppen-webapps`** (`src/bin/dev-heppen-webapps/main.rs`) — The main GUI application where users create/edit/delete web apps
2. **`dev-heppen-webapps-webview`** (`src/bin/webview.rs`) — Lightweight webview process spawned per web app. Reads config from the database, creates a GTK+WebKitGTK window, and runs an event loop

Each web app runs as a separate `dev-heppen-webapps-webview <app_id>` process.

### Library Layer (`src/`)

- **`lib.rs`** — Core types (`Icon`, `IconType`, `Category`, `WindowSize`, `WebviewArgs`), XDG path helpers (`database_path()`, `profiles_path()`, `icons_location()`), icon search/validation, URL validation
- **`browser.rs`** — `Browser` struct: app configuration (URL, title, profile path, window size, decorations, private mode, mobile simulation). Serialized to/from RON
- **`launcher.rs`** — `WebAppLauncher` struct: wraps `Browser` + name/icon/category. Uses `ashpd` (XDG Desktop Portal) `DynamicLauncher` to create/delete `.desktop` entries. Stores webapp data as `.ron` files in the database directory
- **`localize.rs`** — i18n via `i18n-embed` with Fluent. Uses `fl!()` macro. Translation files: `i18n/{lang}/webapps.ftl`

### GUI Application (`src/bin/dev-heppen-webapps/`)

- **`pages/mod.rs`** — `QuickWebApps`: the `cosmic::Application` implementation. Manages nav bar (installed apps list), dialogs (icon picker, delete confirmation, icon downloader), theme system, and config subscription
- **`pages/editor.rs`** — `AppEditor`: form for creating/editing a web app (title, URL, icon, category, window size, toggles for persistent profile/decorations/private mode/mobile simulation)
- **`pages/iconpicker.rs`** — `IconPicker`: modal dialog for searching system icon packs (Papirus) or picking custom files
- **`config.rs`** — `AppConfig` with CosmicConfig integration (persists theme choice)
- **`themes.rs`** — Light/Dark built-in themes + custom RON theme import

### Key Data Flow

**Creating a web app:**
1. User fills `AppEditor` form → generates unique `app_id` (title + random 4-digit suffix)
2. `WebAppLauncher::create()` calls XDG DynamicLauncher portal to install a `.desktop` entry
3. Launcher config saved as RON to `$XDG_DATA_HOME/dev.heppen.webapps/database/{app_id}.ron`

**Launching a web app:**
1. Desktop entry runs `dev.heppen.webapps.webview {app_id}`
2. Webview binary loads `Browser::from_appid()` from the RON database
3. Creates GTK window with WebKitGTK webview using stored settings

### Data Storage (all XDG-compliant)

| Path | Content |
|------|---------|
| `$XDG_DATA_HOME/dev.heppen.webapps/database/*.ron` | Webapp configs (RON format) |
| `$XDG_DATA_HOME/dev.heppen.webapps/profiles/{app_id}/` | Per-app WebKitGTK browser data |
| `$XDG_DATA_HOME/dev.heppen.webapps/icons/` | Cached icons |
| `$XDG_DATA_HOME/dev.heppen.webapps/themes/` | Custom theme RON files |
| `$XDG_CONFIG_HOME/cosmic/{version}/dev.heppen.webapps.ron` | App config (via CosmicConfig) |

## i18n

14 languages supported. English source: `i18n/en/webapps.ftl`. Add translations by creating `i18n/{lang_code}/webapps.ftl`. Strings are accessed via `fl!("key")` or `fl!("key", arg = value)`.

## App ID

The app ID `dev.heppen.webapps` is used throughout: Flatpak manifest, desktop entry, config paths, binary naming. It is defined as `APPID` in the justfile and `APP_ID` constant in `src/lib.rs`.

## Key Dependencies

- **libcosmic** (git dep from pop-os/libcosmic) — COSMIC app framework, provides `cosmic::Application` trait, widgets, nav bar, config system, theme engine
- **wry** + **tao** + **gtk** — WebKitGTK webview creation and window management (webview binary only)
- **ashpd** — XDG Desktop Portal client for DynamicLauncher (creating/removing `.desktop` entries)
- **ron** — Rusty Object Notation for config serialization
- **i18n-embed** + **i18n-embed-fl** — Compile-time embedded Fluent translations
