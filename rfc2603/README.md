# RFC 2603 - Rust Symbol Name Mangling v0

This crate provides a standalone implementation of the Rust v0 symbol name mangling scheme as specified in [RFC 2603](https://rust-lang.github.io/rfcs/2603-rust-symbol-name-mangling-v0.html).

## Overview

The v0 mangling format is used by the Rust compiler to generate unique symbol names for functions, types, and other entities. This crate extracts the core mangling logic from the Rust compiler into a reusable library.

## Features

- Base-62 encoding for compact number representation
- Identifier encoding with Punycode support for Unicode
- Type and path mangling
- Backref compression for shorter symbols

## Usage

```rust
use rfc2603::{push_integer_62, push_ident};

let mut output = String::new();
push_integer_62(42, &mut output);
println!("{}", output); // Prints: "f_"

let mut output = String::new();
push_ident("example", &mut output);
println!("{}", output); // Prints: "7example"
```

## Implementation

This crate contains the core algorithms extracted from `compiler/rustc_symbol_mangling/src/v0.rs` in the Rust compiler repository.
