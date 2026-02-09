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

          nativeBuildInputs = with pkgs; [
            pkg-config
            wrapGAppsHook
          ];

          buildInputs = with pkgs; [
            openssl
            libxkbcommon
            wayland
            gtk3
            webkitgtk_4_1
            glib-networking
          ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        cosmic-web-apps = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ pkgs.just ];

          installPhase = ''
            runHook preInstall

            just --set prefix "$out" \
              --set bin-src "target/release/dev-heppen-webapps" \
              --set webview-src "target/release/dev-heppen-webapps-webview" \
              install

            runHook postInstall
          '';

          meta = with pkgs.lib; {
            description = "Web applications at your fingertips - COSMIC desktop web app manager";
            homepage = "https://github.com/cosmic-utils/web-apps";
            license = licenses.gpl3Only;
            maintainers = with maintainers; [ ];
            platforms = platforms.linux;
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
          packages = with pkgs; [
            rust-analyzer
            rustfmt
            clippy
            cargo-watch
            just
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
