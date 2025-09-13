# 🐙 OctoType - A typing trainer for your terminal

OctoType is an TUI typing trainer made with [Ratatui] - Heavily inspired by
[Monkeytype], with a focus on customizability

<!--toc:start-->

- [💡 Features](#💡-features)
- [🔖 Arguments](#🔖-arguments)
- [⚙️ Configuration](#️-configuration)
- [💻 Development](#💻-development)
- [⭐ Contributing](#contributing)

<!--toc:end-->

## 💡 Features

- 🎭 Custom [Modes](https://github.com/mahlquistj/octotype/wiki/Modes)
- 🖊️ Custom [Sources](https://github.com/mahlquistj/octotype/wiki/Sources)
- 🎨 Custom theming
- 🪶 Lightweight (~3MB)
- 🔥 Blazingly fast (Sorry, i had to.. 🦀)
- .. And more to come!

## 🔖 Arguments

| Short       | Long               | Description                                    |
| ----------- | ------------------ | ---------------------------------------------- |
|             | `--print-config`   | Prints the current settings, modes, and source |
| `-p`        | `--print-settings` | Prints the current settings                    |
| `-c <path>` | `--config <path>`  | Overrides the default config location          |
| `-h`        | `--help`           | Shows a help page with the list of arguments   |

## ⚙️ Configuration

Check out the [wiki](https://github.com/mahlquistj/octotype/wiki/Configuration)
for configuration options.

## 💻 Development

A nix flake dev-shell is provided to run with `nix-develop`.

## ⭐ Contributing

If you have an idea, bug-report or alike, feel free to open an issue!

<!-- LINKS -->

[Monkeytype]: https://monkeytype.com/
[Ratatui]: https://ratatui.rs/
