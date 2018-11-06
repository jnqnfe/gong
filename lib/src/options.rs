// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! "Available" option sets

use std::convert::AsRef;

#[deprecated(since = "1.2.0", note = "Use either `OptionSet` or `OptionSetEx` now, as applicable")]
pub type Options<'a> = OptionSetEx<'a>;

/// Default abbreviation support state
pub(crate) const ABBR_SUP_DEFAULT: bool = true;
/// Default mode
pub(crate) const MODE_DEFAULT: OptionsMode = OptionsMode::Standard;

/// Extendible option set
///
/// Used to supply the set of information about available options to match against
///
/// This is the "extendible" variant which uses `Vec`s to hold the option lists and thus is flexible
/// in allowing addition of options, and may re-allocate as necessary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionSetEx<'a> {
    /* NOTE: these have been left public to allow creation via macros */
    pub long: Vec<LongOption<'a>>,
    pub short: Vec<ShortOption>,
    pub mode: OptionsMode,
    pub allow_abbreviations: bool,
}

impl<'a> Default for OptionSetEx<'a> {
    fn default() -> Self {
        OptionSetEx::new(0, 0)
    }
}

/// Option set
///
/// Used to supply the set of information about available options to match against
///
/// This is the non-"extendible" variant. Unlike its cousin `OptionSetEx`, this holds options lists
/// as slice references rather than `Vec`s, and thus cannot be extended in size (hence no `add_*`
/// methods). This is particularly useful in efficient creation of static/const option sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionSet<'r, 'a: 'r> {
    /* NOTE: these have been left public to allow efficient static creation of options */
    pub long: &'r [LongOption<'a>],
    pub short: &'r [ShortOption],
    pub mode: OptionsMode,
    pub allow_abbreviations: bool,
}

impl<'r, 'a: 'r> PartialEq<OptionSet<'r, 'a>> for OptionSetEx<'a> {
    fn eq(&self, rhs: &OptionSet<'r, 'a>) -> bool {
        rhs.eq(&self.as_fixed())
    }
}

