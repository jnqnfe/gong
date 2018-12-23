// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Analysis components
//!
//! This module contains components to do with the analysis that results from parsing an argument
//! list.
//!
//! The most fundamental components are the [`Item`], [`ItemW`], [`ItemE`] and [`ItemClass`] enums,
//! with the first three representing different types of items and item conditions that might be
//! found, and [`ItemClass`] being a wrapper, giving a quick okay/warning/error status indicator,
//! and allowing instances to co-exist within a single list.
//!
//! # “All in one” based analysis
//!
//! If you use the “one at a time” (iterative) parsing model, then the two just mentioned item types
//! are the only analysis types of relevance; However, if you use the “all in one” (aka
//! “data-mining”) model then there are some additional types that come into play, namely
//! [`Analysis`], [`ItemSet`] and [`FindOption`].
//!
//! The “all in one” model is basically a wrapper around the iterative model, collecting the results
//! of running an iterator into a vector within an object ([`Analysis`]) that exposes methods for
//! retrieving information (aka “data-mining”).
//!
//! The [`FindOption`] type is simply one used with certain data-mining methods, for specifying an
//! option to be searched for. Note that a pair of a related long option and short option can be
//! specified together, which data mining methods will correctly consider. The [`FoundOption`] type
//! is the return type equivalent, which differs only in not having a long+short pair variant.
//!
//! [`Item`]: struct.Item.html
//! [`ItemW`]: struct.ItemW.html
//! [`ItemE`]: struct.ItemE.html
//! [`ItemClass`]: struct.ItemClass.html
//! [`ItemSet`]: struct.ItemSet.html
//! [`Analysis`]: struct.Analysis.html
//! [`FindOption`]: enum.FindOption.html
//! [`FoundOption`]: enum.FoundOption.html

use std::ffi::OsStr;

