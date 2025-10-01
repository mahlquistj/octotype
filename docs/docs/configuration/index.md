---
sidebar_position: 1
---

# 🐙 Getting started

:::info

OctoType is still in early development, so the docs are subject to change, and
might not be entirely up to date at the moment.

:::

## Installation

Installing OctoType is currently only tested on Linux - Feel free to open an
[Issue](https://github.com/mahlquistj/octotype/issues/new/choose) if any
problems occur on other systems.

### Cargo

OctoType can be easily installed through
[Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) by
running:

```sh
cargo install octotype
```

To install a specific version, you can prepend the version after the package
name:

```sh
cargo install octotype@0.3.2
```

### Nix

OctoType is currently only supported as a
[Nix Flake](https://nixos.wiki/wiki/Flakes). You can add the flake to your
config, and install octotype like so:

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    octotype.url = "github:mahlquistj/octotype/main";
  };
  
  outputs = { self, nixpkgs, rio }: {
    # Replace `hostname` with your own hostname
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      modules = [
        ({ pkgs, ... }: {
          # Add OctoType to your system packages
          environment.systemPackages = [
            octotype.packages.${pkgs.system}.octotype
          ];
        })
      ];
    };
  };
}
```

For more info regarding using OctoType with
[Home Manager](https://github.com/nix-community/home-manager) see
[Settings](settings)

## 🔖 CLI Arguments

| Short       | Long               | Description                                    |
| ----------- | ------------------ | ---------------------------------------------- |
|             | `--print-config`   | Prints the current settings, modes, and source |
| `-p`        | `--print-settings` | Prints the current settings                    |
| `-c <path>` | `--config <path>`  | Overrides the default config location          |
| `-h`        | `--help`           | Shows a help page with the list of arguments   |
