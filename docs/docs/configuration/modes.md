# 🎭 Modes

A "Mode" in OctoType is what describes certain conditions to be met before a
session can end, and also some rules about what are allowed during a session.

Currently, this includes how many words have been typed, and how much time has
elapsed.

Modes are loaded from `<OCTOTYPE_CONFIG_DIR>/modes/<mode>.toml`.

Official maintained modes can be found
[in the repo](https://github.com/mahlquistj/octotype/tree/main/modes)

## File structure

```toml
[meta]
name = "My Mode"
description = "My custom Mode!"
# .. Other metadata

[parameters]
# .. Parameters the user can customize for this mode

[conditions]
# .. Conditons/rules of the session

[overrides."<source_name>"]
# .. Parameter overrides for the specified source
```

## Options

### `meta`

| option          | type       | description                                                             |
| --------------- | ---------- | ----------------------------------------------------------------------- |
| name            | `String`   | The name of the Mode                                                    |
| description     | `String`   | A description of the Mode                                               |
| allowed_sources | `[String]` | Optional: Names of allowed [Sources](Sources) to be used with this mode |

### `parameters`

Any key is accepted here - See the [Parameters](Parameters) section for more
info.

Keys defined here can be used as [Replacements](Parameters#-replacements) for
values within [`conditions`](#conditions) and
[`overrides`](#overridessourcename)

### `conditions`

| option          | type                    | description                                                                                      |
| --------------- | ----------------------- | ------------------------------------------------------------------------------------------------ |
| time            | `int` or `Replacement`  | Optional: The max time allowed (in seconds)                                                      |
| words_typed     | `int` or `Replacement`  | Optional: The amount of completed words needed                                                   |
| allow_deletions | `bool` or `Replacement` | Optional (Defaults to `true`): Wether to allow the user to delete characters while typing        |
| allow_errors    | `bool` or `Replacement` | Optional (Defaults to `true`): Wether the session should end if the user types a character wrong |

### `overrides."<source_name>"`

Takes any key (name of source parameter) and a value of `String` or
`Replacement` that will act as an override of any parameters of the specified
source.
