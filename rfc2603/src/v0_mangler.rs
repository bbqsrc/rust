//! v0 Symbol Mangler - adapted from rustc_symbol_mangling/src/v0.rs
//!
//! This is a standalone version that doesn't require rustc internals.
//! It maintains the same structure and backref system as rustc.

use std::collections::HashMap;
use crate::{push_integer_62, push_ident, push_disambiguator};

/// Low-level v0 symbol mangler with backref support (copied from rustc).
///
/// This mimics rustc's V0SymbolMangler but works standalone.
pub struct V0Mangler {
    /// Output string being built
    pub out: String,
    /// Start offset for backrefs (length of "_R" prefix = 2)
    pub start_offset: usize,
    /// Cache of path positions for backreferences
    /// Maps a path key to its byte position in `out`
    pub paths: HashMap<String, usize>,
}

impl V0Mangler {
    /// Create a new mangler with the `_R` prefix
    pub fn new() -> Self {
        let prefix = "_R";
        Self {
            out: String::from(prefix),
            start_offset: prefix.len(),
            paths: HashMap::new(),
        }
    }

    /// Push a string to the output
    pub fn push(&mut self, s: &str) {
        self.out.push_str(s);
    }

    /// Push a base-62 integer (delegates to crate function)
    pub fn push_integer_62(&mut self, x: u64) {
        push_integer_62(x, &mut self.out)
    }

    /// Push a disambiguator (delegates to crate function)
    pub fn push_disambiguator(&mut self, dis: u64) {
        push_disambiguator(dis, &mut self.out)
    }

    /// Push an identifier (delegates to crate function)
    pub fn push_ident(&mut self, ident: &str) {
        push_ident(ident, &mut self.out)
    }

    /// Append a path component with namespace (copied from rustc's path_append_ns)
    ///
    /// Format: N + ns + prefix + disambiguator + name
    pub fn path_append_ns(
        &mut self,
        print_prefix: impl FnOnce(&mut Self),
        ns: char,
        disambiguator: u64,
        name: &str,
    ) {
        self.push("N");
        self.out.push(ns);
        print_prefix(self);
        self.push_disambiguator(disambiguator);
        self.push_ident(name);
    }

    /// Print a backref (copied from rustc's print_backref)
    ///
    /// Format: B + base62(offset)
    pub fn print_backref(&mut self, i: usize) {
        self.push("B");
        self.push_integer_62((i - self.start_offset) as u64);
    }

    /// Try to use a cached path, or record current position for future backref
    ///
    /// Returns true if a backref was emitted, false if caller should emit full path
    pub fn try_cache_path(&mut self, key: &str) -> bool {
        if let Some(&pos) = self.paths.get(key) {
            self.print_backref(pos);
            true
        } else {
            // Record current position for future backrefs
            self.paths.insert(key.to_string(), self.out.len());
            false
        }
    }
}

impl Default for V0Mangler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_mangler() {
        let mut m = V0Mangler::new();
        assert_eq!(m.out, "_R");
        assert_eq!(m.start_offset, 2);
    }

    #[test]
    fn test_path_caching() {
        let mut m = V0Mangler::new();

        // First time - should return false and cache position
        let start_pos = m.out.len();
        assert!(!m.try_cache_path("test::path"));
        m.push("C7mycrate");  // Emit some content

        // Second time - should return true and emit backref
        let backref_pos = m.out.len();
        assert!(m.try_cache_path("test::path"));

        // Should have emitted B + offset
        assert!(m.out[backref_pos..].starts_with("B"));
    }

    #[test]
    fn test_path_append_ns() {
        let mut m = V0Mangler::new();
        m.path_append_ns(
            |m| m.push("C7mycrate"),
            'v',
            0,
            "foo"
        );
        // Should be: _R + NvC7mycrate3foo
        assert_eq!(m.out, "_RNvC7mycrate3foo");
    }
}
