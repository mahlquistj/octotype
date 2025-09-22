---
title: Development
---

import Link from '@docusaurus/Link';

Development of OctoType is pretty simple, and shouldn't require any obscure
dependencies.

If you use nix, a devshell is provided for development within the `flake.nix` of
the repository. It can be started with (Reuires flakes enabled):

```sh
nix develop . --command "<your shell here>"
```

## Contributing

If you want to contribute to OctoType you can do so by forking the repository,
making your changes, and making a pull-request.

If you haven't done this before, you can follow the below instructions to get
started, or head directly to [Environment](environment) to get set up for
development.

### Creating a fork

Start by [forking the repository](https://github.com/mahlquistj/octotype/fork).

You now have your own copy of the repository! You can clone it to your own
machine, edit the code, and commit to your own branch!

### Install tools

Start by installing the required tools, and setting up your environment (See how
to do so [here](environment))

### Start coding

Build out your feature/bugfix and test it, commit your changes, and so forth.

### Submitting a Pull Request

To submit a pull request, simply press the **Contribute** button in github:

![contribut-btn](/img/contribute-btn.png)

And the press "Open Pull-request".

When editing the pull request, make sure to write a concise title that describes
the changes. The description should serve as a more in-depth explaination of
what you've changed (And why!).

Once done, you can submit the pull request.

At some point you will either recieve a review, or a comment from a reviewer
(see [Expectations](#expectations)).

If all is good, the reviewer will start the CI (Automated testing), and make
sure your code compiles.

If something goes wrong, you need to go back and take care of the issues.

## Expectations

The below are some expectations that must be set as this is an open source
project:

### All code is treated as your own

When submitting a pull-request, it is expected that you understand your code
enough to make the proper changes asked by the reviewer, if any.

If code is submitted where the submitter is not being able to edit or debug it
if any bugs or disagreements are found, it won't get merged. It is not up to the
reviewer to correct these issues.

That being said doesn't mean that the reviewer can't help, but it's not their
responsibility. Discussions and asking for help is encouraged - The point is
just that there is no help to get if the reviewer is the only one who
understands the code submitted.

### Don't _expect_ your changes to be merged

All changes submitted, might not make it into the codebase. This could be
because of disagreements, not being in scope, regressions, etc.

### Be patient

Reviews might take a while. I'm only one person working on this at the moment.
