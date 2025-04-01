# 🐙 OctoType - A typing trainer for your terminal

OctoType is a simple TUI typing trainer made with [Ratatui](ratatui) and heavily
inspired by [Monkeytype](monkeytype)

# 🔍 Features

- [x] Configurable

- [x] In-depth statistics to improve your bad habits

- [x] Multiple word-generating APIs to chooose from: Quotes and Random words
      (More to come)

- [ ] Multiple modes (Normal, Timed, and custom)

- [ ] Ability to save statistics to track improvements

- [ ] Nix flake package

- [ ] Nix flake home-manager module

# ❓ Why

I made this when i got a new split keyboard while trying to get into vim. I
found myself using [Monkeytype](monkeytype) a lot (Which is where the
inspiration came from), and needed a project to work on to not only practice my
keyboard skills, but also something to do in my new neovim setup that wasn't too
heavy.

# 🔖 Arguments

| Short       | Long              | Description                                  |
| ----------- | ----------------- | -------------------------------------------- |
| `-p`        | `--print-config`  | Prints the current configuration             |
| `-c <path>` | `--config <path>` | Overrides the default config location        |
| `-h`        | `--help`          | Shows a help page with the list of arguments |

# 💻 Development

A nix flake dev-shell is provided to run with `nix-develop` otherwise, it should
be pretty straight forward using cargo.

# ⚙️ Configuration

OctoType accepts a configuration file in the TOML format, located in the default
configuration folder for your system:

| System  | Location                                                                        |
| ------- | ------------------------------------------------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME/octotype/config.toml` or `$HOME/.config/octotype/config.toml` |
| MacOS   | `$HOME/Library/Application Support/com.Mahlquist.OctoType/config.toml`          |
| Windows | `%AppData%\OctoType\config\config.toml`                                         |

## 📘 Default configuration

The default configuration looks like so:

```toml
[theme]
spinner_color = "Yellow"
spinner_symbol = "BrailleSix"

[theme.text]
success = "Green"
warning = "Yellow"
error = "Red"
highlight = "Blue"

[theme.plot]
raw_wpm = "Gray"
actual_wpm = "Yellow"
accurracy = "Gray"
errors = "Red"
scatter_symbol = "Dot"
line_symbol = "HalfBlock"
```

## ✅ Options

| Key                         | Type            | Description                                          |
| --------------------------- | --------------- | ---------------------------------------------------- |
| `theme.spinner_color`       | `Color`         | Sets the color of the loading-screen spinner         |
| `theme.spinner_symbol`      | `SpinnerSymbol` | Sets the symbol of the loading-screen spinner        |
| `theme.text.success`        | `Color`         | Sets the color of `success`-type text                |
| `theme.text.warning`        | `Color`         | Sets the color of `warning`-type text                |
| `theme.text.error`          | `Color`         | Sets the color of `error`-type text                  |
| `theme.text.highlight`      | `Color`         | Sets the color of highlighted text                   |
| `theme.plot.raw_wpm`        | `Color`         | Sets the color of the raw_wpm datapoints             |
| `theme.plot.actual_wpm`     | `Color`         | Sets the color of the actual_wpm datapoints          |
| `theme.plot.accurracy`      | `Color`         | Sets the color of the accurracy datapoints           |
| `theme.plot.errors`         | `Color`         | Sets the color of the error datapoints               |
| `theme.plot.scatter_symbol` | `PlotSymbol`    | Sets the symbols of scatter-type plots (errors)      |
| `theme.plot.line_symbol`    | `PlotSymbol`    | Sets the symbols of line-type plots (wpm, accurracy) |

## ❗ Types

<details>
    <summary>Colors - Click to expand</summary>

Two different types of colors can be supplied: ANSI or Custom which sets a
specific color.

| Color          | Type   | Description |
| -------------- | ------ | ----------- |
| `#RRGGBB`      | Custom | Hex color   |
| `<integer>`    | ANSI   | ANSI index  |
| `Black`        | ANSI   |             |
| `Red`          | ANSI   |             |
| `Green`        | ANSI   |             |
| `Yellow`       | ANSI   |             |
| `Blue`         | ANSI   |             |
| `Magenta`      | ANSI   |             |
| `Cyan`         | ANSI   |             |
| `Gray`         | ANSI   |             |
| `DarkGray`     | ANSI   |             |
| `LightRed`     | ANSI   |             |
| `LightGreen`   | ANSI   |             |
| `LightYellow`  | ANSI   |             |
| `LightBlue`    | ANSI   |             |
| `LightMagenta` | ANSI   |             |
| `LightCyan`    | ANSI   |             |
| `White`        | ANSI   |             |

</details>

<details>
    <summary>PlotSymbols - Click to expand</summary>

| Symbol      |
| ----------- |
| `Dot`       |
| `Block`     |
| `HalfBlock` |
| `Braille`   |
| `Bar`       |

</details>

<details>
    <summary>SpinnerSymbols - Click to expand</summary>

| Symbol               |
| -------------------- |
| `Ascii`              |
| `Arrow`              |
| `BlackCircle`        |
| `BoxDrawing`         |
| `BrailleOne`         |
| `BrailleDouble`      |
| `BrailleSix`         |
| `BrailleSixDouble`   |
| `BrailleEight`       |
| `BrailleEightDouble` |
| `Canadian`           |
| `Clock`              |
| `DoubleArrow`        |
| `HorizontalBlock`    |
| `OghamA`             |
| `OghamB`             |
| `OghamC`             |
| `Paranthesis`        |
| `QuadrantBlock`      |
| `QuadrantBlockCrack` |
| `VerticalBlock`      |
| `WhiteCircle`        |
| `WhiteSquare`        |

</details>

# ⭐ Contributing

While the project is still new, feel free to open an issue with suggestions or
alike. When i feel like the project is mature enough, i will be accepting
pull-requests as well.

<!-- LINKS -->

[monkeytype]: https://monkeytype.com/
[ratatui]: https://ratatui.rs/
