{
  description = "Quick Web Apps - COSMIC desktop web app manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, crane }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      buildFor = system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          craneLib = crane.mkLib pkgs;
        in
        assert (builtins.compareVersions pkgs.rustc.version "1.85.0" >= 0)
          || (builtins.throw "Rust >= 1.85.0 required for edition 2024 (found ${pkgs.rustc.version}). Update your nixpkgs input.");
        let
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

          cosmic-ext-web-apps = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;

            nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ pkgs.just ];

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
        { inherit commonArgs cargoArtifacts craneLib;
          package = cosmic-ext-web-apps;
        };
    in
    {
      packages = forAllSystems (system:
        let b = buildFor system; in {
          cosmic-ext-web-apps = b.package;
          default = b.package;
        }
      );

      devShells = forAllSystems (system:
        let
          b = buildFor system;
          pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = b.craneLib.devShell {
            packages = [
              pkgs.rust-analyzer
              pkgs.cargo-watch
              pkgs.just
            ];

            inputsFrom = [ b.package ];

            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        }
      );

      apps = forAllSystems (system:
        let b = buildFor system; in {
          default = {
            type = "app";
            program = "${b.package}/bin/dev.heppen.webapps";
          };
        }
      );

      checks = forAllSystems (system:
        let b = buildFor system; in {
          workspace-clippy = b.craneLib.cargoClippy (b.commonArgs // {
            inherit (b) cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          workspace-fmt = b.craneLib.cargoFmt {
            src = ./.;
          };
        }
      );

      overlays.default = final: _prev: {
        cosmic-ext-web-apps = self.packages.${final.system}.default;
      };
    };
}
