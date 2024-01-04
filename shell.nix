let sources = import ./nix/sources.nix;
in { pkgs ? import sources.nixpkgs { } }:

with pkgs;

let
  main = import ./default.nix { inherit pkgs; };
in
mkShell {
  buildInputs = main.buildInputs ++ main.nativeBuildInputs ++ [
    cargo-edit
    clippy
    niv
    nixfmt
    rust-analyzer
    rustc
    rustfmt
    shellcheck
    # Required for cargo
    git
    openssh
  ];
  inherit (main) BLAKE3_CSRC;
  CNS_IN_NIX_SHELL = "1";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
