// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! “Available” options

#[cfg(feature = "suggestions")]
use strsim;

/// Extendible option set
///
/// Used to supply the set of information about available options to match against
///
/// This is the "extendible" variant which uses `Vec`s to hold the option lists and thus is flexible
/// in allowing addition of options, and may re-allocate as necessary.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct OptionSetEx<'s> {
    /* NOTE: these have been left public to allow creation via macros */
    pub long: Vec<LongOption<'s>>,
    pub short: Vec<ShortOption>,
}

/// Option set
///
/// Used to supply the set of information about available options to match against
///
/// This is the non-“extendible” variant. Unlike its cousin `OptionSetEx`, this holds options lists
/// as slice references rather than `Vec`s, and thus cannot be extended in size (hence no `add_*`
/// methods). This is particularly useful in efficient creation of static/const option sets.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct OptionSet<'r, 's: 'r> {
    /* NOTE: these have been left public to allow efficient static creation of options */
    pub long: &'r [LongOption<'s>],
    pub short: &'r [ShortOption],
}

impl<'r, 's: 'r> PartialEq<OptionSet<'r, 's>> for OptionSetEx<'s> {
    fn eq(&self, rhs: &OptionSet<'r, 's>) -> bool {
        rhs.eq(&self.as_fixed())
    }
}

impl<'r, 's: 'r> PartialEq<OptionSetEx<'s>> for OptionSet<'r, 's> {
    fn eq(&self, rhs: &OptionSetEx<'s>) -> bool {
        self.eq(&rhs.as_fixed())
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
    /// Long option name contains unicode replacement char (`U+FFFD`)
    LongIncludesRepChar(&'a str),
    /// Short option char is dash (`-`)
    ShortDash,
    /// Short option char is unicode replacement char (`U+FFFD`)
    ShortRepChar,
    /// Duplicate short option found
    ShortDup(char),
    /// Duplicate long option found
    LongDup(&'a str),
}

impl<'s> OptionSetEx<'s> {
    /// Create a new object
    ///
    /// You can alternatively use [`with_capacity`](#method.with_capacity) for more efficient `Vec`
    /// creation.
    #[inline(always)]
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new object, with size estimation
    ///
    /// Takes estimations of the number of options to expect to be added (for efficient vector
    /// allocation).
    pub fn with_capacity(long_count_est: usize, short_count_est: usize) -> Self {
        Self {
            long: Vec::with_capacity(long_count_est),
            short: Vec::with_capacity(short_count_est),
        }
    }

    /// Create an [`OptionSet`](struct.OptionSet.html) referencing `self`’s vectors as slices.
    pub fn as_fixed(&self) -> OptionSet<'_, 's> {
        OptionSet {
            long: &self.long[..],
            short: &self.short[..],
        }
    }

    /// Checks if empty
    pub fn is_empty(&self) -> bool {
        self.long.is_empty() && self.short.is_empty()
    }

    /// Add a long option
    ///
    /// Panics (debug only) on invalid name.
    pub fn add_long(&mut self, name: &'s str) -> &mut Self {
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
    pub fn add_long_data(&mut self, name: &'s str) -> &mut Self {
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
    pub fn add_existing_long(&mut self, long: LongOption<'s>) -> &mut Self {
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
        validation::validate_set(&self.as_fixed(), false).is_ok()
    }

    /// Checks validity of option set, returning details of any problems
    #[inline(always)]
    pub fn validate(&self) -> Result<(), Vec<OptionFlaw<'s>>> {
        validation::validate_set(&self.as_fixed(), true)
    }

    /// Find the best matching long option for the given string
    ///
    /// This is intended to be used when an unknown long option is encountered in an analysis, to
    /// give users a hint when displaying the error to them. I.e.:
    ///
    /// > “Error: Unknown option ‘x’, did you mean ‘y’”
    ///
    /// Specifically, this uses the `jaro_winkler` algorithm from the `strsim` crate; It filters
    /// out any options with a metric calculated as less than `0.8`, and returns the first option
    /// with the highest metric.
    #[cfg(feature = "suggestions")]
    #[inline]
    pub fn suggest(&self, unknown: &str) -> Option<&'s str> {
        self.as_fixed().suggest(unknown)
    }
}

impl<'r, 's: 'r> OptionSet<'r, 's> {
    /// Creates an “extendible” copy of `self`
    ///
    /// This duplicates the options in `self` into an [`OptionSetEx`](struct.OptionSetEx.html).
    pub fn to_extendible(&self) -> OptionSetEx<'s> {
        OptionSetEx {
            long: self.long.iter().cloned().collect(),
            short: self.short.iter().cloned().collect(),
        }
    }

    /// Checks if empty
    pub fn is_empty(&self) -> bool {
        self.long.is_empty() && self.short.is_empty()
    }

    /// Checks validity of option set
    ///
    /// Returns `true` if valid.
    ///
    /// See also the [`validate`](#method.validate) method.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        validation::validate_set(self, false).is_ok()
    }

