{
  description = "floresta (forest) — brasa's userspace init + convergence orchestrator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    substrate = {
      url = "github:pleme-io/substrate";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  # Phase 0 note: dev shell only. Phase 2 wires up a brasa-target binary build
  # once the aarch64-unknown-brasa triple is registered.
  outputs = { self, nixpkgs, flake-utils, fenix, substrate }:
    flake-utils.lib.eachSystem [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustToolchain = (fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = pkgs.lib.fakeSha256;
        });
      in
      {
        devShells.default = pkgs.mkShellNoCC {
          name = "floresta-dev";
          packages = [ rustToolchain ] ++ (with pkgs; [
            cargo-nextest
            cargo-watch
            just
          ]);
          shellHook = ''
            echo "floresta dev shell — Phase 0 (design)"
            echo "See https://github.com/pleme-io/brasa for the kernel."
          '';
        };

        packages.default = pkgs.writeTextFile {
          name = "floresta-phase-0-marker";
          text = "floresta Phase 0 — no binary yet. See README.md.\n";
          destination = "/STATUS";
        };
      }
    );
}
