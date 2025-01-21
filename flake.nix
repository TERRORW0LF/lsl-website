{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      overlays = [
        rust-overlay.overlays.default
        (final: prev: {
          rustToolchain =
            let
              rust = prev.rust-bin;
            in
            if builtins.pathExists ./rust-toolchain.toml then
              rust.fromRustupToolchainFile ./rust-toolchain.toml
            else if builtins.pathExists ./rust-toolchain then
              rust.fromRustupToolchainFile ./rust-toolchain
            else
              rust.stable.latest.default;
        })
        (final: prev: {
          cargo-leptos = prev.cargo-leptos.overrideAttrs (oldAttrs: rec {
              pname = "cargo-leptos";
              version = "0.2.26";

              src = prev.fetchFromGitHub {
                owner = "leptos-rs";
                repo = pname;
                rev = "v${version}";
                hash = "sha256-v1gNH3pq5db/swsk79nEzgtR4jy3f/xHs4QaLnVcVYU=";
              };

              cargoDeps = oldAttrs.cargoDeps.overrideAttrs (prev.lib.const {
                name = "${pname}-vendor.tar.gz";
                inherit src;
                outputHash = "sha256-ATfnMcwyOGlBDULi57VsLtLsL9n3K9TWbVPHX8N/BV0=";
              });
            });
        })
      ];
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs { inherit overlays system; };
      });
    in
    {
      devShells = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            openssl
            pkg-config
            cargo-deny
            cargo-edit
            cargo-watch
            cargo-leptos
            dart-sass
            rust-analyzer
            leptosfmt
            gcc
          ];
        };
      });
    };
}
