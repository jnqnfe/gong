//! A flexible, lightweight and simple-to-use library for processing command line arguments. A
//! 'getopt' next-gen replacement.

// This file is part of the `gong` command-line argument processing library.
//
// Copyright (c) 2017 Lyndon Brown
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! # About
//!
//! The 'getopt' and related 'getopt_long' functions, have, for a long time, served to assist in the
//! processing of arguments supplied to a program. This library provides a "next-gen" replacement
//! for use with Rust programs.
//!
//! Licensed under the MIT license or the Apache license, Version 2.0, at your option.
//!
//! ## Design
//!
//! This is not the only solution available to projects built with the Rust programming language,
//! and each one may have it's own positive and negative aspects. Unlike some of these other
//! "next-gen"/modern solutions, this library does not try to completely take over all aspects of
//! argument handling; doing so can rather easily impose restrictions making a solution unsuitable
//! for some program designs. This solution, like 'getopt', is used to *assist* in argument
//! processing, not take over, and as such is highly flexible and lightweight.
//!
//! The basic premise of usage is simple - provide the processing function with a set of available
//! options and the input arguments to be processed, and it returns the results of its analysis.
//! From there you can take further action - output error information if the user made a mistake;
//! output help/usage information if requested; store state information from flag type options; and
//! store data (converting values as necessary) from non-options and options with data, before
//! proceeding with whatever your program was designed to do.
//!
//! Some major differences to the old 'getopt'/'getopt_long' solution include:
//!
//! 1. All processing can be done in one go, rather than with recursive function calls;
//! 2. "Non-options" are **not** shuffled to the end of the list, unlike the default behaviour of
//!    'getopt'. Not doing this preserves possibly invaluable information;
//! 3. The "convenience" functionality of `-W foo` being treated as `--foo` is not supported
//!    (unnecessary complexity).
//!
//! This library could also be used as a foundation for other libraries that want to take over more
//! of the workload of argument handling than this library does.
//!
//! # Functionality
//!
//! Basic feature support is on par with legacy 'getopt_long'.
//!
//! ## Option support
//!
//! Two option processing modes are available, supporting two different popular styles of options.
//!
//! ### Mode 1 - Standard (default)
//!
//! This mode supports traditional *long* and *short* options.
//!
//! An argument starting with a dash (`-`) and followed by additional characters, is treated as an
//! *option* argument, anything else is a *non-option*. An argument of `--` followed by additional
//! characters is a *long* option, which, after the `--` prefix, consists of an option *name*
//! (followed optionally by a *data sub-argument*, as discussed below). An argument of a single dash
//! (`-`) followed by additional (non-dash) characters is a *short option set*, where each character
//! after the `-` prefix is a *short* option *character* (except with respect to *data
//! sub-arguments*, as mentioned below). An argument of exactly `--` only is special - an *early
//! terminator* - symbolising early termination of argument interpretation, meaning that all
//! subsequent arguments should be assumed to be non-options (useful in some situations/designs for
//! separating your option arguments from those to be passed on to something else).
//!
//! Options may have a single mandatory *data sub-argument*. For long options, data is provided
//! either in the next argument (e.g. `--foo bar`) or in the same argument, separated from the name
//! by an `=` (e.g. `--foo=bar`). For short options, data is provided either in the next argument
//! (e.g. `-o arg`), or if the option character is not the last character in the argument, the
//! remaining characters are taken to be its data arg (e.g. `-oarg`). An argument can contain
//! multiple short options grouped together as a *set* (e.g. `-abc`), but of course users need to be
//! careful doing so with those requiring data - for correct interpretation only one short option
//! with data can be grouped, and it must be the last one in the set. (If in `-abc` all three
//! characters are valid options, and `b` takes data, `c` will be consumed as `b`'s data instead of
//! being interpreted as an option).
//!
//! If a long option is encountered where the argument contains one or more `=` characters, then the
//! left hand portion of the first `=` character is taken to be the long option name, and the right
//! hand portion as a data sub-argument, thus valid available option names cannot contain `=`
//! characters. If the name does not match any available long option, a failed match is reported and
//! the data sub-arg is completely ignored. If there is a match and it requires a data sub-arg, but
//! the `=` was the last character in the argument, (e.g. `--foo=`), then the data sub-arg is taken
//! to be an empty string. If there is a match with an option that does not require a data sub-arg,
//! but one was provided and it is not an empty string, this will be noted as unexpected in the
//! results of analysis.
//!
//! Abbreviated long option name matching is supported, i.e. the feature than users can use an
//! abbreviated form of a long option's name and get a match, so long as the abbreviation uniquely
//! matches a single long option. As an example, if `foo` and `foobar` are available long options,
//! then for the possible input arguments of { `--f`, `--fo`, `--foo`, `--foob`, `--fooba`, and
//! `--foobar` }, `--foo` and `--foobar` are exact matches for `foo` and `foobar` respectively;
//! `--f` and `--fo` are invalid as being ambiguous (and noted as such in the results); and `--foob`
//! and `--fooba` both uniquely match `foobar` and so are valid. This feature is enabled by default,
//! but can be disabled if desired.
//!
//! ### Mode 2 - Alternate
//!
//! This mode is very similar to mode 1, with the main difference simply being that only long
//! options are supported, and long options use a single dash (`-`) as a prefix rather than two,
//! i.e. `-help` rather than `--help`. Some people simply prefer this style, and support for it was
//! very easy to add.
//!
//! **Note**: Short options can still be added to the option set in this mode, and it will still
//! pass as valid; they will simply be ignored when performing matching.
//!
//! ## Mismatch suggestions
//!
//! This library does not (currently) itself provide any suggestion mechanism for failed option
//! matches - i.e. the ability to take an unmatched long option and pick the most likely of the
//! available options that the user may have actually meant to use, to suggest to them when
//! reporting the error. There is nothing however stopping users of this library from running
//! unmatched options through a third-party library to obtain the suggestion to display.
//!
//! ## Utf-8 support
//!
//! This library expects all provided strings to be valid Utf-8.
//!
//! Native Utf-8 support in Rust makes handling Utf-8 strings largely trivial. It is important to
//! understand that in Rust a `char` is four bytes (it was only one byte in older languages like C);
//! but a sequence of `char`s are typically stored more efficiently than this in a string. This
//! widened `char` type broadens the range of possible characters that can be used as short options,
//! without us worrying about any multi-byte complexity. This allows for instance `💖` (the
//! "sparkle heart" `char`) to be a short option, if you wanted, along with a huge set of other
//! characters of various types to choose from. (The "sparkle heart" `char` take three bytes in a
//! Utf-8 string, and would not have been easy to support in C with the legacy 'getopt' solution).
//!
//! With respect to long options, `--foo`, `--föö` and `--föö` are all different options (the last
//! two may look the same, but read on), and are all perfectly valid options to make available. The
//! first consists of simple latin characters only. The second and third use "umlauts" (diaeresis)
//! above the `o`'s, however the first of these uses a `char` with the umlaut built in (`U+F6`) and
//! the second uses the standard `o` (`U+6F`) followed by the special umlaut combining `char`
//! (`U+0308`), thus they appear the same but are actually different "under the hood". (It would not
//! be efficient or worthwhile to try to handle the latter two as being the same option).
//!
//! Only single `char`s are supported for short options. A `char` paired with one or more special
//! combinator/selector `char`s thus cannot be specified as an available short option. Such special
//! `char`s are treated by this library as perfectly valid available short options in their own
//! right. Thus, whilst `-ö` (using `U+F6`) results in a single matched/unmatched entry in the
//! results returned from the [`process`] function, `-ö` (using `U+6F` followed by the `U+0308`
//! combinator) will result in two entries, for what looks visibly to be one character. As another
//! example, `❤` is the "black heart" character, and `❤️` is it along with the `U+FE0F` "variant #16
//! \- emoji" selector `char`; with the selector, `--❤️` is a single matched/unmatched long
//! option, while `-❤️` is a pair of matched/unmatched short options, one for the "black heart"
//! `char` and one for the selector `char`.
//!
//! # Usage
//!
//! To use this library, start by adding a dependency entry for it in your project's `Cargo.toml`
//! file; then make sure to declare use of the crate at the root of the module hierarchy
//! (`src/main.rs` or `src/lib.rs`):
//!
//! ```rust,ignore
//! extern crate gong;
//! ```
//!
//! Now proceed with the following steps.
//!
//! ## Step #1: Describe the available options
//!
//! First, you need to compile a list of available options. For example:
//!
//! ```rust
//! let mut opts = gong::Options::new(6, 4); //Estimate counts for efficiency
//! opts.add_long("help")
//!     .add_short('h')
//!     .add_long("foo")
//!     .add_long("version")
//!     .add_long("foobar")
//!     .add_long("ábc")      // Using a combining char (accent)
//!     .add_long_data("hah") // This one expects a data arg
//!     .add_short('❤')
//!     .add_short('x')
//!     .add_short_data('o'); // So does this one
//! debug_assert!(opts.is_valid());
//! ```
//!
//! **Note**: The underlying data structures used to represent options actually have publicly
//! accessible attributes, thus leaving open less tidy, but more efficient means of declaring a data
//! set, bypassing the function calls used here, if desired.
//!
//! ### Set mode
//!
//! If you want to use *alternate* option mode rather than *standard* (default), as discussed above,
//! the [`Options::set_mode`] method is available.
//!
//! You can control whether or not to allow abbreviated matching with the
//! [`Options::set_allow_abbreviations`] method.
//!
//! ### Validation
//!
//! Some validation is performed by the `add_*` methods, but for full validation (including checking
//! for duplicates) the [`Options::is_valid`] method is provided, as used above. Details of any
//! problems identified by this method are output to `stderr`. It is recommended that you only use
//! it in a *debug* assert variant, as here, to allow catching mistakes in development, but
//! otherwise avoid wasting energy for option sets in release builds that you know must be perfectly
//! valid.
//!
//! **Note**: With respect to what is or is not a duplicate, only the name/`char` matters; the
//! `expects_data` attribute makes no difference.
//!
//! ## Step #2: Gather arguments to be processed
//!
//! You also need to retrieve (or build) a set of arguments to be processed. This must be a set of
//! `String` objects (as Rust provides actual program args from the environment as `String` not
//! `&str`). You can collect program arguments as follows:
//!
//! ```rust
//! let args: Vec<String> = std::env::args().collect();
//! ```
//!
//! The very first entry in the list is the program path/name, and often you will not be interested
//! in it. You can skip it in two easy ways, either: a) when passing the arguments to the processing
//! function in the next step, use `&args[1..]` instead of `&args[..]`; or b) use the iterator
//! `skip` method, as here:
//!
//! ```rust
//! let args: Vec<String> = std::env::args().skip(1).collect();
//! ```
//!
//! **Note**: Of course you do not have to provide the real program args, you can provide any set of
//! `String` objects, and you can even of course take the real set and modify it first if you wish.
//!
//! ## Step #3: Processing
//!
//! With data gathered, you now simply need to give it to the [`process`] function. This function
//! will perform an analysis and return a set of results that describe what it identified.
//!
//! ```rust,ignore
//! let results = gong::process(&args[..], &opts);
//! ```
//!
//! Of course if for any reason you do **not** want to process all arguments in one go, you always
//! have the option of processing one argument at a time (or in groups of whatever number you
//! choose), calling [`process`] for each. (Naturally though you must beware of complications
//! handling *in-next-arg* data sub-arguments doing this).
//!
//! ## Step #4: Take action
//!
//! It is now up to you to take appropriate action in response to what was found.
//!
//! The [`Results`] object returned by the [`process`] function contains `error` and `warn`
//! booleans, which give a quick indication of problems. It also contains a list of items,
//! describing in detail what was found. The items in the item list are stored in the same order as
//! found in the input arguments.
//!
//! The entries in the item list are [`ItemClass`] variants, which wrap variants of [`Item`],
//! [`ItemW`] or [`ItemE`] \(okay/warn/error), thus making it simple to match by class. All variants
//! of each item class hold a `usize` value to be used for indicating the index of the argument in
//! which the item was found. For simple scenarios, this may be ignored, but in some situations it
//! is highly valuable information. Similarly, information is returned where applicable with data
//! sub-args as to whether the data arg was located in the same argument or the next.
//!
//! **Note**: some item variants that may be returned in the result set hold `&str` references to
//! strings that were provided in the argument and option data provided to [`process`]. This is done
//! for efficiency. Beware of this with respect to lifetimes.
//!
//! # Have a play
//!
//! The source code repository includes a small test application for trying out the library's
//! analysis capabilities. It has a small set of built-in example options of different kinds, and
//! when run, outputs details of them along with details of analysing any provided arguments against
//! them.
//!
//! To use it, see the instructions in the `README.md` file found in the `bin` sub-directory.
//!
//! [`process`]: fn.process.html
//! [`ItemClass`]: enum.ItemClass.html
//! [`Item`]: enum.Item.html
//! [`ItemW`]: enum.ItemW.html
//! [`ItemE`]: enum.ItemE.html
//! [`Results`]: struct.Results.html
//! [`Options::is_valid`]: struct.Options.html#method.is_valid
//! [`Options::set_mode`]: struct.Options.html#method.set_mode
//! [`Options::set_allow_abbreviations`]: struct.Options.html#method.set_allow_abbreviations

#![doc(html_logo_url = "https://github.com/jnqnfe/gong/raw/master/logo.png",
       html_favicon_url = "https://github.com/jnqnfe/gong/raw/master/favicon.ico")]

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

/// Process provided command-line arguments
///
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
                    matched = Some(candidate);
                    ambiguity = false;
                    break;
                }
                // Abbreviated
                else if options.allow_abbreviations &&
                    !ambiguity &&
                    cand_char_count > name_char_count &&
                    candidate.name.starts_with(name)
                {
                    match matched {
                        Some(_) => { ambiguity = true; },
                        None => { matched = Some(candidate); },
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
