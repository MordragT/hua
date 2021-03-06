{ system
, lib
, pkgs
, naersk
, fenix
}:

let
  toolchain = with fenix.packages.${system};
    combine [
      minimal.rustc
      minimal.cargo
    ];
  naersk-lib = naersk.lib.${system}.override {
    cargo = toolchain;
    rustc = toolchain;
  };
in
naersk-lib.buildPackage {
  pname = "hua";
  root = ./.;
  gitAllRefs = true;
  buildInputs = with pkgs; [
    openssl
    pkgconfig
    bubblewrap
  ];
  meta = with lib; {
    description = "Hua package manager";
  };
}
