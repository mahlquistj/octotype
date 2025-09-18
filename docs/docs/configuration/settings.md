---
sidebar_position: 2
---

# 🔧 Settings

OctoType accepts a configuration file in the [TOML](https://toml.io/en/) format,
located in the default configuration folder for your system:

| System  | Location                                                                        |
| ------- | ------------------------------------------------------------------------------- |
| Linux   | `$XDG_CONFIG_HOME/octotype/config.toml` or `$HOME/.config/octotype/config.toml` |
| MacOS   | `$HOME/Library/Application Support/com.Mahlquist.OctoType/config.toml`          |
| Windows | `%AppData%\OctoType\config\config.toml`                                         |

The configuration location can also be specified with the `-c <path>` flag.

Some official themes can be found
[in the repo](https://github.com/mahlquistj/octotype/tree/main/themes)

## Default settings

The default settings looks like so:

```toml
sources_dir = "/path/to/your/config/octotype/sources"
modes_dir = "/path/to/your/config/octotype/modes"
words_per_line = 5
show_ghost_lines = 3
ghost_opacity = [
    0.2,
    0.5,
    0.8,
]

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

[theme.cursor]
color = "White"
text = "Black"
```

## Options and Types

| Key                         | Type            | Description                                                                                             |
| --------------------------- | --------------- | ------------------------------------------------------------------------------------------------------- |
| `sources_dir`               | `String`        | Overwrites the path of the sources directory                                                            |
| `modes_dir`                 | `String`        | Overwrites the path of the modes directory                                                              |
| `words_per_line`            | `int`           | How many words should be displayed per line                                                             |
| `show_ghost_lines`          | `int`           | How many "ghost lines" should be displayed around the active line                                       |
| `ghost_opacity`             | `[float]`       | Overwrite the levels of opacity for each ghost line. Must have a length matching `show_ghost_lines`     |
| `disable_ghost_fade`        | `bool`          | Set this to true if you want the "scrolling" behaviour of ghost lines, but don't like the fading colors |
| `theme.spinner_color`       | `Color`         | Sets the color of the loading-screen spinner                                                            |
| `theme.spinner_symbol`      | `SpinnerSymbol` | Sets the symbol of the loading-screen spinner                                                           |
| `theme.term_fg`             | `Color`         | The foreground of your terminal (Queried directly from you terminal or else it defaults to White)       |
| `theme.term_bg`             | `Color`         | The background of your terminal (Queried directly from you terminal or else it defaults to Black)       |
| `theme.text.success`        | `Color`         | Sets the color of `success`-type text                                                                   |
| `theme.text.warning`        | `Color`         | Sets the color of `warning`-type text                                                                   |
| `theme.text.error`          | `Color`         | Sets the color of `error`-type text                                                                     |
| `theme.text.highlight`      | `Color`         | Sets the color of highlighted text                                                                      |
| `theme.plot.raw_wpm`        | `Color`         | Sets the color of the raw_wpm datapoints                                                                |
| `theme.plot.actual_wpm`     | `Color`         | Sets the color of the actual_wpm datapoints                                                             |
| `theme.plot.accurracy`      | `Color`         | Sets the color of the accurracy datapoints                                                              |
| `theme.plot.errors`         | `Color`         | Sets the color of the error datapoints                                                                  |
| `theme.plot.scatter_symbol` | `PlotSymbol`    | Sets the symbols of scatter-type plots (errors)                                                         |
| `theme.plot.line_symbol`    | `PlotSymbol`    | Sets the symbols of line-type plots (wpm, accurracy)                                                    |
| `theme.cursor.color`        | `Color`         | The color of the cursor when in a session                                                               |
| `theme.cursor.text`         | `Color`         | The color of the text under the cursor                                                                  |

### Colors

:::warning

While ANSI colors are supported, OctoType is made for mordern terminals with
24bit color support - We recommend using hex colors for the best experience
possible.

Some known issues regarding ghost-lines can happen, if not using hex colors.

:::

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

### PlotSymbol

This type determines what symbols are used to display data within the graphs

| Symbol      |
| ----------- |
| `Dot`       |
| `Block`     |
| `HalfBlock` |
| `Braille`   |
| `Bar`       |

### SpinnerSymbol

This type determines the symbol used for the "spinner" on the loading screen

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
