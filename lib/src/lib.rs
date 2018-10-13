// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! A lightweight, flexible and simple-to-use library provided to *assist* in processing command
//! line arguments.
//!
//! Licensed under the MIT license or the Apache license, Version 2.0, at your option.
//!
//! # Documentation
//!
//! Documentation has been split up into chapters:
//!
//! - [Overview](docs/overview/index.html)
//! - [Functionality](docs/functionality/index.html)
//! - [Usage](docs/usage/index.html)

#![doc(html_logo_url = "https://github.com/jnqnfe/gong/raw/master/logo.png",
       html_favicon_url = "https://github.com/jnqnfe/gong/raw/master/favicon.ico")]

pub mod docs;
#[cfg(test)]
mod tests;

/// Used to supply the set of information about available options to match against
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options<'a> {
    /* NOTE: these have been left public to allow efficient static creation of options */
    pub long: Vec<LongOption<'a>>,
    pub short: Vec<ShortOption>,
    pub mode: OptionsMode,
    pub allow_abbreviations: bool,
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
        OptionsMode::Standard
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

/// Result data returned from analysing an argument list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Results<'a> {
    /// Set of items describing what was found
    pub items: Vec<ItemClass<'a>>,
    /// Quick indication of error level issues (e.g. ambiguous match, or missing arg data)
    pub error: bool,
    /// Quick indication of warning level issues (e.g. unknown option, or unexpected data)
    pub warn: bool,
}

