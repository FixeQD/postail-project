{
  description = "Postail - secure Tauri email client";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:

    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];

          targets = [ "x86_64-pc-windows-gnu" ];
        };

        mingw = pkgs.pkgsCross.mingwW64;

        winTargetEnv = "CARGO_TARGET_X86_64_PC_WINDOWS_GNU";
        winCC = "${mingw.stdenv.cc}/bin/${mingw.stdenv.cc.targetPrefix}gcc";
        winCXX = "${mingw.stdenv.cc}/bin/${mingw.stdenv.cc.targetPrefix}g++";
        winAR = "${mingw.stdenv.cc}/bin/${mingw.stdenv.cc.targetPrefix}ar";

        # tauri-cli's `--target` handling doesn't inspect the rustc/cargo it's actually about to invoke
        fakeRustup = pkgs.writeShellScriptBin "rustup" ''
          if [ "$1" = "target" ] && [ "$2" = "list" ]; then
            echo "x86_64-unknown-linux-gnu (installed)"
            echo "x86_64-pc-windows-gnu (installed)"
            exit 0
          fi
          echo "rustup: this is a Nix devShell shim for tauri-cli's target check only." >&2
          echo "the real toolchain here is managed by rust-overlay, not rustup." >&2
          exit 1
        '';

        nativeBuildInputs = with pkgs; [
          pkg-config
          wrapGAppsHook3
        ];

        buildInputs = with pkgs; [
          glib
          gtk3
          webkitgtk_4_1
          libsoup_3
          cairo
          pango
          gdk-pixbuf
          atk
          at-spi2-atk
          librsvg
          dbus
          openssl
          sqlite
          libayatana-appindicator
          tpm2-tss
          gsettings-desktop-schemas
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          packages = [
            rustToolchain
            pkgs.bun
            pkgs.cargo-tauri
          ];

          shellHook = ''
            # Workaround for GTK/WebKitGTK DPI bug in Nix devShells (no schema = broken devicePixelRatio in Wayland)
            # see https://github.com/tauri-apps/tauri/issues/5600#issuecomment-4871251687
            export GSETTINGS_SCHEMA_DIR="${pkgs.glib.getSchemaPath pkgs.gsettings-desktop-schemas}"
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH
            echo "postail dev shell ready!"
          '';
        };

        devShells.windows-cross = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs ++ [
            mingw.stdenv.cc
            mingw.buildPackages.binutils
          ];

          buildInputs = [ mingw.windows.pthreads ];

          packages = [
            rustToolchain
            pkgs.bun
            pkgs.cargo-tauri
            fakeRustup
          ];

          # cargo's own cross linker hookup.
          "CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER" = winCC;

          # [lib] crate-type = ["staticlib", "cdylib", "rlib"] in src-tauri/Cargo.toml: pls sybau, MSVC worked without `--exclude-all-symbols`
          "CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS" = "-C link-arg=-Wl,--exclude-all-symbols";

          "CC_x86_64-pc-windows-gnu" = winCC;
          "CC_x86_64_pc_windows_gnu" = winCC;
          "CXX_x86_64-pc-windows-gnu" = winCXX;
          "CXX_x86_64_pc_windows_gnu" = winCXX;
          "AR_x86_64-pc-windows-gnu" = winAR;
          "AR_x86_64_pc_windows_gnu" = winAR;

          OPENSSL_STATIC = "1";
          OPENSSL_DIR = "${mingw.openssl.dev}";
          OPENSSL_LIB_DIR = "${mingw.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${mingw.openssl.dev}/include";

          shellHook = ''
            echo "postail windows-cross shell ready"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "postail";
          version = "0.1.0";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;
          buildAndTestSubdir = "src-tauri";

          inherit nativeBuildInputs buildInputs;

          # No network access inside the build sandbox
          doCheck = false;
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