    /// Checks validity of option set, returning details of any problems
    #[inline(always)]
    pub fn validate(&'r self) -> Result<(), Vec<OptionFlaw<'s>>> {
        validation::validate_set(self, true)
    }

    /// Find the best matching long option for the given string
    ///
    /// This is intended to be used when an unknown long option is encountered in an analysis, to
    /// give users a hint when displaying the error to them. I.e.:
    ///
    /// > “Error: Unknown option ‘*x*’, did you mean ‘*y*’?”
    ///
    /// Specifically, this uses the `jaro_winkler` algorithm from the `strsim` crate; It filters
    /// out any options with a metric calculated as less than `0.8`, and returns the first option
    /// with the highest metric.
    #[cfg(feature = "suggestions")]
    pub fn suggest(&self, unknown: &str) -> Option<&'s str> {
        let filter = 0.8;
        let mut best_metric: f64 = filter;
        let mut best: Option<&str> = None;
        for opt in self.long {
            let metric = strsim::jaro_winkler(unknown, opt.name);
            if metric > best_metric || (best.is_none() && metric >= filter) {
                best = Some(opt.name);
                best_metric = metric;
            }
        }
        best
    }
}

impl<'a> LongOption<'a> {
    /// Create a new long option descriptor
    ///
    /// Panics (debug only) if the given name is an empty string, contains an equals ('=') or
    /// contains a unicode replacement character ('\u{FFFD}').
    fn new(name: &'a str, expects_data: bool) -> Self {
        debug_assert!(!name.is_empty(), "Long option name cannot be an empty string!");
        debug_assert!(!name.contains('='), "Long option name cannot contain ‘=’!");
        debug_assert!(!name.contains('\u{FFFD}'), "Long option name cannot contain ‘\\u{FFFD}’!");
        Self { name, expects_data, }
    }
}

impl ShortOption {
    /// Create a new short option descriptor
    ///
    /// Panics (debug only) if the given `char` is a dash ('-') or the unicode replacement character
    /// ('\u{FFFD}').
    fn new(ch: char, expects_data: bool) -> Self {
        debug_assert_ne!('-', ch, "Dash (‘-’) is not a valid short option!");
        debug_assert_ne!('\u{FFFD}', ch, "Unicode replacement char (‘\u{FFFD}’) is not a valid short option!");
        Self { ch, expects_data, }
    }
}

/// Option set validation
pub(crate) mod validation {
    use super::{OptionSet, OptionFlaw};

    /// Checks validity of option set, returning details of any problems
    ///
    /// If no problems are found, it returns `Ok(())`, otherwise `Err(_)`.
    ///
    /// If `detail` is `false`, it returns early on encountering a problem (with an empty `Vec`),
    /// useful for quick `is_valid` checks. Otherwise it builds up and provides a complete list of
    /// flaws.
    pub fn validate_set<'r, 's: 'r>(set: &OptionSet<'r, 's>, detail: bool)
        -> Result<(), Vec<OptionFlaw<'s>>>
    {
        let mut flaws: Vec<OptionFlaw<'s>> = Vec::new();

        for candidate in set.long {
            if candidate.name.is_empty() {
                match detail {
                    true => { flaws.push(OptionFlaw::LongEmpty); },
                    false => { return Err(flaws); },
                }
            }
            else if candidate.name.contains('=') {
                match detail {
                    true => { flaws.push(OptionFlaw::LongIncludesEquals(candidate.name)); },
                    false => { return Err(flaws); },
                }
            }
            else if candidate.name.contains('\u{FFFD}') {
                match detail {
                    true => { flaws.push(OptionFlaw::LongIncludesRepChar(candidate.name)); },
                    false => { return Err(flaws); },
                }
            }
        }

        for candidate in set.short {
            if candidate.ch == '-' {
                match detail {
                    true => { flaws.push(OptionFlaw::ShortDash); },
                    false => { return Err(flaws); },
                }
            }
            else if candidate.ch == '\u{FFFD}' {
                match detail {
                    true => { flaws.push(OptionFlaw::ShortRepChar); },
                    false => { return Err(flaws); },
                }
            }
        }

        let mut dupes: bool = false;
        find_duplicates_short(set, &mut flaws, detail, &mut dupes);
        if !detail && dupes {
            return Err(flaws);
        }
        find_duplicates_long(set, &mut flaws, detail, &mut dupes);
        if !detail && dupes {
            return Err(flaws);
        }

        match flaws.is_empty() {
            true => Ok(()),
            false => Err(flaws),
        }
    }

