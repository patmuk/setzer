{
  description = "Setzer: static sites as Rust programs";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        toolchain = fenix.packages.${system}.stable.withComponents [
          "cargo" "rustc" "rust-src" "clippy" "rustfmt" "rust-analyzer"
        ];
        rustPlatform = pkgs.makeRustPlatform { cargo = toolchain; rustc = toolchain; };
      in
      {
        # `nix build` = `cargo test` + `cargo build` in the sandbox: the CI check.
        packages.default = rustPlatform.buildRustPackage {
          pname = "setzer";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          doCheck = true;
        };
        devShells.default = pkgs.mkShellNoCC {
          strictDeps = true;
          packages = [ toolchain pkgs.cargo-watch pkgs.just ];
        };
      }
    );
}
