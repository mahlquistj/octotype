---
title: Environment
---

import Link from '@docusaurus/Link';

Development of OctoType is pretty simple, and shouldn't require any obscure
dependencies. All tools used by me are listed in the [Tools](#tools) section.

## Nix Devshell

If you use nix, a devshell is provided for development within the `flake.nix` of
the repository. It can be started with (Reuires flakes enabled):

```sh
nix develop . --command "<your shell here>"
```

The devshell supplies all dependencies needed to develop on OctoType, with the
addition of a few tools that i prefer.

## `just`

`just` is command runner (Like `make`).

`just` is used to ensure that scripts are streamlined throughout the development
of octotype.

The easiest way to install `just` is through Cargo:

```sh
cargo install just
```

If you don't already have Cargo installed, check the
[Codebase Development](#codebase-development) section for instructions.

You can see how to install just
[here](https://github.com/casey/just?tab=readme-ov-file#installation).

:::note

On **Windows** `just` requires you to have `sh` installed by default. You should
be able to overwrite this overwrite this by setting

`set shell := ["powershell.exe", "-c"]` or `set shell := ["cmd.exe", "/c"]`

in your
[user justfile](https://github.com/casey/just?tab=readme-ov-file#installation),
if you prefer not installing `sh`

:::

To get started using just, you can list the available commands by simply
executing:

```sh
just
```

Or, you can take a look in the `justfile` in the root of the repository.

## Codebase development

All you need for developing on the codebase is the rust "suite" of tools (Cargo,
rustup, etc.) and `cargo-nextest`.

The rust tools can be installed [here](https://rustup.rs/).

And after that, you can install `cargo-nextest` via. Cargo:

```sh
cargo install cargo-nextest --locked
```

You can then use the [`just` scripts](#just) to run, build or test the binary.

## Documentation development

Editing the documentation doesn't **need** any tools, but they can come in handy
for previewing and testing compilation of the page.

If you want to be able to serve the development site of the docs, you need
[NodeJS](https://nodejs.org/en).

Once installed you can go into the `docs` folder in the repository and install
the node dependencies by running:

```sh
npm install
```

Then, to start the development server you need to run:

```sh
npm start # Runs a development preview server with hot-reloading
```

## Other great tools

| Tool                                                      | Description                                       | Installation                 |
| --------------------------------------------------------- | ------------------------------------------------- | ---------------------------- |
| [`bacon`](https://dystroy.org/bacon/)                     | A background code checker for rust                | `cargo install bacon`        |
| [`cargo-expand`](https://github.com/dtolnay/cargo-expand) | Expand rust macros to see what code they generate | `cargo install cargo-expand` |
