// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Option description components
//!
//! This module contains components to do with describing the *options* “available” within a given
//! program, i.e. those that an argument list will be parsed against. There are components for
//! describing both individual *options* and sets of *options*.
//!
//! See the separate [*options* support discussion][options] for details on the types of *options*
//! supported by this parsing library.
//!
//! [options]: ../docs/ch3_options/index.html

#[cfg(feature = "suggestions")]
use std::ffi::OsStr;

/// Description of an available long option
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LongOption<'a> {
    /* NOTE: these have been left public to allow efficient static creation of options */
    /// Long option name, excluding the `--` prefix
    pub name: &'a str,
    /// Option type
    pub opt_type: OptionType,
}

/// Description of an available short option
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ShortOption {
    /* NOTE: these have been left public to allow efficient static creation of options */
    /// Short option character
    pub ch: char,
    /// Option type
    pub opt_type: OptionType,
}

/// Description of an available option with both a long and short identifier
///
/// This is not used in option sets directly, but can be useful where you have a pair of related
/// long and short option identifiers, in terms of being able to spawn individual long and short
/// option types from this single definition, as well as [`FindOption`]s.
///
/// [`FindOption`]: ../analysis/enum.FindOption.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OptionPair<'a> {
    /* NOTE: these have been left public to allow efficient static creation of options */
    /// Long option name, excluding the `--` prefix
    pub name: &'a str,
    /// Short option character
    pub ch: char,
    /// Option type
    pub opt_type: OptionType,
}

/// Type of option (flag or data-value taking)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OptionType {
    /// A simple flag style option (takes no data value)
    Flag,
    /// A data-value taking option, where providing a value is mandatory
    ///
    /// The data value can be provided within the same argument, but if not provided there then the
    /// next argument is consumed as the data value. In the latter scenario, if no next argument
    /// exists, then a missing-data-value problem is reported.
    Data,
    /// An option where providing a data-value is optional, thus it can be used in both flag and
    /// data-taking forms in an input argument, i.e. providing the data value is optional.
    ///
    /// One restriction with this type is that to avoid parsig ambiguity, a data value can only be
    /// provided within the same argument, never the next argument. If a data value is not provided
    /// within the same argument as the option `char`/name, then it is considered to be without one.
    Mixed,
}

/// Extendible option set
///
/// Used to supply the set of information about available options to match against
///
/// This is the “extendible” variant which uses `Vec`s to hold the option lists and thus is flexible
/// in allowing addition of options, and may re-allocate as necessary.
///
/// Note, certain add option methods panic with invalid identifiers, as documented, however you must
/// understand that the validation checks only do the bare minimum of checking for the most crucial
/// problems that could cause issues when parsing. It is up to you to otherwise ensure that
/// identifiers are sensibly chosen.
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
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct OptionSet<'r, 's: 'r> {
    /* NOTE: these have been left public to allow efficient static creation of options */
    pub long: &'r [LongOption<'s>],
    pub short: &'r [ShortOption],
}

impl<'r, 's: 'r> From<&'r OptionSetEx<'s>> for OptionSet<'r, 's> {
    /// Create an `OptionSet` referencing an `OptionSetEx`’s vectors as slices
    fn from(r: &'r OptionSetEx<'s>) -> Self {
        Self {
            long: &r.long[..],
            short: &r.short[..],
        }
    }
}

impl<'r, 's: 'r> From<OptionSet<'r, 's>> for OptionSetEx<'s> {
    /// Create an `OptionSetEx` from an `OptionSet`
    fn from(o: OptionSet<'r, 's>) -> Self {
        Self {
            long: o.long.iter().cloned().collect(),
            short: o.short.iter().cloned().collect(),
        }
    }
}

impl<'r, 's: 'r> PartialEq<OptionSet<'r, 's>> for OptionSetEx<'s> {
    fn eq(&self, rhs: &OptionSet<'r, 's>) -> bool {
        OptionSet::from(self).eq(rhs)
    }
}

impl<'r, 's: 'r> PartialEq<OptionSetEx<'s>> for OptionSet<'r, 's> {
    fn eq(&self, rhs: &OptionSetEx<'s>) -> bool {
        self.eq(&OptionSet::from(rhs))
    }
}