impl<'r, 'a: 'r> PartialEq<OptionSetEx<'a>> for OptionSet<'r, 'a> {
    fn eq(&self, rhs: &OptionSetEx<'a>) -> bool {
        self.eq(&rhs.as_fixed())
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

/// Description of a validation issue within an option in an [`OptionSet`](struct.OptionSet.html) or
/// [`OptionSetEx`](struct.OptionSetEx.html) set.
#[derive(Debug, PartialEq, Eq)]
pub enum OptionFlaw<'a> {
    /// Long option name is empty string
    LongEmpty,
    /// Long option name contains equals
    LongIncludesEquals(&'a str),
    /// Short option char is dash (`-`)
    ShortDash,
    /// Duplicate short option found
    ShortDup(char),
    /// Duplicate long option found
    LongDup(&'a str),
}

impl<'a> OptionSetEx<'a> {
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

    /// Create an [`OptionSet`](struct.OptionSet.html) referencing `self`'s vectors as slices.
    pub fn as_fixed(&self) -> OptionSet<'_, 'a> {
        OptionSet {
            long: &self.long[..],
            short: &self.short[..],
            mode: self.mode,
            allow_abbreviations: self.allow_abbreviations,
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

    /// Add an existing (ready-made) long option
    pub fn add_existing_long(&mut self, long: LongOption<'a>) -> &mut Self {
        self.long.push(long);
        self
    }

    /// Add an existing (ready-made) short option
    pub fn add_existing_short(&mut self, short: ShortOption) -> &mut Self {
        self.short.push(short);
        self
    }

    /// Checks validity of option set
    ///
    /// Returns `true` if valid.
    ///
    /// See also the [`validate`](#method.validate) method.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        validation::validate_set(&self.as_fixed()).is_ok()
    }

    /// Checks validity of option set, returning details of any problems
    #[inline(always)]
    pub fn validate(&self) -> Result<(), Vec<OptionFlaw<'a>>> {
        validation::validate_set(&self.as_fixed())
    }

    /// Analyses provided program arguments.
    ///
    /// This is the same as calling the `process` function directly.
    ///
    /// Returns a result set describing the result of the analysis. This may include `&str`
    /// references to strings provided in the `args` parameter and in `self`. Take note of this with
    /// respect to object lifetimes.
    ///
    /// Expects `self` to be valid (see [`is_valid`](#method.is_valid)).
    pub fn process<T>(&self, args: &'a [T]) -> super::analysis::Analysis<'a>
        where T: AsRef<str>
    {
        super::engine::process(args, &self.as_fixed())
    }
}

impl<'r, 'a: 'r> OptionSet<'r, 'a> {
    /// Creates an 'extendible' copy of `self`
    ///
    /// This duplicates the options in `self` into an [`OptionSetEx`](struct.OptionSetEx.html).
    pub fn to_extendible(&self) -> OptionSetEx<'a> {
        OptionSetEx {
            long: self.long.iter().cloned().collect(),
            short: self.short.iter().cloned().collect(),
            mode: self.mode,
            allow_abbreviations: self.allow_abbreviations,
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

    /// Checks validity of option set
    ///
    /// Returns `true` if valid.
    ///
    /// See also the [`validate`](#method.validate) method.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        validation::validate_set(self).is_ok()
    }

    /// Checks validity of option set, returning details of any problems
    #[inline(always)]
    pub fn validate(&'r self) -> Result<(), Vec<OptionFlaw<'a>>> {
        validation::validate_set(self)
    }

    /// Analyses provided program arguments.
    ///
    /// This is the same as calling the `process` function directly.
    ///
    /// Returns a result set describing the result of the analysis. This may include `&str`
    /// references to strings provided in the `args` parameter and in `self`. Take note of this with
    /// respect to object lifetimes.
    ///
    /// Expects `self` to be valid (see [`is_valid`](#method.is_valid)).
    pub fn process<T>(&self, args: &'a [T]) -> super::analysis::Analysis<'a>
        where T: AsRef<str>
    {
        super::engine::process(args, self)
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

impl ShortOption {
    /// Create a new short option descriptor
    ///
    /// Panics (debug only) if the given char is `-`.
    fn new(ch: char, expects_data: bool) -> Self {
        debug_assert_ne!('-', ch, "Dash ('-') is not a valid short option!");
        Self { ch, expects_data, }
    }
}

/// Option set validation
mod validation {
    use super::{OptionSet, OptionFlaw};

    /// Checks validity of option set, returning details of any problems
    pub fn validate_set<'r, 'a: 'r>(set: &OptionSet<'r, 'a>
        ) -> Result<(), Vec<OptionFlaw<'a>>>
    {
        let mut flaws = Vec::new();

        for candidate in set.long {
            if candidate.name.len() == 0 {
                flaws.push(OptionFlaw::LongEmpty);
            }
            else if candidate.name.contains('=') {
                flaws.push(OptionFlaw::LongIncludesEquals(candidate.name));
            }
        }

        for candidate in set.short {
            if candidate.ch == '-' {
                flaws.push(OptionFlaw::ShortDash);
            }
        }

        find_duplicates_short(set, &mut flaws);
        find_duplicates_long(set, &mut flaws);

        match flaws.len() {
            0 => Ok(()),
            _ => Err(flaws),
        }
    }

    fn find_duplicates_short<'r, 'a: 'r>(set: &OptionSet<'r, 'a>,
        flaws: &mut Vec<OptionFlaw<'a>>)
    {
        let opts = set.short;
        let mut checked: Vec<char> = Vec::with_capacity(opts.len());

        let mut duplicates = Vec::new();
        for short in opts {
            let ch = short.ch;
            if !duplicates.contains(&OptionFlaw::ShortDup(ch)) {
                match checked.contains(&ch) {
                    true => { duplicates.push(OptionFlaw::ShortDup(ch)); },
                    false =>  { checked.push(ch); },
                }
            }
        }
        if !duplicates.is_empty() {
            flaws.append(&mut duplicates);
        }
    }

    fn find_duplicates_long<'r, 'a: 'r>(set: &OptionSet<'r, 'a>,
        flaws: &mut Vec<OptionFlaw<'a>>)
    {
        let opts = set.long;
        let mut checked: Vec<&'a str> = Vec::with_capacity(opts.len());

        let mut duplicates = Vec::new();
        for long in opts {
            let name = long.name.clone();
            if !duplicates.contains(&OptionFlaw::LongDup(name)) {
                match checked.contains(&name) {
                    true => { duplicates.push(OptionFlaw::LongDup(name)); },
                    false =>  { checked.push(name); },
                }
            }
        }
        if !duplicates.is_empty() {
            flaws.append(&mut duplicates);
        }
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