/// Analysis of parsing arguments
///
/// This type provides a set of “data-mining” methods for extracing information from the set of
/// wrapped items. Note that most such methods ignore problem items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis<'s> {
    /// Set of items describing what was found
    pub items: Vec<ItemClass<'s>>,
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
/// preserves order more simply. We wrap items with this class indicator for the advantages in
/// matching.
///
/// All sub-variants hold a `usize` value to be used for indicating the index of the argument at
/// which the item was found.
///
/// Most sub-variants also hold additional data. Long option sub-variants hold a string slice
/// reference to the matched option. Short option sub-variants hold the `char` matched. Options with
/// data arguments additionally hold a string slice reference to the data string matched, and in
/// some cases also a [`DataLocation`] variant. The [`Positional`] sub-variant holds a string slice
/// reference to the matched string.
///
/// [`Item`]: enum.Item.html
/// [`ItemW`]: enum.ItemW.html
/// [`ItemE`]: enum.ItemE.html
/// [`DataLocation`]: enum.DataLocation.html
/// [`Positional`]: enum.Item.html#variant.Positional
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ItemClass<'s> {
    /// Non-problematic item
    Ok(Item<'s>),
    /// Warn-level item
    Warn(ItemW<'s>),
    /// Error-level item
    Err(ItemE<'s>),
}

/// Non-problematic items. See [`ItemClass`](enum.ItemClass.html) documentation for details.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Item<'a> {
    /// Positional argument (not an option, command, or early terminator).
    Positional(usize, &'a OsStr),
    /// Early terminator (`--`) encountered.
    EarlyTerminator(usize),
    /// Long option match.
    Long(usize, &'a str),
    /// Long option match, with expected data argument.
    LongWithData{ i: usize, n: &'a str, d: &'a OsStr, l: DataLocation },
    /// Short option match.
    Short(usize, char),
    /// Short option match, with expected data argument.
    ShortWithData{ i: usize, c: char, d: &'a OsStr, l: DataLocation },
    /// Command match.
    Command(usize, &'a str),
}

/// Error-level items. See [`ItemClass`](enum.ItemClass.html) documentation for details.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ItemE<'a> {
    /// Long option match, but data argument missing [ERROR]
    LongMissingData(usize, &'a str),
    /// Short option match, but data argument missing [ERROR]
    ShortMissingData(usize, char),
    /// Ambiguous match with multiple long options. This only occurs when an exact match was not
    /// found, but multiple  abbreviated possible matches were found. [ERROR]
    AmbiguousLong(usize, &'a OsStr),
}

/// Warn-level items. See [`ItemClass`](enum.ItemClass.html) documentation for details.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ItemW<'a> {
    /// Looked like a long option, but no match [WARN]
    UnknownLong(usize, &'a OsStr),
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
    LongWithUnexpectedData{ i: usize, n: &'a str, d: &'a OsStr },
}

/// Used to describe where data was located, for options that require data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLocation {
    /// Found in the same argument (after an `=` for long options, or the remaining characters for a
    /// short option).
    SameArg,
    /// Found in the next argument.
    NextArg,
}

/// A *to find* option description
///
/// Used for describing an option to find to data-mining methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindOption<'a> {
    /// Long option only
    Long(&'a str),
    /// Short option only
    Short(char),
    /// Related short and long option pair
    Pair(char, &'a str),
}

/// A *found* option description
///
/// Used by data-mining methods that search for options to describe an option they found. This
/// represents either a short option **or** a long option identifier, never both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoundOption<'a> {
    /// Long option
    Long(&'a str),
    /// Short option
    Short(char),
}

impl<'a> FindOption<'a> {
    /// Check whether instance matches a given long option name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gong::analysis::FindOption;
    /// assert_eq!(true,  FindOption::Pair('a', "foo").matches_long("foo"));
    /// assert_eq!(false, FindOption::Pair('a', "foo").matches_long("bar"));
    /// assert_eq!(true,  FindOption::Long("foo").matches_long("foo"));
    /// assert_eq!(false, FindOption::Long("foo").matches_long("bar"));
    /// assert_eq!(false, FindOption::Short('a').matches_long("foo"));
    /// ```
    pub fn matches_long(&self, long: &str) -> bool {
        match *self {
            FindOption::Pair(_, l) | FindOption::Long(l) => { l == long },
            FindOption::Short(_) => false,
        }
    }

    /// Check whether instance matches a given short option character
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gong::analysis::FindOption;
    /// assert_eq!(true,  FindOption::Pair('a', "foo").matches_short('a'));
    /// assert_eq!(false, FindOption::Pair('a', "foo").matches_short('b'));
    /// assert_eq!(true,  FindOption::Short('a').matches_short('a'));
    /// assert_eq!(false, FindOption::Short('a').matches_short('b'));
    /// assert_eq!(false, FindOption::Long("foo").matches_short('a'));
    /// ```
    pub fn matches_short(&self, short: char) -> bool {
        match *self {
            FindOption::Pair(s, _) | FindOption::Short(s) => { s == short },
            FindOption::Long(_) => false,
        }
    }

    /// Copies the instance, keeping only the long option name, if available
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gong::analysis::FindOption;
    /// let long = FindOption::Long("foo");
    /// assert_eq!(Some(long), long.copy_long());
    /// assert_eq!(Some(long), FindOption::Pair('a', "foo").copy_long());
    /// assert_eq!(None,       FindOption::Short('a').copy_long());
    /// ```
    pub fn copy_long(&self) -> Option<Self> {
        match *self {
            FindOption::Pair(_, l) | FindOption::Long(l) => Some(FindOption::Long(l)),
            FindOption::Short(_) => None,
        }
    }

    /// Copies the instance, keeping only the short option character, if available
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gong::analysis::FindOption;
    /// let short = FindOption::Short('a');
    /// assert_eq!(Some(short), short.copy_short());
    /// assert_eq!(Some(short), FindOption::Pair('a', "foo").copy_short());
    /// assert_eq!(None,        FindOption::Long("foo").copy_short());
    /// ```
    pub fn copy_short(&self) -> Option<Self> {
        match *self {
            FindOption::Pair(s, _) | FindOption::Short(s) => Some(FindOption::Short(s)),
            FindOption::Long(_) => None,
        }
    }
}

impl<'a> From<super::options::LongOption<'a>> for FindOption<'a> {
    #[inline(always)]
    fn from(o: super::options::LongOption<'a>) -> Self {
        FindOption::Long(o.name)
    }
}

impl<'a> From<super::options::ShortOption> for FindOption<'a> {
    #[inline(always)]
    fn from(o: super::options::ShortOption) -> Self {
        FindOption::Short(o.ch)
    }
}

impl<'r, 's: 'r> Analysis<'s> {
    /// Create a new result set (mostly only useful internally and in test suite)
    #[doc(hidden)]
    pub fn new(size_guess: usize) -> Self {
        Self {
            items: Vec::with_capacity(size_guess),
            error: false,
            warn: false,
        }
    }

    /// Gives an iterator over all items
    #[inline]
    pub fn get_items(&'r self) -> impl Iterator<Item = &'r ItemClass<'s>> {
        self.items.iter()
    }

    /// Gives an iterator over any good (non-error/warn) items
    pub fn get_good_items(&'r self) -> impl Iterator<Item = &'r ItemClass<'s>> {
        self.items.iter()
            .filter(|i| match i {
                ItemClass::Ok(_) => true,
                _ => false,
            })
    }

    /// Gives an iterator over any problem (error/warn) items
    ///
    /// Note, since certain problems could cause subsequent arguments to be mis-interpretted, it may
    /// be preferable to ignore all but the first problem (and of course terminate the program after
    /// outputting a suitable error message for that problem). Note that the alternate
    /// [`get_first_problem`] method offers a cleaner way to get at only the first problem. Of
    /// course if the [`stop_on_problem`] setting is enabled, then only the first problem will have
    /// been captured anyway.
    ///
    /// [`get_first_problem`]: #method.get_first_problem
    /// [`stop_on_problem`]: ../parser/struct.Settings.html#structfield.stop_on_problem
    pub fn get_problem_items(&'r self) -> impl Iterator<Item = &'r ItemClass<'s>> {
        self.items.iter()
            .filter(|i| match i {
                ItemClass::Err(_) | ItemClass::Warn(_) => true,
                _ => false,
            })
    }

    /// Gets the first problem item, if any
    ///
    /// This gives you the first problematic item, if any. It may be more preferable than the
    /// [`get_problem_items`] method since certain problems could cause subsequent arguments to be
    /// mis-interpretted.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let analysis = gong::analysis::Analysis::new(0);
    /// if let Some(problem) = analysis.get_first_problem() {
    ///     // Deal with it (print error and end program)
    /// }
    /// ```
    ///
    /// [`get_problem_items`]: #method.get_problem_items
    #[inline]
    pub fn get_first_problem(&'r self) -> Option<&'r ItemClass<'s>> {
        self.get_problem_items().next()
    }

    /// Gives an iterator over any *positional* arguments
    ///
    /// They are served in the order given by the user.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let analysis = gong::analysis::Analysis::new(0);
    /// let positionals: Vec<_> = analysis.get_positionals().collect();
    /// ```
    pub fn get_positionals(&'r self) -> impl Iterator<Item = &'s OsStr> + 'r {
        self.items.iter()
            .filter_map(|i| match i {
                ItemClass::Ok(Item::Positional(_, s)) => Some(*s),
                _ => None,
            })
    }

    /// Discover if a specified option was used
    ///
    /// This is useful for instance to ask if a *flag* type option (e.g. `--help`) was used, though
    /// it is not restricted to *flag* type options.
    ///
//TODO: update if changing the corresponding functionality
    /// Note that problematic items are ignored. [`LongWithUnexpectedData`] is an exception.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let analysis = gong::analysis::Analysis::new(0);
    /// if analysis.option_used(gong::findopt!(@pair 'h', "help")) {
    ///     // Print help output and exit...
    /// }
    /// ```
    ///
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    pub fn option_used(&self, option: FindOption<'_>) -> bool {
        for item in &self.items {
            match *item {
                ItemClass::Ok(Item::Long(_, n)) |
                ItemClass::Ok(Item::LongWithData { n, .. }) |
//TODO: possibly remove, if we take a strict stance against all problems
                ItemClass::Warn(ItemW::LongWithUnexpectedData { n, .. }) => {
                    if option.matches_long(n) { return true; }
                },
                ItemClass::Ok(Item::Short(_, c)) |
                ItemClass::Ok(Item::ShortWithData { c, .. }) => {
                    if option.matches_short(c) { return true; }
                },
                _ => {},
            }
        }
        false
    }

    /// Count the number of times a specified option was used
    ///
    /// Useful for instance in the common use of short option `v` for specifing the verbosity level
    /// of a program’s output, where common behaviour is that the more times this short option is
    /// used, the greater the verbosity (up to a limit), hence the need to count how many times it
    /// occurred.
    ///
//TODO: update if changing the corresponding functionality
    /// Note that problematic items are ignored. [`LongWithUnexpectedData`] is an exception.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let analysis = gong::analysis::Analysis::new(0);
    /// let count = analysis.count_instances(gong::findopt!(@short 'v'));
    /// ```
    ///
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    pub fn count_instances(&self, option: FindOption<'_>) -> usize {
        let mut count = 0;
        for item in &self.items {
            match *item {
                // Reminder: one use of this function may be for users who actually want to enforce
                // a single-use option property, giving users of their program an error if certain
                // options are used multiple times. They may use this function to retrieve a count,
                // thus we must keep that in mind with what item types we respond to here, even
                // though we discourage such enforcement, prefering to just ignore for
                // non-data-taking options and just taking the last value provided otherwise.
                ItemClass::Ok(Item::Long(_, n)) |
                ItemClass::Ok(Item::LongWithData { n, .. }) |
//TODO: possibly remove, if we take a strict stance against all problems
                ItemClass::Warn(ItemW::LongWithUnexpectedData { n, .. }) => {
                    if option.matches_long(n) { count += 1; }
                },
                ItemClass::Ok(Item::Short(_, c)) |
                ItemClass::Ok(Item::ShortWithData { c, .. }) => {
                    if option.matches_short(c) { count += 1; }
                },
                _ => {},
            }
        }
        count
    }

    /// Gets the last value provided for the specified option
    ///
    /// An option could appear more than once within an argument list; This function allows you to
    /// get the *data value* of the last instance, ignoring those that appear earlier in the list
    /// (obviously this is only applicable to options that actually take a data value). In other
    /// words, this is useful for *single use* type options, i.e. where the value provided replaces
    /// any value provided earlier in the argument list, and thus you are only interested in the
    /// final (last) value. Contrast this with [`get_all_values`].
    ///
    /// This is a faster and more efficient option than calling `last()` on the iterator returned by
    /// the [`get_all_values`] method, since this hunts in reverse for the last instance then stops.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let analysis = gong::analysis::Analysis::new(0);
    /// let val = analysis.get_last_value(gong::findopt!(@long "output-format"));
    /// ```
    ///
    /// [`get_all_values`]: #method.get_all_values
    pub fn get_last_value(&'r self, option: FindOption<'r>) -> Option<&'s OsStr> {
        for item in self.items.iter().rev() {
            match *item {
                ItemClass::Ok(Item::LongWithData { n, ref d, .. }) => {
                    if option.matches_long(n) { return Some(d.clone()); }
                },
                ItemClass::Ok(Item::ShortWithData { c, ref d, .. }) => {
                    if option.matches_short(c) { return Some(d.clone()); }
                },
                _ => {},
            }
        }
        None
    }

    /// Gives an iterator over any and all values for the specified option
    ///
    /// An option could appear more than once within an argument list; This function allows you to
    /// collect the *data values* of all instances. In other words, this is to be used for options
    /// that are *multi-use*. Contrast this with [`get_last_value`](#method.get_last_value).
    ///
    /// Values are returned in the order given by the user.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let analysis = gong::analysis::Analysis::new(0);
    /// for val in analysis.get_all_values(gong::findopt!(@pair 'f', "foo")) {
    ///     // Do something with it...
    /// }
    /// ```
    pub fn get_all_values(&'r self, option: FindOption<'r>)
        -> impl Iterator<Item = &'s OsStr> + 'r
    {
        self.items.iter()
            .filter_map(move |i| match i {
                ItemClass::Ok(Item::LongWithData { n, d, .. }) => {
                    if option.matches_long(n) { Some(*d) } else { None }
                },
                ItemClass::Ok(Item::ShortWithData { c, d, .. }) => {
                    if option.matches_short(*c) { Some(*d) } else { None }
                },
                _ => None,
            })
    }

    /// Determines which of the specified options was used last
    ///
    /// This is similar to the [`get_bool_flag_state_multi`] method, but instead of providing
    /// separate positive and negative lists, only one list is given, and instead of returning a
    /// `bool`, the option name/character is given. This method also differs in not being limited to
    /// any particular kinds of options.
    ///
    /// Note that if a pair of a short and a long option are described together within a single
    /// [`FindOption`], and one of these are matched, only the one actually matched will be returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let analysis = gong::analysis::Analysis::new(0);
    /// let find = [ gong::findopt!(@pair 'c', "color"), gong::findopt!(@long "no-color") ];
    /// match analysis.get_last_used(&find) {
    ///     Some(gong::foundopt!(@short 'c')) |
    ///     Some(gong::foundopt!(@long "color")) => { /* positive flag used... */ },
    ///     Some(gong::foundopt!(@long "no-color")) => { /* negative flag used... */ },
    ///     _ => { /* None of those were used... */ },
    /// }
    /// ```
    ///
    /// [`FindOption`]: enum.FindOption.html
    /// [`get_bool_flag_state_multi`]: #method.get_bool_flag_state_multi
    pub fn get_last_used(&'r self, options: &'r [FindOption<'r>]) -> Option<FoundOption<'r>> {
        for item in self.items.iter().rev() {
            match *item {
                ItemClass::Ok(Item::Long(_, n)) |
                ItemClass::Ok(Item::LongWithData{ n, .. }) |
//TODO: possibly remove, if we take a strict stance against all problems
                ItemClass::Warn(ItemW::LongWithUnexpectedData { n, .. }) => {
                    for o in options.clone() {
                        if o.matches_long(n) { return Some(FoundOption::Long(&n)); }
                    }
                },
                ItemClass::Ok(Item::Short(_, c)) |
                ItemClass::Ok(Item::ShortWithData{ c, .. }) => {
                    for o in options.clone() {
                        if o.matches_short(c) { return Some(FoundOption::Short(c)); }
                    }
                },
                _ => {},
            }
        }
        None
    }

    /// Gets a boolean flag state per last instance used of the specified options
    ///
    /// This is useful for *boolean flag* style options, i.e. where you have a pair of *flag* type
    /// options (those that do not take a data value), where one represents a *positive* form and
    /// the other a *negative* form, e.g. `--foo` and `--no-foo`, or `--enable-foo` and
    /// `--disable-foo`. Within an argument list one or more instance of either form could exist and
    /// in any order, with the last instance being the one of importance, ultimately determining
    /// the state. This function allows you to get an answer as to which form (positive or negative)
    /// of an option was used last, if either, and thus determine the state for an option.
    ///
    /// The return value is `None` if neither of the specified options were used (in which case you
    /// must use a default value best suited to your application); `Some(true)` if the last used of
    /// those specified was of the *positive* form; and `Some(false)` if it was of the negative
    /// form.
    ///
    /// # Notes
    ///
    ///  - Typically a program may only ever offer a single flag option for each form (or one long
    ///    option for each and a short option that toggles the default state), as expected by this
    ///    function; however if you need more flexibility, see [`get_bool_flag_state_multi`].
    ///  - There is no restriction on naming, since you are in full control of the names searched
    ///    for, i.e. you could use `foo` for positive and `no-foo` for negative, or `with-foo` and
    ///    `without-foo`; and you can obviously change the style used per search.
    ///  - Only flag type options are searchable, data taking ones are ignored.
//TODO: update if changing the corresponding functionality
    ///  - With the potential [`LongWithUnexpectedData`] problematic item variant, the problem is
    ///    ignored, considering it a clean match.
    ///  - It is *undefined behaviour* for the positive and negative forms to specify the same short
    ///    option character or long option name. This function *may* **panic** in such situations
    ///    (currently only with a debug assertion and only in certain cases).
    ///
    /// # Example
    ///
    /// ```rust
    /// # let analysis = gong::analysis::Analysis::new(0);
    /// let val = analysis.get_bool_flag_state(
    ///         gong::findopt!(@pair 'c', "color"), // Positive (true)
    ///         gong::findopt!(@long "no-color")    // Negative (false)
    ///     ).unwrap_or(true);                      // Default
    /// ```
    ///
    /// [`get_bool_flag_state_multi`]: #method.get_bool_flag_state_multi
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    pub fn get_bool_flag_state(&self, positive: FindOption<'_>, negative: FindOption<'_>)
        -> Option<bool>
    {
        let set = vec![(positive, true), (negative, false)];
        #[allow(unused_variables)] //`o` only used in debug release for below block
        let (o, tag) = self._get_bool_flag_state(set.iter().cloned())?;
        #[cfg(debug_assertions)]
        {
            if tag == true { //We always check pos first, we know not in both if neg
                let mut ambiguous = false;
                match o {
                    FoundOption::Long(name) => {
                        if negative.matches_long(name) { ambiguous = true; }
                    },
                    FoundOption::Short(ch) => {
                        if negative.matches_short(ch) { ambiguous = true; }
                    },
                }
                debug_assert_eq!(false, ambiguous, "you put it in both forms");
            }
        }
        Some(tag)
    }

    /// Gets a boolean flag state per last instance used of the specified options
    ///
    /// This is an alternative to [`get_bool_flag_state`], taking slices of positive and negative
    /// options. It is intended for situations where there may be more than one long option name, or
    /// short option `char` available for one or both boolean forms. For instance, if you offer both
    /// `--no-foo` and `--nofoo` as negative forms of an option.
    ///
    /// See the documentation of [`get_bool_flag_state`] for further details.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let analysis = gong::analysis::Analysis::new(0);
    /// let val = analysis.get_bool_flag_state_multi(
    ///         &[ gong::findopt!(@pair 'c', "color") ],
    ///         &[ gong::findopt!(@long "no-color"), gong::findopt!(@long "nocolor") ]
    ///     ).unwrap_or(true);
    /// ```
    ///
    /// [`get_bool_flag_state`]: #method.get_bool_flag_state
    pub fn get_bool_flag_state_multi(&self, positive: &[FindOption<'_>], negative: &[FindOption<'_>])
        -> Option<bool>
    {
        // We chain the positive and negative iterators to find which option from either set
        // occurred last, tagging the positive options with `true` and the negatives `false`, by
        // mapping to a tuple. This “tag” can then be returned by the common function we call to let
        // us know which list the returned item belonged to, saving us from having to figure that
        // out, and we can return that tag directly as the answer.
        let pos_iter = positive.iter().map(|&o| (o, true));
        let neg_iter = negative.iter().map(|&o| (o, false));
        #[allow(unused_variables)] //`o` only used in debug release for below block
        let (o, tag) = self._get_bool_flag_state(pos_iter.chain(neg_iter))?;
        #[cfg(debug_assertions)]
        {
            if tag == true { //We always check pos list first, we know not in both lists if neg
                let opposite_list = negative;
                let mut ambiguous = false;
                match o {
                    FoundOption::Long(name) => {
                        for n in opposite_list {
                            if n.matches_long(name) { ambiguous = true; break; }
                        }
                    },
                    FoundOption::Short(ch) => {
                        for n in opposite_list {
                            if n.matches_short(ch) { ambiguous = true; break; }
                        }
                    },
                }
                debug_assert_eq!(false, ambiguous, "you put it in both lists");
            }
        }
        Some(tag)
    }

    /// Private common function to some methods
    ///
    /// The `options` param is an iterator over a tuple of a [`FindOption`] together with a boolean,
    /// with the boolean used as a tag to make positive/negative identification work and be more
    /// efficient.
    ///
    /// [`FindOption`]: enum.FindOption.html
    fn _get_bool_flag_state(&'r self, options: impl Iterator<Item = (FindOption<'r>, bool)> + Clone)
        -> Option<(FoundOption<'r>, bool)>
    {
        for item in self.items.iter().rev() {
            // Note, we deliberately ignore data-taking option variants here.
            match *item {
                ItemClass::Ok(Item::Long(_, n)) |
//TODO: possibly remove, if we take a strict stance against all problems
                ItemClass::Warn(ItemW::LongWithUnexpectedData { n, .. }) => {
                    for (o, tag) in options.clone() {
                        if o.matches_long(n) { return Some((FoundOption::Long(&n), tag)); }
                    }
                },
                ItemClass::Ok(Item::Short(_, c)) => {
                    for (o, tag) in options.clone() {
                        if o.matches_short(c) { return Some((FoundOption::Short(c), tag)); }
                    }
                },
                _ => {},
            }
        }
        None
    }
}

impl<'s> ItemClass<'s> {
    /// Returns `true` if `self` is `Ok` variant
    pub fn is_ok(&self) -> bool {
        match *self { ItemClass::Ok(_) => true, _ => false }
    }

    /// Returns `true` if `self` is `Err` variant
    pub fn is_err(&self) -> bool {
        match *self { ItemClass::Err(_) => true, _ => false }
    }

    /// Returns `true` if `self` is `Warn` variant
    pub fn is_warn(&self) -> bool {
        match *self { ItemClass::Warn(_) => true, _ => false }
    }
}