/// Description of a validation issue within an option in an [`OptionSet`](struct.OptionSet.html) or
/// [`OptionSetEx`](struct.OptionSetEx.html) set.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OptionFlaw<'a> {
    /// Long option name is empty string
    LongEmptyName,
    /// Long option name contains a forbidden `char`
    LongNameHasForbiddenChar(&'a str, char),
    /// Short option `char` is a forbidden `char`
    ShortIsForbiddenChar(char),
    /// Duplicate short option found
    ShortDuplicated(char),
    /// Duplicate long option found
    LongDuplicated(&'a str),
}

impl<'a> LongOption<'a> {
    /// Create a new long option descriptor
    ///
    /// Panics (debug only) if the given name is invalid.
    #[inline]
    fn new(name: &'a str, opt_type: OptionType) -> Self {
        debug_assert!(Self::validate(name).is_ok());
        Self { name, opt_type, }
    }

    /// Validate a given name as a possible long option
    ///
    /// Returns the first flaw identified, if any
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    #[must_use]
    fn validate(name: &str) -> Result<(), OptionFlaw> {
        static FORBIDDEN: &[char] = &[
            '=',        // Would clash with “in-same-arg” data value extraction
            '\u{FFFD}', // Would cause problems with correct `OsStr` based parsing
        ];
        if name.is_empty() {
            return Err(OptionFlaw::LongEmptyName);
        }
        for c in FORBIDDEN {
            if name.contains(*c) {
                return Err(OptionFlaw::LongNameHasForbiddenChar(name, *c));
            }
        }
        Ok(())
    }
}

impl ShortOption {
    /// Create a new short option descriptor
    ///
    /// Panics (debug only) if the given `char` is invalid.
    #[inline]
    fn new(ch: char, opt_type: OptionType) -> Self {
        debug_assert!(Self::validate(ch).is_ok());
        Self { ch, opt_type, }
    }

    /// Validate a given character as a possible short option
    ///
    /// Returns the first flaw identified, if any
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    #[must_use]
    fn validate<'a>(ch: char) -> Result<(), OptionFlaw<'a>> {
        static FORBIDDEN: &[char] = &[
            '-',        // Would clash with correct identification of short option sets in some cases
            '\u{FFFD}', // Would cause problems with correct `OsStr` based parsing
        ];
        for c in FORBIDDEN {
            if ch == *c {
                return Err(OptionFlaw::ShortIsForbiddenChar(*c));
            }
        }
        Ok(())
    }
}

impl<'a> OptionPair<'a> {
    /// Create a corresponding long option type
    #[inline(always)]
    pub const fn as_long(&self) -> LongOption<'a> {
        LongOption { name: self.name, opt_type: self.opt_type }
    }

    /// Create a corresponding short option type
    #[inline(always)]
    pub const fn as_short(&self) -> ShortOption {
        ShortOption { ch: self.ch, opt_type: self.opt_type }
    }

    /// Create a corresponding `FindOption`
    #[inline(always)]
    pub const fn as_findopt(&self) -> crate::analysis::FindOption<'a> {
        crate::analysis::FindOption::Pair(self.ch, self.name)
    }

    /// Create from corresponding separate short and long types
    #[inline]
    pub const fn from_separate(short: ShortOption, long: LongOption<'a>) -> Self {
        //TODO: cannot use this assertion in const functions yet...
        //assert_eq!(short.opt_type, long.opt_type);
        Self { name: long.name, ch: short.ch, opt_type: long.opt_type }
    }
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
    #[inline]
    pub fn with_capacity(long_count_est: usize, short_count_est: usize) -> Self {
        Self {
            long: Vec::with_capacity(long_count_est),
            short: Vec::with_capacity(short_count_est),
        }
    }

    /// Create an [`OptionSet`](struct.OptionSet.html) referencing `self`’s vectors as slices
    #[inline]
    pub fn as_fixed<'r>(&'r self) -> OptionSet<'r, 's> where 's: 'r {
        self.into()
    }

    /// Checks if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.long.is_empty() && self.short.is_empty()
    }

    /// Add a long option
    ///
    /// Panics (debug only) on invalid name.
    #[inline]
    pub fn add_long(&mut self, name: &'s str, ty: OptionType) -> &mut Self {
        self.long.push(LongOption::new(name, ty));
        self
    }

    /// Add a short option
    ///
    /// Panics (debug only) on invalid `char` choice.
    #[inline]
    pub fn add_short(&mut self, ch: char, ty: OptionType) -> &mut Self {
        self.short.push(ShortOption::new(ch, ty));
        self
    }

    /// Add a long option and short option pair
    ///
    /// Panics (debug only) on invalid name or character.
    #[inline]
    pub fn add_pair(&mut self, ch: char, name: &'s str, ty: OptionType) -> &mut Self {
        self.add_short(ch, ty).add_long(name, ty)
    }

    /// Add an existing (ready-made) long option
    ///
    /// No validation is performed here; the item given should be valid though.
    #[inline]
    pub fn add_existing_long(&mut self, long: LongOption<'s>) -> &mut Self {
        self.long.push(long);
        self
    }

    /// Add an existing (ready-made) short option
    ///
    /// No validation is performed here; the item given should be valid though.
    #[inline]
    pub fn add_existing_short(&mut self, short: ShortOption) -> &mut Self {
        self.short.push(short);
        self
    }

    /// Add an existing (ready-made) option pair
    ///
    /// No validation is performed here; the item given should be valid though.
    #[inline]
    pub fn add_existing_pair(&mut self, pair: OptionPair<'s>) -> &mut Self {
        self.add_existing_long(pair.as_long()).add_existing_short(pair.as_short())
    }

    /// Add multiple short options from string
    ///
    /// Sometimes it may be preferable to add multiple short options by specifying them as a string,
    /// rather than making multiple individual method calls. This method offers such a facility as a
    /// convenience.
    ///
    /// Note that the colon (`:`) character has special meaning when used in the provided string;
    /// each `char` may optionally be followed by a colon (`:`) which if present indicates that the
    /// option is a data-taking short option; or two colons, which indicates that providing a data
    /// argument is optional (“mixed” type). (This is the behaviour offered by *getopt*). Unexpected
    /// colons (at the beginning of the string), are simply ignored. Using more than two is treated
    /// as if only two were given.
    ///
    /// Also, note that this does nothing to avoid duplicates being added, and white space
    /// characters in the provided string are **not** ignored (not that it is sensible or even makes
    /// any sense to attempt to assign whitespace characters as short options).
    ///
    /// Panics (debug only) on invalid `char` choice.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let mut set = gong::options::OptionSetEx::new();
    /// // The following adds six short options; `b` and `e` take data, `e` optionally.
    /// set.add_shorts_from_str("ab:cde::f");
    /// ```
    pub fn add_shorts_from_str(&mut self, set: &str) -> &mut Self {
        let mut iter = set.chars().peekable();
        while let Some(':') = iter.peek() {
            let _ = iter.next();
        }
        while let Some(ch) = iter.next() {
            let opt_type = match iter.peek() {
                Some(':') => {
                    let _ = iter.next();
                    match iter.peek() {
                        Some(':') => {
                            let _ = iter.next();
                            OptionType::Mixed
                        },
                        _ => OptionType::Data,
                    }
                },
                _ => OptionType::Flag,
            };
            // Note, we deliberately use a method known to panic on invalid `char` here!
            self.short.push(ShortOption::new(ch, opt_type));
            while let Some(':') = iter.peek() {
                let _ = iter.next();
            }
        }
        self
    }

    /// Checks validity of option set
    ///
    /// Returns `true` if valid.
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    ///
    /// See also the [`validate`](#method.validate) method.
    #[inline]
    pub fn is_valid(&self) -> bool {
        validation::validate_set(&self.as_fixed(), false).is_ok()
    }

    /// Checks validity of option set, returning details of any problems
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    #[inline]
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
    pub fn suggest(&self, unknown: &OsStr) -> Option<&'s str> {
        self.as_fixed().suggest(unknown)
    }
}

