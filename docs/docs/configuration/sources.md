---
sidebar_position: 5
---

# 🖊️ Sources

A "Source" in OctoType is what generates the words/sentences/characters for a
session.

Sources can generate text in two ways:
- **Command-based**: Execute an external command/script that outputs words
- **List-based**: Use a predefined list of words (from an array or file), with optional randomization

Sources are loaded from `<OCTOTYPE_CONFIG_DIR>/sources/<source>.toml`.

Official maintained sources can be found
[in the repo](https://github.com/mahlquistj/octotype/tree/main/sources)

:::warning

Beware when using Command-based sources made by other people. Command sources execute commands
directly on your PC, and could potentially be malicious! Make sure you read and
understand what a source is executing before using it!

:::

## File structure

### Command-based generator
```toml
[meta]
name = "My Source"
description = "My custom Source!"

[generator]
command = ["my-command", "arg1", "arg2"]
# .. Optional: formatting, network_required, required_tools

[parameters]
# .. Parameters the user can customize for this source
```

### List-based generator
```toml
[meta]
name = "My Source"
description = "My custom Source!"

[generator]
source = ["word1", "word2", "word3"]  # Inline array
# or: source = { path = "file.txt" }  # File-based (whitespace separator)
# or: source = { path = "file.txt", separator = "," }  # File-based with custom separator
randomize = true

[parameters]
# .. Parameters the user can customize for this source
```

## Options

### `meta`

| option      | type     | description                |
| ----------- | -------- | -------------------------- |
| name        | `String` | The name of the Source     |
| description | `String` | A description of the Source |

### `generator`

The generator section defines how text is generated. There are two types:

#### Command Generator

Executes an external command/script to generate text.

| option           | type       | required | description                                                                                               |
| ---------------- | ---------- | -------- | --------------------------------------------------------------------------------------------------------- |
| command          | `[String]` | yes      | The [command](#command) to execute                                                                        |
| formatting       | `String`   | no       | Output [formatting](#formatting) (default: "raw")                                                         |
| network_required | `bool`     | no       | Set to true if network is required for this source (default: false)                                       |
| required_tools   | `[String]` | no       | A list of commands needed to use the Source (recommended if you want to share your source)                |

#### List Generator

Uses a predefined list of words, either inline or from a file.

| option    | type                  | required | description                                                                                    |
| --------- | --------------------- | -------- | ---------------------------------------------------------------------------------------------- |
| source    | `[String]` or `Table` | yes      | Either an inline array of words, or a table with `path` (and optional `separator`)            |
| randomize | `bool`                | yes      | Whether to shuffle the words each time (true) or use them in order (false)                    |

For file-based sources:
- `source.path`: Path to the file containing words
- `source.separator`: Optional character to split on (default: any whitespace)

### `parameters`

Any key is accepted here - See the [Parameters](parameters) section for more
info.

Keys defined here can be used as [Replacements](parameters#replacements) for
values within command-based generator's `command` field

## Command Generator Details

### Command Syntax

Since OctoType uses Rust's `std::process::Command` to execute commands, a
list of arguments are needed.

Do this: `["cat", "myfile.txt"]`

Not this: `["cat myfile.txt"]`

Piping and other operators are not supported as of right now. If these are
needed then you must make a script:

```toml
command = ["cat", "myfile.txt", "|", "head"] # Would fail
```

Instead, we put it in a script:

```sh
# File: myscript.sh
cat myfile.txt | head
```

And then supply the command:

```toml
command = ["/path/to/myscript.sh"]
```

### Formatting

The `formatting` option controls how the command output is processed:

- `"raw"` (default): Words separated by any ascii-whitespace (newlines, tabs, single/multiple spaces, etc.) are translated to a single `<space>`
- `"spaced"`: Similar to raw (future expansion planned)

Example with `formatting = "raw"`:

Input:
```
the quick  brown
            fox
```

Output:
```
the quick brown fox
```

## List Generator Details

List generators provide a simpler way to define static word lists without needing external commands.

### Inline Arrays

The simplest form uses an inline array:

```toml
[generator]
source = ["word1", "word2", "word3"]
randomize = false
```

### File-based Lists

For larger word lists, you can load from a file:

```toml
[generator]
source = { path = "words.txt" }
randomize = true
```

By default, the file is split on any whitespace. You can specify a custom separator:

```toml
[generator]
source = { path = "words.csv", separator = "," }
randomize = true
```

### Randomization

- `randomize = true`: Shuffles the words each time they're fetched
- `randomize = false`: Uses words in the order they're defined
