// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! "Available" option sets

/// Default abbreviation support state
pub(crate) const ABBR_SUP_DEFAULT: bool = true;
/// Default mode
pub(crate) const MODE_DEFAULT: OptionsMode = OptionsMode::Standard;

/// Used to supply the set of information about available options to match against
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options<'a> {
    //TODO: Idiot! `Vec` cannot be created as a static! Create actual means for static option sets.
    /* NOTE: these have been left public to allow efficient static creation of options */
    pub long: Vec<LongOption<'a>>,
    pub short: Vec<ShortOption>,
    pub mode: OptionsMode,
    pub allow_abbreviations: bool,
}

impl<'a> Default for Options<'a> {
    fn default() -> Self {
        Options::new(0, 0)
    }
}

/// Used to assert which option processing mode to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionsMode {
    /// Standard (default): Short (`-o`) and long (`--foo`) options, with single and double dash
    /// prefixes respectively.
    Standard,
    /// Alternate: Long options only, with single dash prefix.
    Alternate,
}

impl Default for OptionsMode {
    fn default() -> Self {
        MODE_DEFAULT
    }
}

/// Description of an available long option
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LongOption<'a> {
    /* NOTE: these have been left public to allow efficient static creation of options */
    /// Long option name, excluding the `--` prefix
    pub name: &'a str,
    /// Whether option expects a data argument
    pub expects_data: bool,
}

/// Description of an available short option
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortOption {
    /* NOTE: these have been left public to allow efficient static creation of options */
    /// Short option character
    pub ch: char,
    /// Whether option expects a data argument
    pub expects_data: bool,
}

impl<'a> Options<'a> {
    /// Create a new object. Takes estimations of the number of options to expect to be added (for
    /// efficient vector allocation).
    pub fn new(count_long: usize, count_short: usize) -> Self {
        Self {
            long: Vec::with_capacity(count_long),
            short: Vec::with_capacity(count_short),
            mode: MODE_DEFAULT,
            allow_abbreviations: ABBR_SUP_DEFAULT,
        }
    }

    /// Set mode
    pub fn set_mode(&mut self, mode: OptionsMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Enable/disable abbreviated matching
    pub fn set_allow_abbreviations(&mut self, allow: bool) -> &mut Self {
        self.allow_abbreviations = allow;
        self
    }

    /// Add a long option
    ///
    /// Panics (debug only) on invalid name.
    pub fn add_long(&mut self, name: &'a str) -> &mut Self {
        self.long.push(LongOption::new(name, false));
        self
    }

    /// Add a short option
    ///
    /// Panics (debug only) on invalid `char` choice.
    pub fn add_short(&mut self, ch: char) -> &mut Self {
        self.short.push(ShortOption::new(ch, false));
        self
    }

    /// Add a long option that expects data
    ///
    /// Panics (debug only) on invalid name.
    pub fn add_long_data(&mut self, name: &'a str) -> &mut Self {
        self.long.push(LongOption::new(name, true));
        self
    }

    /// Add a short option that expects data
    ///
    /// Panics (debug only) on invalid `char` choice.
    pub fn add_short_data(&mut self, ch: char) -> &mut Self {
        self.short.push(ShortOption::new(ch, true));
        self
    }

    /// Checks validity of option set
    ///
    /// Returns `true` if valid. Outputs details of problems found to `stderr`.
    pub fn is_valid(&self) -> bool {
        let mut valid = true;

        for candidate in &self.long {
            if candidate.name.len() == 0 {
                eprintln!("Long option name cannot be an empty string!");
                valid = false;
            }
            else if candidate.name.contains('=') {
                eprintln!("Long option name cannot contain an '=' character!");
                valid = false;
            }
        }

        for candidate in &self.short {
            if candidate.ch == '-' {
                eprintln!("A dash ('-') is not a valid short option!");
                valid = false;
            }
        }

        if let Some(dups) = self.find_duplicates_short() {
            eprintln!("Duplicate short options were found!\n\
                       Duplicated options: {:?}", dups);
            valid = false;
        }
        if let Some(dups) = self.find_duplicates_long() {
            eprintln!("Duplicate long options were found!\n\
                       Duplicated options: {:?}", dups);
            valid = false;
        }

        valid
    }

    fn find_duplicates_short(&self) -> Option<Vec<char>> {
        let mut short_checked: Vec<char> = Vec::with_capacity(self.short.len());

        let mut short_dupes = Vec::new();
        for short in &self.short {
            let ch = short.ch;
            if !short_dupes.contains(&ch) {
                match short_checked.contains(&ch) {
                    true => { short_dupes.push(ch); },
                    false =>  { short_checked.push(ch); },
                }
            }
        }

        match short_dupes.len() {
            0 => None,
            _ => Some(short_dupes),
        }
    }

    fn find_duplicates_long(&self) -> Option<Vec<&'a str>> {
        let mut long_checked: Vec<&'a str> = Vec::with_capacity(self.long.len());

        let mut long_dupes = Vec::new();
        for long in &self.long {
            let name = long.name.clone();
            if !long_dupes.contains(&name) {
                match long_checked.contains(&name) {
                    true => { long_dupes.push(name); },
                    false =>  { long_checked.push(name); },
                }
            }
        }

        match long_dupes.len() {
            0 => None,
            _ => Some(long_dupes),
        }
    }
}

impl<'a> LongOption<'a> {
    /// Create a new long option descriptor
    ///
    /// Panics (debug only) if the given name contains an `=` or is an empty string.
    fn new(name: &'a str, expects_data: bool) -> Self {
        debug_assert!(name.len() >= 1, "Long option name cannot be an empty string!");
        debug_assert!(!name.contains('='), "Long option name cannot contain '='!");
        Self { name, expects_data, }
    }
}

impl<'a> ShortOption {
    /// Create a new short option descriptor
    ///
    /// Panics (debug only) if the given char is `-`.
    fn new(ch: char, expects_data: bool) -> Self {
        debug_assert_ne!('-', ch, "Dash ('-') is not a valid short option!");
        Self { ch, expects_data, }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /* Dash ('-') is an invalid short option (clashes with early terminator if it were given on its
     * own (`--`), and would be misinterpreted as a long option if given as the first in a short
     * option set (`--abc`)). */

    /// Check `ShortOption::new` rejects '-'
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_short_dash() {
        let _opt = ShortOption::new('-', false); // Should panic here in debug mode!
    }

    /// Check `LongOption::new` rejects empty string
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_long_no_name() {
        let _opt = LongOption::new("", false); // Should panic here in debug mode!
    }

    /* Long option names cannot contain an '=' (used for declaring a data sub-argument in the same
     * argument; if names could contain an '=', as data can, we would not know where to do the
     * split, complicating matching. */

    /// Check `LongOption::new` rejects equals ('=') char
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_long_with_equals() {
        let _opt = LongOption::new("a=b", false); // Should panic here in debug mode!
    }
}