impl<'r, 's: 'r> OptionSet<'r, 's> {
    /// Converts to an “extendible” copy of `self`
    ///
    /// This duplicates the options in `self` into an [`OptionSetEx`](struct.OptionSetEx.html).
    #[inline]
    pub fn to_extendible(self) -> OptionSetEx<'s> {
        OptionSetEx::from(self)
    }

    /// Checks if empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.long.is_empty() && self.short.is_empty()
    }

    /// Checks validity of option set
    ///
    /// Returns `true` if valid.
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    ///
    /// See also the [`validate`](#method.validate) method.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        validation::validate_set(self, false).is_ok()
    }

    /// Checks validity of option set, returning details of any problems
    ///
    /// Note, only the most crucial problems that could cause issues when parsing are checked for.
    /// Passing validation is not a confirmation that a given identifier is sensible, or entirely
    /// free of issues.
    #[inline(always)]
    pub fn validate(&self) -> Result<(), Vec<OptionFlaw<'s>>> {
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
    pub fn suggest(&self, unknown: &OsStr) -> Option<&'s str> {
        let unknown_lossy = unknown.to_string_lossy();
        crate::matching::suggest(&unknown_lossy, self.long.iter(), |&o| o.name)
    }
}

/// Option set validation
pub(crate) mod validation {
    use super::{LongOption, ShortOption, OptionSet, OptionFlaw};

