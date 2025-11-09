# RFC 2603 - Rust Symbol Name Mangling v0

This crate provides a standalone implementation of the Rust v0 symbol name mangling scheme as specified in [RFC 2603](https://rust-lang.github.io/rfcs/2603-rust-symbol-name-mangling-v0.html).

## Overview

The v0 mangling format is used by the Rust compiler to generate unique symbol names for functions, types, and other entities. This crate extracts the core mangling logic from the Rust compiler into a reusable library.

## Features

- **High-level API** for encoding complete paths and symbols
- **Base-62 encoding** for compact number representation
- **Identifier encoding** with Punycode support for Unicode
- **Type and path mangling** utilities
- Standalone - no rustc dependencies

## Usage

### High-Level API (Recommended)

```rust
use rfc2603::{encode_symbol, encode_simple_path, encode_crate_root, Namespace};

// Encode a crate root
let crate_root = encode_crate_root("mycrate", 0);
// Output: "C7mycrate"

// Encode a path: mycrate::module::function
let path = encode_simple_path(&[
    ("mycrate", Namespace::Crate, 0),
    ("module", Namespace::Type, 0),
    ("function", Namespace::Value, 0),
]);
// Output: "NvNtC7mycrate6module8function"

// Encode a complete symbol with _R prefix
let symbol = encode_symbol(&path);
// Output: "_RNvNtC7mycrate6module8function"
```

### Low-Level API

For advanced use cases where you need fine-grained control:

```rust
use rfc2603::{push_integer_62, push_ident, push_disambiguator};

let mut output = String::new();

// Encode numbers in base-62
push_integer_62(42, &mut output); // Outputs: "f_"

// Encode identifiers
push_ident("example", &mut output); // Outputs: "7example"

// Encode Unicode identifiers (uses Punycode)
push_ident("café", &mut output); // Outputs: "u6caf_1ga"
```

## Examples

See the `examples/` directory for more detailed usage:

```bash
cargo run --example basic_usage
```

## Implementation

This crate contains the core algorithms extracted from `compiler/rustc_symbol_mangling/src/v0.rs` in the Rust compiler repository. It provides a clean, standalone API without rustc dependencies.

## Testing

The crate includes comprehensive tests using real symbol names from compiled Rust code:

```bash
cargo test
```

## License

Dual-licensed under MIT or Apache-2.0.
