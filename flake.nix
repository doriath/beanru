{
  description = "Development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust = pkgs.rust-bin.stable.latest.default.override {
          targets = ["wasm32-wasi" "wasm32-unknown-unknown"];
        };

      in
      rec
      {
        formatter = pkgs.nixpkgs-fmt;

        packages = flake-utils.lib.flattenTree {
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          buildInputs = [
            pkgs.autoconf
            pkgs.automake
            pkgs.bashInteractive
            pkgs.openssl
            pkgs.rust-analyzer
            rust
          ];
          shellHook = ''
          '';
        };
      }
    );
}
