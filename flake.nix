{
  description = "A basic flake with a shell";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    devenv.url = "github:cachix/devenv";
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, devenv }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            allowUnfree = true;
          };
        };
        rustPackages = pkgs.fenix.complete;
        rust = (rustPackages.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
        ]);
      in
      {
        packages.devenv = devenv.packages.${system}.devenv;

        devShells = {
          default = self.devShells.${system}.devenv;
          devenv = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              ({ pkgs, ... }: {
                languages.rust = {
                  enable = true;
                  packages = { inherit (rustPackages) cargo clippy rust-src rustc rustfmt; };
                };

                pre-commit.hooks = {
                  clippy.enable = true;
                  rustfmt.enable = true;
                };
                packages = with pkgs; [
                  openssl
                  pkg-config
                  llvmPackages_latest.llvm
                  llvmPackages_latest.bintools
                  llvmPackages_latest.lld
                  cargo-outdated
                  rust-analyzer-nightly
                ];

                services.mongodb.enable = true;

                enterShell = ''
                  # Unset all empty environment variables
                  unset $(printenv | awk -F= '$2=="" { print $1 }')
                  export RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
                '';
              })
            ];
          };
          nix-direnv = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              pkg-config
              llvmPackages_latest.llvm
              llvmPackages_latest.bintools
              llvmPackages_latest.lld
              cargo-outdated
              rust-analyzer-nightly
            ];
            buildInputs = [ rust pkgs.openssl ];

            # Certain Rust tools won't work without this
            # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
            # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        };
      });
}
