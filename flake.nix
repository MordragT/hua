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
                
      # RUST_SRC_PATH = "${complete.rust-src}/lib/rustlib/src/rust/src";
        
      nativeBuildInputs = with pkgs; [
        (toolchain.withComponents [
          "cargo" "rustc" "rust-src" "rustfmt" "clippy"    
        ])
        openssl
        pkgconfig
      ];
    };
  });
}
