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
//! The most fundamental components are the [`Item`] and [`ProblemItem`] enums, which represent all
//! of the good and problematic items respectively that might be found. Functions (including
//! iterators) that can return either typically do so with a `Result` wrapper ([`ItemResult`]).
//!
//! # “All in one” based analysis
//!
//! If you use the “one at a time” (iterative) parsing model, then the two just mentioned item types
//! are the only analysis types of relevance; However, if you use the “all in one” (aka
//! “data-mining”) model then there are some additional types that come into play, namely
//! [`Analysis`], [`ItemSet`], [`FindOption`] and [`FoundOption`].
//!
//! The “all in one” model is basically a wrapper around the iterative model, collecting the results
//! of running an iterator into a vector within an object ([`Analysis`]) that exposes methods for
//! retrieving information (aka “data-mining”). For programs using *command arguments*, the items
//! are partitioned into blocks per use of commands.
//!
//! The [`FindOption`] type is simply one used with certain data-mining methods, for specifying an
//! option to be searched for. Note that a pair of a related long option and short option can be
//! specified together, which data mining methods will correctly consider. The [`FoundOption`] type
//! is the return type equivalent, which differs only in not having a long+short pair variant.
//!
//! # Command arguments
//!
//! If your program makes use of *command arguments* then you should familiarise yourself with how
//! the presence of such an argument changes the option and command sets used to parse subsequent
//! arguments in an argument list, [as discussed separately][commands].
//!
//! When it comes to the “all in one” model, construction of an [`Analysis`] object involves
//! partitioning (i.e. “splitting”) the set of items returned by the internally-used iterator based
//! upon the presence of commands in the parsed argument list. The result of this is a vector of
//! [`ItemSet`], with each representing a command name (always an empty string for the first) and
//! the set of items up to the next use of a command. Command items themselves are consumed in the
//! process of building this. Note that each item set contains a reference to the relevant option
//! set used to parse that set of items, for suggestion matching purposes. Similarly the analysis
//! object itself holds a copy of the last relevant command set reference (when applicable), for
//! unknown-command suggestion matching.
//!
//! > **Tip:** If you encounter an unknown command item, you should be able to always safely unwrap
//! > the `Option` wrapper of the analysis object’s command set reference.
//!
//! For the “one at a time” (iterative) parsing model, note that 1) the iterator has methods for
//! getting a pointer to the current option/command set pointers, which you can use to perform
//! suggestion matching, and 2) there are also methods for setting those to be used for subsequent
//! iterations, which is useful if not wanting to describe the full command-option tree of your
//! program up front (for instance, you may want, upon encountering a command, to pass along the
//! iterator to a function dedicated to handling that command, which sets up the iterator with the
//! correct sets for handling any remaining arguments).
//!
//! [`Item`]: struct.Item.html
//! [`ProblemItem`]: struct.ProblemItem.html
//! [`ItemResult`]: type.ItemResult.html
//! [`ItemSet`]: struct.ItemSet.html
//! [`Analysis`]: struct.Analysis.html
//! [`FindOption`]: enum.FindOption.html
//! [`FoundOption`]: enum.FoundOption.html
//! [commands]: ../docs/commands/index.html

use std::ffi::OsStr;
use crate::commands::CommandSet;
use crate::options::OptionSet;

pub type ItemResult<'s> = Result<Item<'s>, ProblemItem<'s>>;

/// Analysis of parsing arguments
///
/// This type provides a set of “data-mining” methods for extracing information from the set of
/// wrapped items. Note that most such methods ignore problem items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis<'r, 's: 'r> {
    /// Set of item sets describing what was found, partitioned into sets by commands
    pub item_sets: Vec<ItemSet<'r, 's>>,
    /// Quick indication of problems (e.g. unknown option, or missing arg data)
    pub problems: bool,
    /// Pointer to the final command set, for use with suggestion matching an unknown command (which
    /// only applies to the first positional).
    pub cmd_set: Option<&'r CommandSet<'r, 's>>,
}

