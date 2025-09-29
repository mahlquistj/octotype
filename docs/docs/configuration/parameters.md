---
sidebar_position: 3
---

# ⚙️ Parameters

[Sources](Sources) and [Modes](Modes) both have a `parameters` section where you
can define arbritrary keys.

These parameters can be edited by the user upon selection and can be used to
customize the command that is executed by the [Source](Sources)

## Types

### Range

A range of numbers.

```toml
"<param>" = {
    min = 10,     # Optional - Defaults to 0
    max = 100,    # Optional - Defaults to 9223372036854775807 (essentially unbounded)
    step = 10,    # Optional - Defaults to 1
    default = 50, # Optional - Defaults to the value of `min`
}
```

Even though every key is optional, at least `min` or `max` must be defined.

### Selection

A list of selectable value.

```toml
"<param>" = {
    options = ["A", "B", "C"], # Required (Can't be empty),
    default = "C",             # Optional - Defaults to the first value in `options`
}
```

### Boolean

A boolean.

```toml
"<param>" = true
```

### Int

An integer - This cannot be changed by the user (Use [Range](#range) instead, if
needed)

```toml
"<param>" = 42
```

### String

A string - This cannot be changed by the user (Use [Selection](#selection)
instead, if needed)

```toml
"<param>" = "some_string"
```

## Replacements

Parameters exist for one reason only: To make an interface for the user to
customize the default values of a [Source](Sources).

Every parameter can be used as a replacement in certain places of a
[Source](Sources#parameters) or [Mode](Modes#parameters).

A Replacement is defined by a `String` which looks like so:
`"{<parameter_key>}"` - This will translate into the final value of the
`parameter_key`

An example of how this is used would look like so:

```toml
# File: <OCTOTYPE_CONFIG_DIR>/sources/programming.toml
[meta]
name = "Programming"
description = "Generates common code-patterns from some languages"
command = [
    "./programming-generator.py",
    "--lang", "{language}", 
    "--mode", "{mode}", 
    "--amount", "{amount}",
    "--function-lines", "{function_lines}"
]
required_tools = ["python3"]

[parameters]
language = {
    options = ["Python", "C", "Rust"]
}
mode = {
    options = ["Words", "Functions"]
}
amount = {
    min: 1
}
# Set the default function-lines for the "Functions" mode to be 3
# This value is never shown to the user, but can be replaced by a Mode
function_lines = 3
```

In our mode we can do the following:

```toml
# File: <OCTOTYPE_CONFIG_DIR>/modes/rust-function-sprint.toml
[meta]
name = "Function sprint"
description = "Complete functions within a time limit"
allowed_sources = ["Programming"]

[parameters]
# A range for how many functions the user wants in total
functions_amount = {
    min = 1,
    default = 3
}
# A range for how many lines we want in each function
function_lines = {
    min = 3,
    default = 5
}
# Another range for how much time they have to type all the words
time_limit = {
    min = 10
    step = 5
    default = 30
}

# Override some parameters in the "Programming" source parameters
[overrides."Programming"]
mode = "Functions"
amount = "{functions_amount}" # Replace with this modes parameter
function_lines = "{function_lines}" # Replace with this modes parameter
```

Now, if the the `Rust function sprint` mode is chosen, it would present the
following options to be changed before starting the session:

```
functions_amount: 3
function_lines: 5
time_limit: 10
language: "Python"
```
