{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
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

    # Android Tooling
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

    # Android SDK/NDK setup
    export ANDROID_HOME="$HOME/Android/Sdk"
    export ANDROID_SDK_ROOT="$ANDROID_HOME"
    if [ -d "$ANDROID_HOME/ndk" ]; then
      # Auto-detect NDK version (use latest)
      NDK_VERSION=$(ls "$ANDROID_HOME/ndk" 2>/dev/null | sort -V | tail -1)
      if [ -n "$NDK_VERSION" ]; then
        export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/$NDK_VERSION"
      fi
    fi

    echo "Dioxus Mobile Development Environment"
    echo "======================================"
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

}
