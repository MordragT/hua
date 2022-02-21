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
      nativeBuildInputs = with pkgs; [
        rustc
        cargo
        openssl
        pkgconfig
      ];
    };
  });
}
