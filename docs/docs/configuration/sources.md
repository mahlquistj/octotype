---
sidebar_position: 5
---

# 🖊️ Sources

A "Source" in OctoType is what generates the words/sentences/characters for a
session.

Currently this is just an external command/script which takes some parameters,
and outputs some words for the user to type.

Sources are loaded from `<OCTOTYPE_CONFIG_DIG>/sources/<source>.toml`.

Official maintained sources can be found
[in the repo](https://github.com/mahlquistj/octotype/tree/main/sources)

:::warning

Beware when using Sources made by other people. Sources execute commands
directly on your PC, and could potentially be malicious! Make sure you read and
understand what a Source is executing before using it!

:::

## File structure

```
[meta]
name = "My Source"
description = "My custom Source!"
# .. Other metadata

[parameters]
# .. Parameters the user can customize for this source
```

## Options

### `meta`

| option              | type       | description                                                                                               |
| ------------------- | ---------- | --------------------------------------------------------------------------------------------------------- |
| name                | `String`   | The name of the Source                                                                                    |
| description         | `String`   | A description of the Source                                                                               |
| command             | `[String]` | The [command](#command) to execute.                                                                       |
| output              | `String`   | Unused, but reserved for the future: The [output format](#output) of the script                           |
| network_required    | `bool`     | Optional: Set to true if network is required for this source                                              |
| offline_alternative | `String`   | Optional: An offline alternative to this source (Used if network fails)                                   |
| required_tools      | `[String]` | Optional (but recommended, if you want to share your source): A list of commands needed to use the Source |

### `parameters`

Any key is accepted here - See the [Parameters](Parameters) section for more
info.

Keys defined here can be used as [Replacements](Parameters#-replacements) for
values within [`meta.command`](#command)

## Command

Since OctoType uses Rust's `std::process::Command` to execute it's Sources, a
list of arguments are needed.

Do this: `["cat", "myfile.txt"]`

Not this: `["cat myfile.txt"]`

Piping and other operators are not supported as of right now. If these are
needed then you must make a script as of now:

```toml
command = ["cat", "myfile.txt", "|", "head"] # Would fail
```

Instead, we put it in a script:

```sh
# File: myscript.sh
cat myfile.txt | head
```

And the supply the command:

```
command = ["/path/to/myscript.sh"]
```

### Output

The output can currently only accept one type: `"default"`, which expects words
seperated by whitespace - Any ascii-whitespace (newlines, tabs, single/multiple
spaces, etc.) will be translated to a single `<space>`

Example:

```
the quick  brown 
            fox
```

Translates to:

```
the quick brown fox
```
