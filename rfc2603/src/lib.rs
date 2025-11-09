//! RFC 2603 - Rust Symbol Name Mangling v0
//!
//! This crate provides a standalone implementation of the core algorithms
//! used in Rust's v0 symbol name mangling scheme.
//!
//! The v0 mangling format is specified in RFC 2603 and is used by the Rust
//! compiler to generate deterministic, platform-independent symbol names.
//!
//! # Features
//!
//! - **Base-62 encoding**: Compact representation of numbers using `0-9`, `a-z`, `A-Z`
//! - **Identifier encoding**: Length-prefixed identifiers with optional Punycode for Unicode
//! - **Punycode support**: Encode Unicode identifiers in ASCII-safe format
//!
//! # Examples
//!
//! ## Base-62 Encoding
//!
//! ```
//! use rfc2603::{push_integer_62, encode_integer_62};
//!
//! // Encode numbers in base-62 format
//! assert_eq!(encode_integer_62(0), "_");
//! assert_eq!(encode_integer_62(1), "0_");
//! assert_eq!(encode_integer_62(62), "Z_");
//! assert_eq!(encode_integer_62(1000), "g7_");
//! ```
//!
//! ## Identifier Encoding
//!
//! ```
//! use rfc2603::push_ident;
//!
//! let mut output = String::new();
//! push_ident("example", &mut output);
//! assert_eq!(output, "7example");
//!
//! let mut output = String::new();
//! push_ident("gödel", &mut output);
//! // Unicode identifiers are encoded with Punycode
//! assert!(output.starts_with("u"));
//! ```

use std::fmt::Write;

/// Push a `_`-terminated base 62 integer, using the format
/// specified in RFC 2603 as `<base-62-number>`, that is:
/// * `x = 0` is encoded as just the `"_"` terminator
/// * `x > 0` is encoded as `x - 1` in base 62, followed by `"_"`,
///   e.g. `1` becomes `"0_"`, `62` becomes `"Z_"`, etc.
///
/// # Examples
///
/// ```
/// use rfc2603::push_integer_62;
///
/// let mut output = String::new();
/// push_integer_62(0, &mut output);
/// assert_eq!(output, "_");
///
/// let mut output = String::new();
/// push_integer_62(1, &mut output);
/// assert_eq!(output, "0_");
///
/// let mut output = String::new();
/// push_integer_62(62, &mut output);
/// assert_eq!(output, "Z_");
/// ```
pub fn push_integer_62(x: u64, output: &mut String) {
    if let Some(x) = x.checked_sub(1) {
        output.push_str(&to_base_62(x));
    }
    output.push('_');
}

/// Encode an integer in base-62 format and return it as a String.
///
/// This is a convenience wrapper around [`push_integer_62`].
///
/// # Examples
///
/// ```
/// use rfc2603::encode_integer_62;
///
/// assert_eq!(encode_integer_62(0), "_");
/// assert_eq!(encode_integer_62(1), "0_");
/// assert_eq!(encode_integer_62(11), "a_");
/// assert_eq!(encode_integer_62(62), "Z_");
/// assert_eq!(encode_integer_62(63), "10_");
/// assert_eq!(encode_integer_62(1000), "g7_");
/// ```
pub fn encode_integer_62(x: u64) -> String {
    let mut output = String::new();
    push_integer_62(x, &mut output);
    output
}

/// Encode an identifier using the v0 mangling scheme.
///
/// Identifiers are encoded as a length-prefixed string. If the identifier
/// contains non-ASCII characters, it is first encoded using Punycode and
/// prefixed with `u`.
///
/// The format is: `[u]<length>[_]<bytes>`
/// - Optional `u` prefix indicates Punycode encoding
/// - `<length>` is the decimal length of the identifier
/// - Optional `_` separator if the identifier starts with a digit or `_`
/// - `<bytes>` is the identifier itself (or Punycode-encoded version)
///
/// # Examples
///
/// ```
/// use rfc2603::push_ident;
///
/// let mut output = String::new();
/// push_ident("example", &mut output);
/// assert_eq!(output, "7example");
///
/// let mut output = String::new();
/// push_ident("_foo", &mut output);
/// assert_eq!(output, "4__foo"); // Note the separator _
///
/// let mut output = String::new();
/// push_ident("0abc", &mut output);
/// assert_eq!(output, "4_0abc"); // Note the separator _
/// ```
pub fn push_ident(ident: &str, output: &mut String) {
    let mut use_punycode = false;
    for b in ident.bytes() {
        match b {
            b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => {}
            0x80..=0xff => use_punycode = true,
            _ => panic!("Invalid byte {} in identifier {:?}", b, ident),
        }
    }

    let punycode_string;
    let ident = if use_punycode {
        output.push('u');

        let mut punycode_bytes = match punycode::encode(ident) {
            Ok(s) => s.into_bytes(),
            Err(()) => panic!("Punycode encoding failed for identifier {:?}", ident),
        };

        // Replace `-` with `_`.
        if let Some(c) = punycode_bytes.iter_mut().rfind(|&&mut c| c == b'-') {
            *c = b'_';
        }

        punycode_string = String::from_utf8(punycode_bytes).unwrap();
        &punycode_string
    } else {
        ident
    };

    let _ = write!(output, "{}", ident.len());

    // Write a separating `_` if necessary (leading digit or `_`).
    if let Some('_' | '0'..='9') = ident.chars().next() {
        output.push('_');
    }

    output.push_str(ident);
}

