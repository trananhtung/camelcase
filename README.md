# camelcase

[![crates.io](https://img.shields.io/crates/v/camelcase.svg)](https://crates.io/crates/camelcase)
[![docs.rs](https://docs.rs/camelcase/badge.svg)](https://docs.rs/camelcase)
[![CI](https://github.com/trananhtung/camelcase/actions/workflows/ci.yml/badge.svg)](https://github.com/trananhtung/camelcase/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/camelcase.svg)](#license)

**Convert a string to `camelCase`.**

`foo-bar` → `fooBar`, `foo_bar_baz` → `fooBarBaz`, `XML-http` → `xmlHttp`. Handles dash,
dot, underscore, and space separators, as well as existing camelCase/PascalCase and
acronyms. A faithful Rust port of the widely-used
[`camelcase`](https://www.npmjs.com/package/camelcase) npm package.

It is the inverse of [`decamelize`](https://crates.io/crates/decamelize).

- `PascalCase`, `preserve_consecutive_uppercase`, and `capitalize_after_number` options
- Differential-tested against the reference `camelcase` implementation

## Install

```toml
[dependencies]
camelcase = "0.1"
```

## Usage

```rust
use camelcase::{camel_case, camel_case_with, camel_case_slice, Options};

assert_eq!(camel_case("foo-bar"), "fooBar");
assert_eq!(camel_case("foo_bar_baz"), "fooBarBaz");
assert_eq!(camel_case("XML-http"), "xmlHttp");
assert_eq!(camel_case("p2p network"), "p2pNetwork");
assert_eq!(camel_case("foo123bar"), "foo123Bar");
assert_eq!(camel_case("__foo__bar__"), "__fooBar"); // leading _/$ preserved

// PascalCase:
assert_eq!(camel_case_with("foo-bar", Options::new().pascal_case(true)), "FooBar");

// Preserve consecutive uppercase runs:
assert_eq!(
    camel_case_with("XML-http", Options::new().preserve_consecutive_uppercase(true)),
    "XMLHttp"
);

// From a list of fragments:
assert_eq!(camel_case_slice(&["foo", "bar"], Options::default()), "fooBar");
```

## Options

| Option                            | Default | Effect                                                          |
| --------------------------------- | ------- | -------------------------------------------------------------- |
| `pascal_case`                     | `false` | Uppercase the first character (`FooBar`).                      |
| `preserve_consecutive_uppercase`  | `false` | Keep runs of uppercase letters (`XMLHttp` instead of `xmlHttp`). |
| `capitalize_after_number`         | `true`  | Treat a digit run as a word boundary (`foo123Bar`).            |

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
