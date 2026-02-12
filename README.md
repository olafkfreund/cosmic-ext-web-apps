<!--suppress HtmlDeprecatedAttribute -->
<div align="center">
  <br>
  <img alt="Quick Web Apps" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/master/resources/icons/hicolor/256x256/apps/dev.heppen.webapps.png" width="192" />
  <h1>Quick Web Apps</h1>

  <p>Web App Manager for the COSMIC™ desktop written with love and libcosmic. Create web applications from any URL, running in their own dedicated window with WebKitGTK rendering.</p>

  <br>

  <img alt="Quick Web Apps" src="https://raw.githubusercontent.com/cosmic-utils/web-apps/refs/heads/master/resources/screenshots/window.png" width="512">

<br><br><br>

  <a href='https://flathub.org/apps/dev.heppen.webapps'>
    <img width='240' alt='Download on Flathub' src='https://flathub.org/api/badge?locale=en'/>
  </a>
</div>

## Features

- Create web apps from any URL with a dedicated window and desktop entry
- **Favicon auto-detection** from URL using Google S2 Favicons API
- **Search and filter** installed apps from the header bar
- **Apps organized by category** then sorted alphabetically in the nav bar
- **Import/Export** web app configurations as RON files for backup and sharing
- **App duplication** to quickly clone an existing web app with all settings
- **First-run onboarding** with an empty state guide for new users
- **Keyboard shortcuts**: Ctrl+N (new app), Ctrl+S (save)
- **Desktop actions**: "New Window" action in generated `.desktop` files
- Icon picker with system icon search and Papirus icon pack support
- Per-app settings: custom window size, decorations, private mode, mobile UA simulation
- Persistent browser profiles with isolated data directories
- **Toast notifications** for save/delete feedback
- **14 languages**: English, Bulgarian, Czech, Esperanto, Spanish, French, Italian, Dutch, Polish, Portuguese (Brazil), Serbian, Swedish, Turkish, Ukrainian
- Secure: URL scheme validation, desktop entry injection prevention, path traversal protection, import sanitization

## Support

This app is fully distributed for **free** under the **GPL-3.0 license**.
Developed with passion in free time. If you find it useful, consider supporting the project!

## Installation

### Flatpak (recommended)

Clone the repository:

```bash
git clone https://github.com/cosmic-utils/web-apps.git
cd web-apps
```

Make sure you have configured the `flathub` remote as `--user`:

```bash
flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
```

Install `flatpak-builder`:

```bash
flatpak install -y flathub org.flatpak.Builder
```

Build and install:

```bash
flatpak run --command=flathub-build org.flatpak.Builder --install dev.heppen.webapps.json
```

Launch:

```bash
flatpak run dev.heppen.webapps
```

Uninstall:

```bash
flatpak uninstall dev.heppen.webapps
```

### NixOS / Nix

This project includes a Nix flake for reproducible builds using [crane](https://github.com/ipetkov/crane).

#### Quick run (no installation)

```bash
nix run github:olafkfreund/cosmic-ext-web-apps
```

#### Build from source

```bash
git clone https://github.com/olafkfreund/cosmic-ext-web-apps.git
cd cosmic-ext-web-apps
nix build
./result/bin/dev.heppen.webapps
```

#### Development shell

Enter a shell with all build dependencies and development tools:

```bash
nix develop
just build-release
```

#### NixOS system-wide installation

Add the flake input to your `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cosmic-ext-web-apps = {
      url = "github:olafkfreund/cosmic-ext-web-apps";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, cosmic-ext-web-apps, ... }: {
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        {
          environment.systemPackages = [
            cosmic-ext-web-apps.packages.x86_64-linux.default
          ];
        }
      ];
    };
  };
}
```

Then rebuild:

```bash
sudo nixos-rebuild switch --flake .#yourhostname
```

#### Home Manager installation

Add the flake input and use it in your Home Manager configuration (loaded as a NixOS module):

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    cosmic-ext-web-apps = {
      url = "github:olafkfreund/cosmic-ext-web-apps";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, home-manager, cosmic-ext-web-apps, ... }: {
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./configuration.nix
        home-manager.nixosModules.home-manager
        {
          home-manager.users.yourusername = {
            home.packages = [
              cosmic-ext-web-apps.packages.x86_64-linux.default
            ];
          };
        }
      ];
    };
  };
}
```

#### Nix flake checks

Run clippy and formatting checks:

```bash
nix flake check
```

### From source (generic Linux)

Requirements:
- Rust (latest stable)
- `pkg-config`
- `libxkbcommon-dev`
- `libwebkit2gtk-4.1-dev`
- `libssl-dev`
- `libgtk-3-dev`
- `just`

```bash
git clone https://github.com/cosmic-utils/web-apps.git
cd web-apps
just build-release
sudo just install
```

## Usage

Created Web Apps use the WebKitGTK rendering engine. To create a new Web App, fill in the editor form with:

- A valid URL (starting with `http://` or `https://`)
- A name for the application (minimum 3 characters)
- An icon (press the icon button to choose from your system or download a favicon)
- A category for the web app

The application uses the [DynamicLauncher Portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.DynamicLauncher.html) to create launcher entries. Make sure your system supports this portal.

### Additional options

| Option | Description |
|--------|-------------|
| Persistent profile | Keep browser data between sessions in an isolated directory |
| Window size | Custom width and height (200-8192 pixels) |
| Decorations | Show or hide window title bar and borders |
| Private mode | Run in incognito mode (no data persisted) |
| Simulate mobile | Use a mobile user agent string for mobile-optimized sites |

### Import and export

You can export all your web apps to a `.ron` file for backup or sharing, and import them on another machine. Use the app menu to access import/export options.

### Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | Create new web app |
| `Ctrl+S` | Save current web app |

## Architecture

The project consists of two binaries:

- **`dev-heppen-webapps`** - The main GUI application built with libcosmic (iced-based)
- **`dev-heppen-webapps-webview`** - Per-app webview process using wry/tao/gtk with WebKitGTK

Data is stored as RON files in `$XDG_DATA_HOME/dev.heppen.webapps/database/`. Desktop entries are created via the XDG DynamicLauncher portal (ashpd). Internationalization uses Fluent `.ftl` files via the `fl!()` macro.

## License

Code is distributed under the [GPL-3.0 license](https://github.com/cosmic-utils/web-apps/blob/master/LICENSE).
