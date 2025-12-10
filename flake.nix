{
  description = "PrsnlAssistant Native App - Dioxus mobile development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    android-nixpkgs = {
      url = "github:tadfisher/android-nixpkgs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, android-nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        };

        # Android SDK configuration
        androidSdk = android-nixpkgs.sdk.${system} (sdkPkgs: with sdkPkgs; [
          build-tools-34-0-0
          build-tools-33-0-0
          cmdline-tools-latest
          platform-tools
          platforms-android-34
          platforms-android-33
          ndk-26-1-10909125
          emulator
        ]);

      in {
        devShells.default = pkgs.mkShell {
          name = "dioxus-mobile-dev";

          nativeBuildInputs = with pkgs; [
            pkg-config
            cmake
          ];

          buildInputs = with pkgs; [
            # Task runner
            go-task

            # Dioxus CLI
            dioxus-cli

            # Android SDK (from android-nixpkgs)
            androidSdk

            # Android tools from nixpkgs (adb, fastboot)
            android-tools

            # Rust toolchain
            rustup
            cargo
            rustc
            rust-analyzer

            # C compiler and linker
            gcc
            binutils

            # Build dependencies
            pkg-config
            openssl
            openssl.dev

            # For desktop development/testing (GTK/WebKit)
            webkitgtk_4_1
            gtk3
            gtk3.dev
            libsoup_3
            glib
            glib.dev
            cairo
            pango
            gdk-pixbuf
            atk
            at-spi2-atk
            libxkbcommon
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libxcb

            # Dioxus specific
            dbus
            dbus.dev

            # For file dialogs (rfd/ashpd)
            xdotool

            # Java for Android builds
            jdk17
          ];

          shellHook = ''
            # Set up Rust
            export RUSTUP_HOME="$HOME/.rustup"
            export CARGO_HOME="$HOME/.cargo"
            export PATH="$CARGO_HOME/bin:$PATH"

            # Ensure stable Rust with required targets
            rustup default stable 2>/dev/null || true
            rustup target add wasm32-unknown-unknown 2>/dev/null || true

            # PKG_CONFIG paths
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.gtk3.dev}/lib/pkgconfig:${pkgs.glib.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

            # Android SDK setup (from android-nixpkgs)
            export ANDROID_HOME="${androidSdk}/share/android-sdk"
            export ANDROID_SDK_ROOT="$ANDROID_HOME"
            export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/26.1.10909125"
            export PATH="$ANDROID_HOME/platform-tools:$ANDROID_HOME/tools:$ANDROID_HOME/tools/bin:$PATH"

            # Java for Android builds
            export JAVA_HOME="${pkgs.jdk17}"

            echo "Dioxus Mobile Development Environment (Flake)"
            echo "=============================================="
            echo ""
            echo "Android SDK: $ANDROID_HOME"
            echo "Android NDK: $ANDROID_NDK_HOME"
            echo ""
            echo "Quick Start:"
            echo "  task dev              - Run desktop app with hot reload"
            echo "  task adb:wireless     - Setup wireless ADB (guide)"
            echo "  task deploy           - Build & install to device"
            echo ""
            echo "All commands: task --list"
            echo ""
          '';

          # Environment variables
          RUST_BACKTRACE = "1";
          RUST_LOG = "info";

          # For OpenSSL
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        };
      }
    );
}