    /// Checks validity of option set, optionally returning details of any problems
    ///
    /// If no problems are found, it returns `Ok(())`, otherwise `Err(_)`.
    ///
    /// If `detail` is `false`, it returns early on encountering a problem (with an empty `Vec`),
    /// useful for quick `is_valid` checks. Otherwise it builds up and provides a complete list of
    /// flaws.
    #[must_use]
    pub fn validate_set<'r, 's: 'r>(set: &OptionSet<'r, 's>, detail: bool)
        -> Result<(), Vec<OptionFlaw<'s>>>
    {
        let mut flaws: Vec<OptionFlaw<'s>> = Vec::new();

        for candidate in set.long {
            if let Err(f) = LongOption::validate(candidate.name) {
                match detail {
                    true => { flaws.push(f); },
                    false => { return Err(flaws); },
                }
            }
        }

        for candidate in set.short {
            if let Err(f) = ShortOption::validate(candidate.ch) {
                match detail {
                    true => { flaws.push(f); },
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
        if opts.is_empty() { return; }
        let mut duplicates = Vec::new();
        for (i, short) in opts[..opts.len()-1].iter().enumerate() {
            let ch = short.ch;
            if !duplicates.contains(&OptionFlaw::ShortDuplicated(ch)) {
                for short2 in opts[i+1..].iter() {
                    if ch == short2.ch {
                        match detail {
                            true => {
                                duplicates.push(OptionFlaw::ShortDuplicated(ch));
                                break;
                            },
                            false => { *found = true; return; },
                        }
                    }
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
        if opts.is_empty() { return; }
        let mut duplicates = Vec::new();
        for (i, long) in opts[..opts.len()-1].iter().enumerate() {
            let name = long.name.clone();
            if !duplicates.contains(&OptionFlaw::LongDuplicated(name)) {
                for long2 in opts[i+1..].iter() {
                    if name == long2.name {
                        match detail {
                            true => {
                                duplicates.push(OptionFlaw::LongDuplicated(name));
                                break;
                            },
                            false => { *found = true; return; },
                        }
                    }
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
    use super::{LongOption, ShortOption, OptionType};

    /* Dash (`-`) is an invalid short option (clashes with early terminator if it were given on its
     * own (`--`), and would be misinterpreted as a long option if given as the first in a short
     * option set (`--abc`)). */

    /// Check `ShortOption::new` rejects ‘-’
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_short_dash() {
        let _opt = ShortOption::new('-', OptionType::Flag); // Should panic here in debug mode!
    }

    /* A short option cannot be represented by the unicode replacement char (`\u{FFFD}`). Support
     * for handling `OsStr` based argument sets involves a temporary lossy conversion to `str`, and
     * if the replacement char was allowed in valid options, this could result in incorrect matches.
     */

    /// Check `ShortOption::new` rejects ‘\u{FFFD}’
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_short_rep_char() {
        let _opt = ShortOption::new('\u{FFFD}', OptionType::Flag); // Should panic here in debug mode!
    }

    /// Check `LongOption::new` rejects empty string
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_long_no_name() {
        let _opt = LongOption::new("", OptionType::Flag); // Should panic here in debug mode!
    }

    /* Long option names cannot contain an `=` (used for declaring a data sub-argument in the same
     * argument; if names could contain an `=`, as data can, we would not know where to do the
     * split, complicating matching.
     */

    /// Check `LongOption::new` rejects equals (`=`) char
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_long_with_equals() {
        let _opt = LongOption::new("a=b", OptionType::Flag); // Should panic here in debug mode!
    }

    /* Long option names cannot contain the unicode replacement char (`\u{FFFD}`). Support for
     * handling `OsStr` based argument sets involves a temporary lossy conversion to `str`, and if
     * the replacement char was allowed in valid options, this could result in incorrect matches.
     */

    /// Check `LongOption::new` rejects unicode replacement char (`\u{FFFD}`)
    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn create_long_with_rep_char() {
        let _opt = LongOption::new("a\u{FFFD}b", OptionType::Flag); // Should panic here in debug mode!
    }
}
