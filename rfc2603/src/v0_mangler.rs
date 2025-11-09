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
        let m = V0Mangler::new();
        assert_eq!(m.out, "_R");
        assert_eq!(m.start_offset, 2);
        assert_eq!(m.paths.len(), 0);
    }

    #[test]
    fn test_default_mangler() {
        let m = V0Mangler::default();
        assert_eq!(m.out, "_R");
        assert_eq!(m.start_offset, 2);
    }

    #[test]
    fn test_push() {
        let mut m = V0Mangler::new();
        m.push("Nv");
        assert_eq!(m.out, "_RNv");
        m.push("C7mycrate");
        assert_eq!(m.out, "_RNvC7mycrate");
    }

    #[test]
    fn test_push_integer_62() {
        let mut m = V0Mangler::new();
        m.push_integer_62(0);
        assert_eq!(m.out, "_R_");

        let mut m = V0Mangler::new();
        m.push_integer_62(1);
        assert_eq!(m.out, "_R0_");

        let mut m = V0Mangler::new();
        m.push_integer_62(62);
        assert_eq!(m.out, "_RZ_");
    }

    #[test]
    fn test_push_disambiguator() {
        let mut m = V0Mangler::new();
        m.push_disambiguator(0);
        assert_eq!(m.out, "_R");

        let mut m = V0Mangler::new();
        m.push_disambiguator(1);
        assert_eq!(m.out, "_Rs_");

        let mut m = V0Mangler::new();
        m.push_disambiguator(10);
        // 10 - 1 = 9, then 9 - 1 = 8, 8 in base-62 = "8"
        assert_eq!(m.out, "_Rs8_");
    }

    #[test]
    fn test_push_ident() {
        let mut m = V0Mangler::new();
        m.push_ident("foo");
        assert_eq!(m.out, "_R3foo");

        let mut m = V0Mangler::new();
        m.push_ident("_bar");
        assert_eq!(m.out, "_R4__bar");
    }

    #[test]
    fn test_path_caching() {
        let mut m = V0Mangler::new();

        // First time - should return false and cache position
        assert!(!m.try_cache_path("test::path"));
        m.push("C7mycrate");  // Emit some content

        // Second time - should return true and emit backref
        let backref_pos = m.out.len();
        assert!(m.try_cache_path("test::path"));

        // Should have emitted B + offset
        assert!(m.out[backref_pos..].starts_with("B"));
    }

    #[test]
    fn test_path_caching_multiple_paths() {
        let mut m = V0Mangler::new();

        // Cache first path
        assert!(!m.try_cache_path("path1"));
        m.push("C7mycrate");

        // Cache second path
        assert!(!m.try_cache_path("path2"));
        m.push("Nt6module");
        let len_after_path2 = m.out.len();

        // Reference first path
        assert!(m.try_cache_path("path1"));
        assert!(m.out[len_after_path2..].starts_with("B"));

        // Reference second path
        assert!(m.try_cache_path("path2"));
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

    #[test]
    fn test_path_append_ns_with_disambiguator() {
        let mut m = V0Mangler::new();
        m.path_append_ns(
            |m| m.push("C7mycrate"),
            't',
            1,
            "module"
        );
        // Should be: _R + NtC7mycrates_6module
        assert_eq!(m.out, "_RNtC7mycrates_6module");
    }

    #[test]
    fn test_path_append_ns_multiple() {
        let mut m = V0Mangler::new();
        m.push("C7mycrate");
        m.path_append_ns(
            |_| {},
            't',
            0,
            "module"
        );
        m.path_append_ns(
            |_| {},
            'v',
            0,
            "func"
        );
        // Should build nested path
        assert_eq!(m.out, "_RC7mycrateNt6moduleNv4func");
    }

    #[test]
    fn test_print_backref() {
        let mut m = V0Mangler::new();
        m.push("C7mycrate");
        let cached_pos = m.out.len() - 2; // Point somewhere in the middle

        m.print_backref(cached_pos);
        // Should emit B + base62(cached_pos - start_offset)
        // cached_pos = 9, start_offset = 2, so 9-2 = 7
        assert!(m.out.contains("B"));
    }

    #[test]
    fn test_backref_offset_calculation() {
        let mut m = V0Mangler::new();
        // start_offset = 2 (length of "_R")

        m.push("C7mycrate");
        // Position 2: C
        // Backref from position 2 should give offset 0
        m.print_backref(2);
        assert!(m.out.ends_with("B_")); // offset 0 encodes as "_"
    }

    #[test]
    fn test_complex_symbol_building() {
        let mut m = V0Mangler::new();

        // Build: _RNvNtC7mycrate6module3foo
        m.push("Nv");
        m.path_append_ns(
            |m| m.push("C7mycrate"),
            't',
            0,
            "module"
        );
        m.push_ident("foo");

        assert_eq!(m.out, "_RNvNtC7mycrate6module3foo");
    }

    #[test]
    fn test_mangler_with_hash() {
        let mut m = V0Mangler::new();
        m.push("NvCsABC123_7mycrate3foo");
        assert_eq!(m.out, "_RNvCsABC123_7mycrate3foo");
    }

    #[test]
    fn test_path_cache_records_position() {
        let mut m = V0Mangler::new();

        m.push("C7mycrate");
        let pos_before = m.out.len();

        assert!(!m.try_cache_path("test_key"));

        // Should have cached the current position
        assert_eq!(m.paths.get("test_key"), Some(&pos_before));
    }

    #[test]
    fn test_multiple_backrefs_to_same_path() {
        let mut m = V0Mangler::new();

        // Cache a path
        assert!(!m.try_cache_path("shared_path"));
        m.push("C7mycrate");

        // Reference it multiple times
        let pos1 = m.out.len();
        assert!(m.try_cache_path("shared_path"));

        let pos2 = m.out.len();
        assert!(m.try_cache_path("shared_path"));

        // Both should have emitted backrefs
        assert_ne!(pos1, pos2); // Length changed
    }

    #[test]
    fn test_namespace_characters() {
        let mut m = V0Mangler::new();

        // Test different namespace characters
        m.path_append_ns(|m| m.push("C3std"), 'v', 0, "foo");
        assert!(m.out.contains("Nv"));

        let mut m = V0Mangler::new();
        m.path_append_ns(|m| m.push("C3std"), 't', 0, "String");
        assert!(m.out.contains("Nt"));

        let mut m = V0Mangler::new();
        m.path_append_ns(|m| m.push("C3std"), 'C', 0, "closure");
        assert!(m.out.contains("NC"));
    }

    #[test]
    fn test_empty_prefix_callback() {
        let mut m = V0Mangler::new();
        m.path_append_ns(
            |_| {}, // Empty prefix
            'v',
            0,
            "foo"
        );
        assert_eq!(m.out, "_RNv3foo");
    }

    #[test]
    fn test_chained_path_building() {
        let mut m = V0Mangler::new();

        m.push("Nv");
        m.push("Nt");
        m.push("C7mycrate");
        m.push_ident("module");
        m.push_ident("func");

        // Should create nested structure
        assert!(m.out.starts_with("_RNvNt"));
    }
}