    fn find_duplicates_short<'r, 's: 'r>(set: &OptionSet<'r, 's>, flaws: &mut Vec<OptionFlaw<'s>>,
        detail: bool, found: &mut bool)
    {
        let opts = set.short;
        let mut checked: Vec<char> = Vec::with_capacity(opts.len());

        let mut duplicates = Vec::new();
        for short in opts {
            let ch = short.ch;
            if !duplicates.contains(&OptionFlaw::ShortDup(ch)) {
                match checked.contains(&ch) {
                    true => {
                        match detail {
                            true => { duplicates.push(OptionFlaw::ShortDup(ch)); },
                            false => { *found = true; return; },
                        }
                    },
                    false => { checked.push(ch); },
                }
            }
        }
        if !duplicates.is_empty() {
            flaws.append(&mut duplicates);
        }
    }

    fn find_duplicates_long<'r, 's: 'r>(set: &OptionSet<'r, 's>, flaws: &mut Vec<OptionFlaw<'s>>,
        detail: bool, found: &mut bool)
    {
        let opts = set.long;
        let mut checked: Vec<&'s str> = Vec::with_capacity(opts.len());

        let mut duplicates = Vec::new();
        for long in opts {
            let name = long.name.clone();
            if !duplicates.contains(&OptionFlaw::LongDup(name)) {
                match checked.contains(&name) {
                    true => {
                        match detail {
                            true => { duplicates.push(OptionFlaw::LongDup(name)); },
                            false => { *found = true; return; },
                        }
                    },
                    false => { checked.push(name); },
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

    /* Dash (`-`) is an invalid short option (clashes with early terminator if it were given on its
     * own (`--`), and would be misinterpreted as a long option if given as the first in a short
     * option set (`--abc`)). */

    /// Check `ShortOption::new` rejects ‘-’
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_short_dash() {
        let _opt = ShortOption::new('-', false); // Should panic here in debug mode!
    }

    /* A short option cannot be represented by the unicode replacement char (`\u{FFFD}`). Support
     * for handling `OsStr` based argument sets involves a temporary lossy conversion to `str`, and
     * if the replacement char was allowed in valid options, this could result in incorrect matches.
     */

    /// Check `ShortOption::new` rejects ‘\u{FFFD}’
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_short_rep_char() {
        let _opt = ShortOption::new('\u{FFFD}', false); // Should panic here in debug mode!
    }

    /// Check `LongOption::new` rejects empty string
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_long_no_name() {
        let _opt = LongOption::new("", false); // Should panic here in debug mode!
    }

    /* Long option names cannot contain an `=` (used for declaring a data sub-argument in the same
     * argument; if names could contain an `=`, as data can, we would not know where to do the
     * split, complicating matching.
     */

    /// Check `LongOption::new` rejects equals (`=`) char
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_long_with_equals() {
        let _opt = LongOption::new("a=b", false); // Should panic here in debug mode!
    }

    /* Long option names cannot contain the unicode replacement char (`\u{FFFD}`). Support for
     * handling `OsStr` based argument sets involves a temporary lossy conversion to `str`, and if
     * the replacement char was allowed in valid options, this could result in incorrect matches.
     */

    /// Check `LongOption::new` rejects unicode replacement char (`\u{FFFD}`)
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_long_with_rep_char() {
        let _opt = LongOption::new("a\u{FFFD}b", false); // Should panic here in debug mode!
    }
}
