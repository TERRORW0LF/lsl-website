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
          mdbook-embedify = prev.rustPlatform.buildRustPackage rec {
            pname = "mdbook-embedify";
            version = "0.2.11";

            src = prev.fetchFromGitHub {
              owner = "MR-Addict";
              repo = pname;
              rev = version;
              hash = "sha256-xmpGSSwyJ+pSYF6qUjuMGpYPR5Ipki9mqpid4FcWea0=";
            };

            cargoHash = "sha256-UPmWhDnrNGsbXl5E+cfW0UaAbw54zrL6XZx0UPbrLCQ=";
          };
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
            wasm-bindgen-cli_0_2_106
            mdbook
            mdbook-embedify
            binaryen
            dart-sass
            leptosfmt
            gcc
          ];
        };
      });
    };
}