/// Convert a u64 to base-62 representation.
///
/// Base-62 uses digits 0-9, lowercase a-z, and uppercase A-Z.
/// The mapping is:
/// - 0-9 → 0-9
/// - 10-35 → a-z
/// - 36-61 → A-Z
fn to_base_62(mut x: u64) -> String {
    const BASE_62: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    if x == 0 {
        return String::from("0");
    }

    let mut result = Vec::new();
    while x > 0 {
        result.push(BASE_62[(x % 62) as usize]);
        x /= 62;
    }

    result.reverse();
    String::from_utf8(result).unwrap()
}

/// Push a `_`-terminated base 62 integer with an optional tag prefix.
///
/// * `x = 0` is encoded as `""` (nothing)
/// * `x > 0` is encoded as the `tag` followed by `push_integer_62(x - 1)`
///   e.g. `1` becomes `tag + "_"`, `2` becomes `tag + "0_"`, etc.
///
/// This is commonly used for optional disambiguators in the v0 mangling scheme.
///
/// # Examples
///
/// ```
/// use rfc2603::push_opt_integer_62;
///
/// let mut output = String::new();
/// push_opt_integer_62("s", 0, &mut output);
/// assert_eq!(output, ""); // No output for 0
///
/// let mut output = String::new();
/// push_opt_integer_62("s", 1, &mut output);
/// assert_eq!(output, "s_"); // s + "_" for 1
///
/// let mut output = String::new();
/// push_opt_integer_62("s", 2, &mut output);
/// assert_eq!(output, "s0_"); // s + "0_" for 2
/// ```
pub fn push_opt_integer_62(tag: &str, x: u64, output: &mut String) {
    if let Some(x) = x.checked_sub(1) {
        output.push_str(tag);
        push_integer_62(x, output);
    }
}

/// Push a disambiguator using the `s` tag.
///
/// This is a convenience wrapper around [`push_opt_integer_62`] with tag `"s"`.
/// Disambiguators are used in v0 mangling to distinguish between items that
/// would otherwise have identical paths.
///
/// # Examples
///
/// ```
/// use rfc2603::push_disambiguator;
///
/// let mut output = String::new();
/// push_disambiguator(0, &mut output);
/// assert_eq!(output, ""); // No disambiguator
///
/// let mut output = String::new();
/// push_disambiguator(1, &mut output);
/// assert_eq!(output, "s_"); // First disambiguator
/// ```
pub fn push_disambiguator(dis: u64, output: &mut String) {
    push_opt_integer_62("s", dis, output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base62_encoding() {
        assert_eq!(encode_integer_62(0), "_");
        assert_eq!(encode_integer_62(1), "0_");
        assert_eq!(encode_integer_62(11), "a_");
        assert_eq!(encode_integer_62(62), "Z_");
        assert_eq!(encode_integer_62(63), "10_");
        assert_eq!(encode_integer_62(1000), "g7_");
    }

    #[test]
    fn test_push_integer_62() {
        let mut output = String::new();
        push_integer_62(0, &mut output);
        assert_eq!(output, "_");

        let mut output = String::new();
        push_integer_62(1, &mut output);
        assert_eq!(output, "0_");

        let mut output = String::new();
        push_integer_62(62, &mut output);
        assert_eq!(output, "Z_");
    }

    #[test]
    fn test_push_ident_ascii() {
        let mut output = String::new();
        push_ident("example", &mut output);
        assert_eq!(output, "7example");

        let mut output = String::new();
        push_ident("foo", &mut output);
        assert_eq!(output, "3foo");
    }

    #[test]
    fn test_push_ident_with_separator() {
        let mut output = String::new();
        push_ident("_foo", &mut output);
        assert_eq!(output, "4__foo");

        let mut output = String::new();
        push_ident("0abc", &mut output);
        assert_eq!(output, "4_0abc");
    }

    #[test]
    fn test_push_ident_unicode() {
        let mut output = String::new();
        push_ident("gödel", &mut output);
        // Should be Punycode encoded: "gdel-5qa" -> "gdel_5qa"
        assert_eq!(output, "u8gdel_5qa");

        let mut output = String::new();
        push_ident("föö", &mut output);
        // Punycode: "f-1gaa" -> "f_1gaa"
        assert_eq!(output, "u6f_1gaa");
    }

    #[test]
    fn test_push_opt_integer_62() {
        let mut output = String::new();
        push_opt_integer_62("s", 0, &mut output);
        assert_eq!(output, "");

        let mut output = String::new();
        push_opt_integer_62("s", 1, &mut output);
        assert_eq!(output, "s_");

        let mut output = String::new();
        push_opt_integer_62("s", 2, &mut output);
        assert_eq!(output, "s0_");
    }

    #[test]
    fn test_push_disambiguator() {
        let mut output = String::new();
        push_disambiguator(0, &mut output);
        assert_eq!(output, "");

        let mut output = String::new();
        push_disambiguator(1, &mut output);
        assert_eq!(output, "s_");
    }

    #[test]
    fn test_to_base_62() {
        assert_eq!(to_base_62(0), "0");
        assert_eq!(to_base_62(10), "a");
        assert_eq!(to_base_62(35), "z");
        assert_eq!(to_base_62(36), "A");
        assert_eq!(to_base_62(61), "Z");
        assert_eq!(to_base_62(62), "10");
    }
}