/// Non-problematic items
///
/// All variants hold a `usize` value to be used for indicating the index of the argument at which
/// the item was found.
///
/// Most variants also hold additional data. Long option variants hold a string slice reference to
/// the matched option. Short option variants hold the `char` matched. Options with data arguments
/// additionally hold a string slice reference to the data string matched (in `OsStr` form) and also
/// a [`DataLocation`] variant. The [`Positional`] variant holds a string slice reference to the
/// matched string (in `OsStr` form).
///
/// [`DataLocation`]: enum.DataLocation.html
/// [`Positional`]: #variant.Positional
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

/// Problematic items
///
/// All variants hold a `usize` value to be used for indicating the index of the argument at which
/// the item was found.
///
/// Most variants also hold additional data. Long option variants hold a string slice reference of
/// the matched/unmatched option name. Short option variants hold the matched/unmatched short option
/// `char`. The [`LongWithUnexpectedData`] variant holds a string slice reference (in `OsStr` form)
/// to the data string matched.
///
/// [`LongWithUnexpectedData`]: #variant.LongWithUnexpectedData
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProblemItem<'a> {
    /// Looked like a long option, but no match
    UnknownLong(usize, &'a OsStr),
    /// Unknown short option `char`
    UnknownShort(usize, char),
    /// Unknown command
    UnknownCommand(usize, &'a OsStr),
    /// Ambiguous match with multiple long options. This only occurs when an exact match was not
    /// found, but multiple  abbreviated possible matches were found.
    AmbiguousLong(usize, &'a OsStr),
    /// Ambiguous match with multiple commands. This only occurs when an exact match was not found,
    /// but multiple  abbreviated possible matches were found.
    AmbiguousCmd(usize, &'a OsStr),
    /// Long option match, but data argument missing
    LongMissingData(usize, &'a str),
    /// Short option match, but data argument missing
    ShortMissingData(usize, char),
    /// Long option match, but came with unexpected data. For example `--foo=bar` when `--foo` takes
    /// no data.
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

/// A set of analysis items
///
/// This type provides a set of “data-mining” methods for extracing information from the set of
/// wrapped items. Note that most such methods ignore problem items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSet<'r, 's: 'r> {
    /// The command this set of items is associated with (always an empty string for the first)
    pub command: &'s str,
    /// Set of items describing what was found
    pub items: Vec<ItemResult<'s>>,
    /// Quick indication of problems (e.g. unknown options, or missing arg data)
    pub problems: bool,
    /// Pointer to the option set, for use with suggestion matching of unknown options
    pub opt_set: &'r OptionSet<'r, 's>,
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

impl From<super::options::ShortOption> for FindOption<'_> {
    #[inline(always)]
    fn from(o: super::options::ShortOption) -> Self {
        FindOption::Short(o.ch)
    }
}

impl<'r, 's: 'r> ItemSet<'r, 's> {
    /// Create a new result set (mostly only useful internally and in test suite)
    #[doc(hidden)]
    pub fn new(command: &'s str, opt_set: &'r OptionSet<'r, 's>) -> Self {
        Self {
            command: command,
            items: Vec::new(),
            problems: false,
            opt_set: opt_set,
        }
    }

    /// Is the problems indicator attribute `true`?
    #[inline(always)]
    pub fn has_problems(&self) -> bool {
        self.problems
    }

    /// Gives an iterator over all items in the set
    #[inline]
    pub fn get_items(&'r self) -> impl Iterator<Item = &'r ItemResult<'s>> {
        self.items.iter()
    }

    /// Gives an iterator over any good (non-problematic) items
    pub fn get_good_items(&'r self) -> impl Iterator<Item = &'r Item<'s>> {
        self.items.iter()
            .filter(|i| match i {
                Ok(_) => true,
                _ => false,
            })
            .map(|i| match i {
                Ok(inner) => inner,
                _ => unreachable!(),
            })
    }

    /// Gives an iterator over any problem items
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
    pub fn get_problem_items(&'r self) -> impl Iterator<Item = &'r ProblemItem<'s>> {
        self.items.iter()
            .filter(|i| match i {
                Err(_) => true,
                _ => false,
            })
            .map(|i| match i {
                Err(inner) => inner,
                _ => unreachable!(),
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
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::new("", &opt_set);
    /// if let Some(problem) = item_set.get_first_problem() {
    ///     // Deal with it (print error and end program)
    /// }
    /// ```
    ///
    /// [`get_problem_items`]: #method.get_problem_items
    #[inline]
    pub fn get_first_problem(&'r self) -> Option<&'r ProblemItem<'s>> {
        self.get_problem_items().next()
    }

    /// Gives an iterator over any *positional* arguments
    ///
    /// They are served in the order given by the user.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::new("", &opt_set);
    /// let positionals: Vec<_> = item_set.get_positionals().collect();
    /// ```
    pub fn get_positionals(&'r self) -> impl Iterator<Item = &'s OsStr> + 'r {
        self.items.iter()
            .filter_map(|i| match i {
                Ok(Item::Positional(_, s)) => Some(*s),
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
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::new("", &opt_set);
    /// if item_set.option_used(gong::findopt!(@pair 'h', "help")) {
    ///     // Print help output and exit...
    /// }
    /// ```
    ///
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    pub fn option_used(&self, option: FindOption<'_>) -> bool {
        for item in &self.items {
            match *item {
                Ok(Item::Long(_, n)) |
                Ok(Item::LongWithData { n, .. }) |
//TODO: possibly remove, if we take a strict stance against all problems
                Err(ProblemItem::LongWithUnexpectedData { n, .. }) => {
                    if option.matches_long(n) { return true; }
                },
                Ok(Item::Short(_, c)) |
                Ok(Item::ShortWithData { c, .. }) => {
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
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::new("", &opt_set);
    /// let count = item_set.count_instances(gong::findopt!(@short 'v'));
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
                Ok(Item::Long(_, n)) |
                Ok(Item::LongWithData { n, .. }) |
//TODO: possibly remove, if we take a strict stance against all problems
                Err(ProblemItem::LongWithUnexpectedData { n, .. }) => {
                    if option.matches_long(n) { count += 1; }
                },
                Ok(Item::Short(_, c)) |
                Ok(Item::ShortWithData { c, .. }) => {
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
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::new("", &opt_set);
    /// let val = item_set.get_last_value(gong::findopt!(@long "output-format"));
    /// ```
    ///
    /// [`get_all_values`]: #method.get_all_values
    pub fn get_last_value(&'r self, option: FindOption<'r>) -> Option<&'s OsStr> {
        for item in self.items.iter().rev() {
            match *item {
                Ok(Item::LongWithData { n, ref d, .. }) => {
                    if option.matches_long(n) { return Some(d.clone()); }
                },
                Ok(Item::ShortWithData { c, ref d, .. }) => {
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
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::new("", &opt_set);
    /// for val in item_set.get_all_values(gong::findopt!(@pair 'f', "foo")) {
    ///     // Do something with it...
    /// }
    /// ```
    pub fn get_all_values(&'r self, option: FindOption<'r>)
        -> impl Iterator<Item = &'s OsStr> + 'r
    {
        self.items.iter()
            .filter_map(move |i| match i {
                Ok(Item::LongWithData { n, d, .. }) => {
                    if option.matches_long(n) { Some(*d) } else { None }
                },
                Ok(Item::ShortWithData { c, d, .. }) => {
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
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::new("", &opt_set);
    /// let find = [ gong::findopt!(@pair 'c', "color"), gong::findopt!(@long "no-color") ];
    /// match item_set.get_last_used(&find) {
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
                Ok(Item::Long(_, n)) |
                Ok(Item::LongWithData{ n, .. }) |
//TODO: possibly remove, if we take a strict stance against all problems
                Err(ProblemItem::LongWithUnexpectedData { n, .. }) => {
                    for o in options.clone() {
                        if o.matches_long(n) { return Some(FoundOption::Long(&n)); }
                    }
                },
                Ok(Item::Short(_, c)) |
                Ok(Item::ShortWithData{ c, .. }) => {
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
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::new("", &opt_set);
    /// let val = item_set.get_bool_flag_state(
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
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::new("", &opt_set);
    /// let val = item_set.get_bool_flag_state_multi(
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
                Ok(Item::Long(_, n)) |
//TODO: possibly remove, if we take a strict stance against all problems
                Err(ProblemItem::LongWithUnexpectedData { n, .. }) => {
                    for (o, tag) in options.clone() {
                        if o.matches_long(n) { return Some((FoundOption::Long(&n), tag)); }
                    }
                },
                Ok(Item::Short(_, c)) => {
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

impl<'r, 's: 'r> Analysis<'r, 's> {
    /// Create a new result set (mostly only useful internally and in test suite)
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            item_sets: Vec::with_capacity(2), //Would not normally expect more than 2 or 3
            problems: false,
            cmd_set: None,
        }
    }

    /// Is the problems indicator attribute `true`?
    #[inline(always)]
    pub fn has_problems(&self) -> bool {
        self.problems
    }

    /// Gives an iterator over any *positional* arguments
    ///
    /// This is a convenience function; positionals are always found only in the last item set.
    ///
    /// They are served in the order given by the user.
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let mut analysis = gong::analysis::Analysis::new();
    /// # analysis.item_sets.push(gong::analysis::ItemSet::new("", &opt_set));
    /// let positionals: Vec<_> = analysis.get_positionals().collect();
    /// ```
    #[inline]
    pub fn get_positionals(&'r self) -> impl Iterator<Item = &'s OsStr> + 'r {
        // NB. With the way commands are processed, only the last item set can contain positionals!
        self.item_sets.last().unwrap().get_positionals()
    }

    /// Gives an iterator over all items
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
    #[inline]
    pub fn get_items(&'r self) -> impl Iterator<Item = &'r ItemResult<'s>> {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].get_items()
    }

    /// Gives an iterator over any good (non-problematic) items
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
    #[inline]
    pub fn get_good_items(&'r self) -> impl Iterator<Item = &'r Item<'s>> {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].get_good_items()
    }

    /// Gives an iterator over any problem items
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
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
    #[inline]
    pub fn get_problem_items(&'r self) -> impl Iterator<Item = &'r ProblemItem<'s>> {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].get_problem_items()
    }

    /// Gets the first problem item, if any
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
    ///
    /// This gives you the first problematic item, if any. It may be more preferable than the
    /// [`get_problem_items`] method since certain problems could cause later arguments to be
    /// mis-interpretted.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let mut analysis = gong::analysis::Analysis::new();
    /// # analysis.item_sets.push(gong::analysis::ItemSet::new("", &opt_set));
    /// if let Some(problem) = analysis.get_first_problem() {
    ///     // Deal with it (print error and end program)
    /// }
    /// ```
    ///
    /// [`get_problem_items`]: #method.get_problem_items
    #[inline]
    pub fn get_first_problem(&'r self) -> Option<&'r ProblemItem<'s>> {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].get_first_problem()
    }

    /// Discover if a specified option was used
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
    ///
    /// This is useful for instance to ask if a *flag* type option (e.g. `--help`) was used, though
    /// it is not restricted to *flag* type options.
    ///
    /// Note that problematic items are ignored. [`LongWithUnexpectedData`] is an exception.
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let mut analysis = gong::analysis::Analysis::new();
    /// # analysis.item_sets.push(gong::analysis::ItemSet::new("", &opt_set));
    /// if analysis.option_used(gong::findopt!(@pair 'h', "help")) {
    ///     // Print help output and exit...
    /// }
    /// ```
    ///
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    #[inline]
    pub fn option_used(&self, option: FindOption<'_>) -> bool {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].option_used(option)
    }

    /// Count the number of times a specified option was used
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
    ///
    /// Useful for instance in the common use of short option `v` for specifing the verbosity level
    /// of a program’s output, where common behaviour is that the more times this short option is
    /// used, the greater the verbosity (up to a limit), hence the need to count how many times it
    /// occurred.
    ///
    /// Note that problematic items are ignored. [`LongWithUnexpectedData`] is an exception.
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let mut analysis = gong::analysis::Analysis::new();
    /// # analysis.item_sets.push(gong::analysis::ItemSet::new("", &opt_set));
    /// let count = analysis.count_instances(gong::findopt!(@short 'v'));
    /// ```
    ///
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    #[inline]
    pub fn count_instances(&self, option: FindOption<'_>) -> usize {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].count_instances(option)
    }

    /// Gets the last value provided for the specified option
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
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
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let mut analysis = gong::analysis::Analysis::new();
    /// # analysis.item_sets.push(gong::analysis::ItemSet::new("", &opt_set));
    /// let val = analysis.get_last_value(gong::findopt!(@long "output-format"));
    /// ```
    ///
    /// [`get_all_values`]: #method.get_all_values
    #[inline]
    pub fn get_last_value(&'r self, option: FindOption<'r>) -> Option<&'s OsStr> {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].get_last_value(option)
    }

    /// Gives an iterator over any and all values for the specified option
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
    ///
    /// An option could appear more than once within an argument list; This function allows you to
    /// collect the *data values* of all instances. In other words, this is to be used for options
    /// that are *multi-use*. Contrast this with [`get_last_value`](#method.get_last_value).
    ///
    /// Values are returned in the order given by the user.
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let mut analysis = gong::analysis::Analysis::new();
    /// # analysis.item_sets.push(gong::analysis::ItemSet::new("", &opt_set));
    /// for val in analysis.get_all_values(gong::findopt!(@pair 'f', "foo")) {
    ///     // Do something with it...
    /// }
    /// ```
    #[inline]
    pub fn get_all_values(&'r self, option: FindOption<'r>)
        -> impl Iterator<Item = &'s OsStr> + 'r
    {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].get_all_values(option)
    }

    /// Determines which of the specified options was used last
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
    ///
    /// This is similar to the [`get_bool_flag_state_multi`] method, but instead of providing
    /// separate positive and negative lists, only one list is given, and instead of returning a
    /// `bool`, the option name/character is given. This method also differs in not being limited to
    /// any particular kinds of options.
    ///
    /// Note that if a pair of a short and a long option are described together within a single
    /// `FindOption`, and one of these are matched, only the one actually matched will be returned.
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let mut analysis = gong::analysis::Analysis::new();
    /// # analysis.item_sets.push(gong::analysis::ItemSet::new("", &opt_set));
    /// let find = [ gong::findopt!(@pair 'c', "color"), gong::findopt!(@long "no-color") ];
    /// match analysis.get_last_used(&find) {
    ///     Some(gong::foundopt!(@short 'c')) |
    ///     Some(gong::foundopt!(@long "color")) => { /* positive flag used... */ },
    ///     Some(gong::foundopt!(@long "no-color")) => { /* negative flag used... */ },
    ///     _ => { /* None of those were used... */ },
    /// }
    /// ```
    ///
    /// [`get_bool_flag_state_multi`]: #method.get_bool_flag_state_multi
    #[inline]
    pub fn get_last_used(&'r self, options: &'r [FindOption<'r>]) -> Option<FoundOption<'r>> {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].get_last_used(options)
    }

    /// Gets a boolean flag state per last instance used of the specified options
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
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
    /// Notes:
    ///
    ///  - Typically a program may only ever offer a single flag option for each form (or one long
    ///    option for each and a short option that toggles the default state), as expected by this
    ///    function; however if you need more flexibility, see [`get_bool_flag_state_multi`].
    ///  - There is no restriction on naming, since you are in full control of the names searched
    ///    for, i.e. you could use `foo` for positive and `no-foo` for negative, or `with-foo` and
    ///    `without-foo`; and you can obviously change the style used per search.
    ///  - Only flag type options are searchable, data taking ones are ignored.
    ///  - With the potential [`LongWithUnexpectedData`] problematic item variant, the problem is
    ///    ignored, considering it a clean match.
    ///  - It is *undefined behaviour* for the positive and negative forms to specify the same short
    ///    option character or long option name. This function *may* **panic** in such situations
    ///    (currently only with a debug assertion and only in certain cases).
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let mut analysis = gong::analysis::Analysis::new();
    /// # analysis.item_sets.push(gong::analysis::ItemSet::new("", &opt_set));
    /// let val = analysis.get_bool_flag_state(
    ///         gong::findopt!(@pair 'c', "color"), // Positive (true)
    ///         gong::findopt!(@long "no-color")    // Negative (false)
    ///     ).unwrap_or(true);
    /// ```
    ///
    /// [`get_bool_flag_state_multi`]: #method.get_bool_flag_state_multi
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    #[inline]
    pub fn get_bool_flag_state(&self, positive: FindOption<'_>, negative: FindOption<'_>)
        -> Option<bool>
    {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].get_bool_flag_state(positive, negative)
    }

    /// Gets a boolean flag state per last instance used of the specified options
    ///
    /// Note: This method uses the first contained item set only; it is intended for programs that
    /// do **not** use command arguments. For programs that do use them, you must instead iterate
    /// over the item sets and use the methods on each item set.
    ///
    /// This is an alternative to [`get_bool_flag_state`], taking slices of positive and negative
    /// options. It is intended for situations where there may be more than one long option name, or
    /// short option `char` available for one or both boolean forms. For instance, if you offer both
    /// `--no-foo` and `--nofoo` as negative forms of an option.
    ///
    /// See the documentation of [`get_bool_flag_state`] for further details.
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let mut analysis = gong::analysis::Analysis::new();
    /// # analysis.item_sets.push(gong::analysis::ItemSet::new("", &opt_set));
    /// let val = analysis.get_bool_flag_state_multi(
    ///         &[ gong::findopt!(@pair 'c', "color") ],
    ///         &[ gong::findopt!(@long "no-color"), gong::findopt!(@long "nocolor") ]
    ///     ).unwrap_or(true);
    /// ```
    ///
    /// [`get_bool_flag_state`]: #method.get_bool_flag_state
    #[inline]
    pub fn get_bool_flag_state_multi(&self, positive: &[FindOption<'_>], negative: &[FindOption<'_>])
        -> Option<bool>
    {
        debug_assert_eq!(1, self.item_sets.len());
        self.item_sets[0].get_bool_flag_state_multi(positive, negative)
    }
}

impl<'r, 's, A> From<crate::engine::ParseIter<'r, 's, A>> for Analysis<'r, 's>
    where A: 's + AsRef<OsStr>, 's: 'r
{
    fn from(mut iter: crate::engine::ParseIter<'r, 's, A>) -> Self {
        let stop_on_problem = iter.get_parse_settings().stop_on_problem;
        let mut analysis = Analysis::new();
        let mut cmd = "";
        let mut more = true;
        let mut stop = false;
        while !stop && more {
            let mut item_set = ItemSet::new(cmd, iter.get_option_set());
            more = false;
            while let Some(item) = iter.next() {
                match item {
                    Err(_) => {
                        item_set.problems = true;
                        if stop_on_problem { stop = true; }
                    },
                    Ok(Item::Command(_, name)) => {
                        cmd = name;
                        more = true;
                        break;
                    },
                    Ok(_) => {},
                }
                item_set.items.push(item);
                if stop { break; }
            }
            analysis.problems |= item_set.problems;
            analysis.item_sets.push(item_set);
        }
        let cmd_set = iter.get_command_set();
        analysis.cmd_set = match cmd_set.is_empty() {
            false => Some(cmd_set),
            true => None,
        };
        analysis
    }
}
