# Beanru - beancount read-modity-write


[![Crates.io](https://img.shields.io/crates/v/beanru)](https://crates.io/crates/beanru)
[![docs.rs](https://img.shields.io/docsrs/beanru?color=blue&label=docs.rs)](https://docs.rs/beanru)

WARNING: This crate is still in early stages and APIs will change.

The goal of this crate (binary) is to make read-modify-write scripts for
beancount files easy.

Limitations:

* imports are not supported (only single file beancounts are supported)
* the formatting is changed and comments are dropped

## Install

```shell
cargo install beanru
```

## Examples

First, normalize your beancount file (reformat and drop comments):

```shell
beanru normalize -i ledge.beancount
```

WARNING: I strongly recommend looking at the changes that the script made, to
ensure that no important syntax was dropped. I like to use `git diff -w` to
ignore whitespaces. This crate uses different parser than official one, so
ensure no postings or directives were dropped. If you notice any bug, please
file an issue.

Now, the modification scripts can be run.

TODO: provide examples.


## TODO

APIs:

* Figure out if we should make all fields in types public, or if instead we
  should provide getters and setters for all.
* Unify some type names (like currency and commodity).

Features:

* Provide examples of using `rust-script` to make it very simple to write small
  scripts that perform some modification on a beancount file.
* Wasm support and examples.
