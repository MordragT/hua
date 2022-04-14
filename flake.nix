{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, utils, naersk, fenix }: let
    overlay = (final: prev: {
      hua = prev.callPackage (import ./default.nix) {
        inherit naersk fenix;
      };
    });
  in { overlay = overlay; } // utils.lib.eachDefaultSystem (system: let
    pkgs = nixpkgs.legacyPackages."${system}";
    naersk-lib = naersk.lib."${system}";
    toolchain = fenix.packages.${system}.complete;
    llvmPkgs = pkgs.llvmPackages_12;
  in rec {
    # `nix build`
    packages.hua = import ./default.nix {
      inherit system;
      inherit (nixpkgs) lib;
      inherit pkgs;
      inherit naersk fenix;
    };
      
    defaultPackage = packages.hua;

    # `nix run`
    apps.hua = utils.lib.mkApp {
      drv = packages.hua;
    };
    defaultApp = apps.hua;

    # `nix develop`
    devShell = pkgs.mkShell {
      # LLVM_SYS_120_PREFIX = "${llvmPkgs.llvm.dev}";
      # NIX_GLIBC_PATH = if pkgs.stdenv.isLinux then "${pkgs.glibc_multi.out}/lib" else "";
      # RUST_SRC_PATH = "${complete.rust-src}/lib/rustlib/src/rust/src";
      # LD_LIBRARY_PATH = with pkgs;
      #   lib.makeLibraryPath
      #   [ pkg-config stdenv.cc.cc.lib libffi ncurses zlib ];

      nativeBuildInputs = with pkgs; [
        (toolchain.withComponents [
          "cargo" "rustc" "rust-src" "rustfmt" "clippy"    
        ])
        # xorg.libxcb
        openssl
        pkgconfig
        bubblewrap
        # llvmPkgs.clang
        # llvmPkgs.lld
        # llvmPkgs.llvm.dev
        # lldb
        zig
        # (pkgs.writeShellScriptBin "roc" ''
        #   #!/usr/bin/env sh

        #   $HOME/.cargo/bin/roc $@
        # '')
      ];
    };
  });
}