/// The possible classes of items identified and extracted from command line arguments.
///
/// This breaks down items to three classes - okay/warn/error - with each variant holding an
/// [`Item`], [`ItemW`] or [`ItemE`] variant which more specifically represents what was found.
///
/// We use a class wrapper rather than grouping items into separate vectors because a single vector
/// preserves order more simply. We break up item variants into groups for the advantages in
/// matching.
///
/// All sub-variants hold a `usize` value to be used for indicating the index of the argument at
/// which the item was found.
///
/// Most sub-variants also hold additional data. Long option sub-variants hold a string slice
/// reference to the matched option. Short option sub-variants hold the `char` matched. Options with
/// data arguments additionally hold a string slice reference to the data string matched, and in
/// some cases also a [`DataLocation`] variant. The [`NonOption`] sub-variant holds a string slice
/// reference to the matched string.
///
/// [`Item`]: enum.Item.html
/// [`ItemW`]: enum.ItemW.html
/// [`ItemE`]: enum.ItemE.html
/// [`DataLocation`]: enum.DataLocation.html
/// [`NonOption`]: enum.Item.html#variant.NonOption
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemClass<'a> {
    /// Non-problematic item
    Ok(Item<'a>),
    /// Warn-level item
    Warn(ItemW<'a>),
    /// Error-level item
    Err(ItemE<'a>),
}

/// Non-problematic items. See [`ItemClass`](enum.ItemClass.html) documentation for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item<'a> {
    /// Argument not considered an option.
    NonOption(usize, &'a str),
    /// Early terminator (`--`) encountered.
    EarlyTerminator(usize),
    /// Long option match.
    Long(usize, &'a str),
    /// Long option match, with expected data argument.
    LongWithData{ i: usize, n: &'a str, d: &'a str, l: DataLocation },
    /// Short option match.
    Short(usize, char),
    /// Short option match, with expected data argument.
    ShortWithData{ i: usize, c: char, d: &'a str, l: DataLocation },
}

/// Error-level items. See [`ItemClass`](enum.ItemClass.html) documentation for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemE<'a> {
    /// Long option match, but data argument missing [ERROR]
    LongMissingData(usize, &'a str),
    /// Short option match, but data argument missing [ERROR]
    ShortMissingData(usize, char),
    /// Ambiguous match with multiple long options. This only occurs when an exact match was not
    /// found, but multiple  abbreviated possible matches were found. [ERROR]
    AmbiguousLong(usize, &'a str),
}

/// Warn-level items. See [`ItemClass`](enum.ItemClass.html) documentation for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemW<'a> {
    /// Looked like a long option, but no match [WARN]
    UnknownLong(usize, &'a str),
    /// Unknown short option `char` [WARN]
    UnknownShort(usize, char),
    /// Looked like a long option, but a name was not actually specified. This only occurs for
    /// arguments starting with `--=` (in standard mode, `-=` in alternate mode). Because the first
    /// `=` in a long option argument is interpreted as indication that any subsequent characters
    /// are a data sub-argument, an `=` immediately following the long option prefix thus gives an
    /// empty option name. The data (if any) is ignored. [WARN]
    LongWithNoName(usize),
    /// Long option match, but came with unexpected data. For example `--foo=bar` when `--foo` takes
    /// no data. [WARN]
    LongWithUnexpectedData{ i: usize, n: &'a str, d: &'a str },
}

/// Used to describe where data was located, for options that require data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLocation {
    /// Found in the same argument (after an `=` for long options, or the remaining characters for a
    /// short option).
    SameArg,
    /// Found in the next argument.
    NextArg,
}

impl<'a> Options<'a> {
    /// Create a new object. Takes estimations of the number of options to expect to be added (for
    /// efficient vector allocation).
    pub fn new(count_long: usize, count_short: usize) -> Self {
        Self {
            long: Vec::with_capacity(count_long),
            short: Vec::with_capacity(count_short),
            mode: Default::default(),
            allow_abbreviations: true,
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

impl<'a> Results<'a> {
    /// Create a new result set (mostly only useful internally)
    pub fn new(size_guess: usize) -> Self {
        Self {
            items: Vec::with_capacity(size_guess),
            error: false,
            warn: false,
        }
    }

    /// Add a new item to the result set (mostly only useful internally)
    pub fn add(&mut self, item: ItemClass<'a>) {
        self.items.push(item);
    }
}

/// Analyses provided program arguments, using provided information about valid available options.
///
/// Returns a result set describing the result of the analysis. This may include `&str` references
/// to strings provided in the `args` and `options` parameter data. Take note of this with respect
/// to object lifetimes.
///
/// Expects available `options` data to have already been validated. (See
/// [`Options::is_valid`](struct.Options.html#method.is_valid)).
///
/// # Params
///
/// * `args`: List of arguments to process. This is a slice of `String` objects, not `&str`, because
///   Rust gives us command arguments from the environment as `String` not `&str` with
///   `std::env::args()`.
/// * `options`: Description of possible options to match against.
pub fn process<'a>(args: &'a [String], options: &Options<'a>) -> Results<'a> {
    use ItemClass as Class;

    /* NOTE: We deliberately do not perform validation of the provided `options` data within this
     * function; the burden to do so is left to the user, thus allowing this function to be called
     * multiple times if desired, without introducing inefficiency! */

    // Whether an argument of exactly `--` has been encountered, symbolising early termination of
    // option processing, meaning all remaining arguments are to be treated as non-options.
    let mut early_termination = false;

    let mode = options.mode;
    let mut results = Results::new(args.len());

    let mut arg_iter = args.iter().enumerate();
    while let Some((index, arg)) = arg_iter.next() {
        if early_termination {
            results.add(Class::Ok(Item::NonOption(index, arg)));
            continue;
        }

        // Get length of initial portion
        let start_len = arg.chars().take(3).count();

        // Early terminator
        if start_len == 2 && arg == &"--" {
            // Yes, it may be valuable info to the caller to know that one was encountered and
            // where, so let's not leave it out of the results.
            results.add(Class::Ok(Item::EarlyTerminator(index)));
            early_termination = true;
            continue;
        }

        let has_double_dash_prefix = || {
            start_len > 2 && arg.starts_with("--")
        };
        let has_single_dash_prefix = || {
            start_len > 1 && arg.starts_with("-")
        };

        // Long option
        if (mode == OptionsMode::Standard && has_double_dash_prefix()) ||
            (mode == OptionsMode::Alternate && has_single_dash_prefix())
        {
            /* We need to deal with the fact that arg data may be supplied in the same argument,
             * separated by an `=`, and also that the user is allowed to supply an abbreviated form
             * of an available option, so long as it is unique, which requires checking for
             * ambiguity. (See documentation). */

            // Extract name, splitting from optional data arg
            let without_prefix = match mode {
                OptionsMode::Standard => arg.split_at(2).1, //"--" length
                OptionsMode::Alternate => arg.split_at(1).1, //"-" length
            };
            let mut parts_iter = without_prefix.splitn(2, '=');
            let name = parts_iter.next().unwrap(); /* Must exist */
            let data_included = parts_iter.next();

            let name_char_count = name.chars().count();

            // This occurs with `--=` or `--=foo` (`-=` or `-=foo` in alt mode)
            if name_char_count == 0 {
                results.add(Class::Warn(ItemW::LongWithNoName(index)));
                results.warn = true;
                continue;
            }

            let mut matched: Option<&LongOption> = None;
            let mut ambiguity = false;
            for candidate in &options.long {
                let cand_char_count = candidate.name.chars().count();
                // Exact
                if cand_char_count == name_char_count &&
                    candidate.name == name
                {
                    // An exact match overrules a previously found partial match and ambiguity found
                    // with multiple partial matches.
                    matched = Some(&*candidate);
                    ambiguity = false;
                    break;
                }
                // Abbreviated
                else if options.allow_abbreviations &&
                    cand_char_count > name_char_count &&
                    candidate.name.starts_with(name)
                {
                    if matched.is_none() {
                        matched = Some(&*candidate);
                    }
                    else {
                        ambiguity = true;
                        break;
                    }
                }
            }

            if ambiguity {
                results.add(Class::Err(ItemE::AmbiguousLong(index, name)));
                results.error = true;
            }
            else if let Some(matched) = matched {
                // Use option's full name, not the possibly abbreviated user provided one
                let opt_name = &matched.name;

                if matched.expects_data {
                    // Data included in same argument
                    if let Some(data) = data_included {
                        results.add(Class::Ok(Item::LongWithData {
                            i: index, n: opt_name, d: data, l: DataLocation::SameArg }));
                    }
                    // Data included in next argument
                    else if let Some((_, next_arg)) = arg_iter.next() {
                        results.add(Class::Ok(Item::LongWithData {
                            i: index, n: opt_name, d: next_arg, l: DataLocation::NextArg }));
                    }
                    // Data missing
                    else {
                        results.add(Class::Err(ItemE::LongMissingData(index, opt_name)));
                        results.error = true;
                    }
                }
                else {
                    let mut added_entry = false;
                    if let Some(data) = data_included {
                        // Ignore unexpected data if empty string
                        if data.len() > 0 {
                            results.add(Class::Warn(ItemW::LongWithUnexpectedData {
                                i: index, n: opt_name, d: data }));
                            results.warn = true;
                            added_entry = true;
                        }
                    }
                    if !added_entry {
                        results.add(Class::Ok(Item::Long(index, opt_name)));
                    }
                }
            }
            else {
                // Again, we ignore any possibly included data in the argument
                results.add(Class::Warn(ItemW::UnknownLong(index, name)));
                results.warn = true;
            }
            continue;
        }

        // Short option(s)
        // Note, lone `-` argument is considered a non-option!
        if mode == OptionsMode::Standard && has_single_dash_prefix() {
            let last_char_index = arg.chars().skip(1).count() - 1;
            for (i, (byte_pos, ch)) in arg.char_indices().skip(1).enumerate() {
                let mut match_found = false;
                let mut expects_data = false;
                for candidate in &options.short {
                    if candidate.ch == ch {
                        match_found = true;
                        expects_data = candidate.expects_data;
                        break;
                    }
                }

                if !match_found {
                    results.add(Class::Warn(ItemW::UnknownShort(index, ch)));
                    results.warn = true;
                }
                else if !expects_data {
                    results.add(Class::Ok(Item::Short(index, ch)));
                }
                else {
                    // If not last char, remaining chars are our data
                    if i < last_char_index {
                        let next_char_byte_pos = byte_pos + ch.len_utf8();
                        let data = arg.split_at(next_char_byte_pos).1;
                        results.add(Class::Ok(Item::ShortWithData {
                            i: index, c: ch, d: data, l: DataLocation::SameArg }));
                        break;
                    }
                    // Data included in next argument
                    else if let Some((_, next_arg)) = arg_iter.next() {
                        results.add(Class::Ok(Item::ShortWithData {
                            i: index, c: ch, d: next_arg, l: DataLocation::NextArg }));
                    }
                    // Data missing
                    else {
                        results.add(Class::Err(ItemE::ShortMissingData(index, ch)));
                        results.error = true;
                    }
                }
            }
            continue;
        }

        results.add(Class::Ok(Item::NonOption(index, arg)));
    }
    results
}
