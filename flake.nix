{
  description = "Quick Web Apps - COSMIC desktop web app manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Cargo.toml uses edition = "2024" which requires Rust >= 1.85.0
        _ = if (builtins.compareVersions pkgs.rustc.version "1.85.0" < 0)
            then builtins.throw "Rust >= 1.85.0 required for edition 2024 (found ${pkgs.rustc.version}). Update your nixpkgs input."
            else null;

        craneLib = crane.mkLib pkgs;

        commonArgs = {
          src = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = path: type:
              (craneLib.filterCargoSources path type) ||
              (builtins.match ".*resources.*" path != null) ||
              (builtins.match ".*i18n.*" path != null) ||
              (builtins.match ".*justfile$" path != null);
          };

          pname = "dev-heppen-webapps";
          version = "2.0.1";

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.wrapGAppsHook3
          ];

          buildInputs = [
            pkgs.openssl
            pkgs.libxkbcommon
            pkgs.wayland
            pkgs.gtk3
            pkgs.webkitgtk_4_1
            pkgs.glib-networking
          ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        cosmic-web-apps = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ pkgs.just ];

          # wget is used at runtime for favicon downloads
          preFixup = ''
            gappsWrapperArgs+=(
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.wget ]}
            )
          '';

          installPhase = ''
            runHook preInstall

            just --set prefix "$out" \
              --set bin-src "target/release/dev-heppen-webapps" \
              --set webview-src "target/release/dev-heppen-webapps-webview" \
              install

            runHook postInstall
          '';

          meta = {
            description = "Web applications at your fingertips - COSMIC desktop web app manager";
            homepage = "https://github.com/cosmic-utils/web-apps";
            license = pkgs.lib.licenses.gpl3Only;
            maintainers = [];
            platforms = pkgs.lib.platforms.linux;
            mainProgram = "dev-heppen-webapps";
          };
        });
      in
      {
        packages = {
          inherit cosmic-web-apps;
          default = cosmic-web-apps;
        };

        devShells.default = craneLib.devShell {
          packages = [
            pkgs.rust-analyzer
            pkgs.rustfmt
            pkgs.clippy
            pkgs.cargo-watch
            pkgs.just
          ];

          inputsFrom = [ cosmic-web-apps ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };

        apps.default = {
          type = "app";
          program = "${cosmic-web-apps}/bin/dev.heppen.webapps";
        };

        checks = {
          workspace-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          workspace-fmt = craneLib.cargoFmt {
            src = ./.;
          };
        };
      }
    );
}
