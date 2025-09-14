# Thanks to: https://fasterthanli.me/series/building-a-rust-service-with-nix/part-10#a-flake-with-a-dev-shell
# And: https://github.com/raphamorim/rio/blob/main/flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [];

      systems = import inputs.systems;

      perSystem = {
        self',
        pkgs,
        system,
        lib,
        ...
      }: let
        mkOctoType = import ./pkgOctotype.nix;
        rustToolchain = pkgs.rust-bin.stable.latest.minimal;
      in {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [(import inputs.rust-overlay)];
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [rustToolchain];
          packages = self'.packages.octotype.nativeBuildInputs ++ self'.packages.octotype.buildInputs ++ [rustToolchain];
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath self'.packages.octotype.runtimeDependencies}";
        };

        packages.default = pkgs.callPackage mkOctoType {rust-toolchain = pkgs.rust-bin.stable.latest.minimal;};

        apps.default = {
          type = "app";
          program = self'.packages.default;
        };
      };
    };
}
