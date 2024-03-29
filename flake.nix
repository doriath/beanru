{
  description = "Flake for beanru";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
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
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
        rust = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.recurseIntoAttrs (pkgs.makeRustPlatform {
          rustc = rust;
          cargo = rust;
        });
        beanru = rustPlatform.buildRustPackage {
          name = manifest.name;
          version = manifest.version;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          src = pkgs.lib.cleanSource ./.;
          buildFeatures = ["build-binary"];
          nativeBuildInputs = [
            pkgs.pkg-config
          ];
        };
      in
      rec
      {
        formatter = pkgs.nixpkgs-fmt;

        packages = flake-utils.lib.flattenTree {
          beanru = beanru;
        };

        defaultPackage = packages.beanru;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.bashInteractive
            pkgs.rust-analyzer
            rust
          ];
        };
      }
    );
}
