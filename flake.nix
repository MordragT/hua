{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = { self, nixpkgs, utils, fenix }:
    let
      overlay = (final: prev: {
        hua = prev.callPackage (import ./default.nix) {
          inherit fenix;
        };
      });
    in
    utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            overlays = [ fenix.overlays.default ];
            inherit system;
          };
          toolchain = pkgs.fenix.complete;
        in
        rec {
          # `nix build`
          packages.hua = (pkgs.makeRustPlatform {
            # Use nightly rustc and cargo provided by fenix for building
            inherit (toolchain) cargo rustc;
          }).buildRustPackage {
            pname = "hua";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            buildInputs = with pkgs; [
              openssl
              pkgconfig
              bubblewrap
            ];
            meta = with pkgs.lib; {
              description = "Hua package manager";
            };

            # For other makeRustPlatform features see: 
            # https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md#cargo-features-cargo-features
          };
          packages.default = packages.hua;

          apps.hua = utils.lib.mkApp {
            drv = packages.hua;
          };
          apps.default = apps.hua;

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              (with toolchain; [
                cargo
                rustc
                rust-src
                clippy
                rustfmt
              ])
              openssl
              pkgconfig
              bubblewrap
              just
            ];
          };
        }) // {
      overlays.default = this: pkgs: {
        hua = self.packages."${pkgs.system}".default;
      };
    };
}
