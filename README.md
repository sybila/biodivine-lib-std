[![Crates.io](https://img.shields.io/crates/v/biodivine-lib-std?style=flat-square)](https://crates.io/crates/biodivine-lib-std)
[![Api Docs](https://img.shields.io/badge/docs-api-yellowgreen?style=flat-square)](https://docs.rs/biodivine-lib-std/<version>/)
[![Travis (.org)](https://img.shields.io/travis/sybila/biodivine-lib-std?style=flat-square)](https://travis-ci.org/sybila/biodivine-lib-std)
[![Codecov](https://img.shields.io/codecov/c/github/sybila/biodivine-lib-std?style=flat-square)](https://codecov.io/gh/sybila/biodivine-lib-std)
[![GitHub issues](https://img.shields.io/github/issues/sybila/biodivine-lib-std?style=flat-square)](https://github.com/sybila/biodivine-lib-std/issues)
[![Dev Docs](https://img.shields.io/badge/docs-dev-orange?style=flat-square)](https://biodivine.fi.muni.cz/docs/biodivine-lib-std/v<version>/)
[![GitHub last commit](https://img.shields.io/github/last-commit/sybila/biodivine-lib-std?style=flat-square)](https://github.com/sybila/biodivine-lib-std/commits/master)
[![Crates.io](https://img.shields.io/crates/l/biodivine-lib-std?style=flat-square)](https://github.com/sybila/biodivine-lib-std/blob/master/LICENSE)
[![GitHub top language](https://img.shields.io/github/languages/top/sybila/biodivine-lib-std?style=flat-square)](https://github.com/sybila/biodivine-lib-std)

## Biodivine Standard Library

Biodivine standard library defines some basic traits and utility methods which often arise when dealing with models
from computational biology and when writing formal verification tools in general.

### TODO: API Overview pending...

### Set-like objects

Because Rust currently does not have a `Set` trait (and even if it had, its use case would probably differ from outs),
we introduce our own `Set` trait. A struct that implements `Set` is assumed to hold a collection of values, however
the collection itself does not have to be explicit. Items of a `Set` can be even uncountable:

### TODO: Finish template. 

This is a template project for a general Rust based Biodivine library. It comes with a few useful features pre-enabled. 

Provided features:
 - Travis integration pre-configured with Codecov code coverage.
 - `LICENSE` and `.gitignore` files.
 - Run `cargo make rich-doc` and `cargo make rich-doc-dev` to generate documentation with Mermaid and KaTeX enabled (`dev` variant includes also internal functions).
 - Run `cargo make` to run standard test process and compile basic docs, but also run automatic formatting tool of source code (make sure you apply formatting every time before commit).
 - There is a `shields_up` feature flag that can be used to include extra safety checks (invariants, pre-/post-conditions) that should not be needed but may be useful for testing. For usage examples, see the Biodivine Rust developer guide. The flag is off by default, but is enabled during tests on Travis. We provide variants of basic commands with the 
 
To fully initialize the template, perform the following steps:

 - In `Cargo.toml`, specify package name, author, dependencies (if needed, use dev-dependencies for dependencies used only for tests).
 - Enable continuous builds on Travis and code coverage on Codecov. Remember to set Codecov token environment variable to enable coverage reports.
 - Rewrite this readme (including shield urls) :) 
 - Add basic info and shilds to the root readme of the biodivine repo.