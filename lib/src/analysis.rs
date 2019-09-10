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
//! [`ItemSet`], [`FindOption`] and [`FoundOption`], along with [`CommandAnalysis`] and
//! [`CommandBlockPart`] if your program makes use of *command arguments*.
//!
//! The “all in one” model is basically a wrapper around the iterative model, collecting the results
//! of running an iterator into a vector within an object ([`ItemSet`]) that exposes methods for
//! retrieving information (aka “data-mining”). For programs using *command arguments*, the items
//! are partitioned into blocks per use of commands, making use of the extra types just mentioned.
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
//! ## With “All in one” based analysis
//!
//! When it comes to the “all in one” model, as just mentioned, the set of items is partitioned
//! (i.e. “split up”) per use of commands into a vector of [`CommandBlockPart`], held within a
//! [`CommandAnalysis`] wrapper. [`Item::Command`] items are always consumed by this partitioning
//! process and so will never appear within an [`CommandBlockPart::ItemSet`] instance. This set of
//! “parts” will be a sequence of zero or more [`CommandBlockPart::Command`]s, interspersed
//! optionally with single [`CommandBlockPart::ItemSet`]s.
//!
//! In other words it will always look like this:
//!
//! > [items] [cmd [items]] [cmd [items]] ...
//!
//! Where the set of items after a command relate to and belong to that command, and a command
//! that follows another command is a sub-command of it.
//!
//! Note that each item set contains a reference to the relevant option set used to parse them, for
//! suggestion matching purposes. Similarly the analysis object itself holds a copy of the last
//! relevant command set reference (when applicable), for unknown-command suggestion matching.
//!
//! > **Tip:** If you encounter an unknown command item, you should be able to always safely unwrap
//! > the `Option` wrapper of the analysis object’s command set reference.
//!
//! Also note that *positional* arguments will naturally only ever be found within a
//! [`CommandBlockPart::ItemSet`] at the end.
//!
//! ## With iterative based analysis
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
//! [`Item::Command`]: struct.Item.html#variant.Command
//! [`ProblemItem`]: struct.ProblemItem.html
//! [`ItemResult`]: type.ItemResult.html
//! [`ItemSet`]: struct.ItemSet.html
//! [`CommandAnalysis`]: struct.CommandAnalysis.html
//! [`CommandBlockPart`]: enum.CommandBlockPart.html
//! [`CommandBlockPart::ItemSet`]: enum.CommandBlockPart.html#variant.ItemSet
//! [`CommandBlockPart::Command`]: enum.CommandBlockPart.html#variant.Command
//! [`FindOption`]: enum.FindOption.html
//! [`FoundOption`]: enum.FoundOption.html
//! [commands]: ../docs/ch4_commands/index.html

use std::ffi::OsStr;
use crate::positionals::Quantity as PositionalsQuantity;

pub type ItemResult<'set, 'arg> = Result<Item<'set, 'arg>, ProblemItem<'set, 'arg>>;
pub type ItemResultIndexed<'set, 'arg> = (usize, ItemResult<'set, 'arg>, Option<DataLocation>);

/// Non-problematic items
///
/// Long option variants hold a string slice reference to the matched option. Short option variants
/// hold the `char` matched. Options with data arguments additionally hold a string slice reference
/// to the data string matched (in `OsStr` form). The [`Positional`] variant holds a string slice
/// reference to the matched string (in `OsStr` form).
///
/// [`DataLocation`]: enum.DataLocation.html
/// [`Positional`]: #variant.Positional
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Item<'set, 'arg> {
    /// Positional argument (not an option, command, or early terminator).
    Positional(&'arg OsStr),
    /// Early terminator (`--`) encountered.
    EarlyTerminator,
    /// Long option match, possibly with a data argument.
    Long(&'set str, Option<&'arg OsStr>),
    /// Short option match, possibly with a data argument.
    Short(char, Option<&'arg OsStr>),
    /// Command match.
    Command(&'set str),
}

/// Problematic items
///
/// Long option variants hold a string slice reference of the matched/unmatched option name. Short
/// option variants hold the matched/unmatched short option `char`. The [`LongWithUnexpectedData`]
/// variant holds a string slice reference (in `OsStr` form) to the data string matched. Unknown
/// long option and command variants additionally come with a possible mismatch suggestion (if
/// the relevant setting and feature are enabled, and if a suitable suggestion is found).
///
/// [`LongWithUnexpectedData`]: #variant.LongWithUnexpectedData
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ProblemItem<'set, 'arg> {
    /// An unexpected positional
    UnexpectedPositional(&'arg OsStr),
    /// Missing positionals (quantity)
    MissingPositionals(PositionalsQuantity),
    /// Looked like a long option, but no match (mismatch suggestion possibly provided)
    UnknownLong(&'arg OsStr, Option<&'set str>),
    /// Unknown short option `char`
    UnknownShort(char),
    /// Unknown command (mismatch suggestion possibly provided)
    UnknownCommand(&'arg OsStr, Option<&'set str>),
    /// Ambiguous match with multiple long options. This only occurs when an exact match was not
    /// found, but multiple  abbreviated possible matches were found.
    AmbiguousLong(&'arg OsStr),
    /// Ambiguous match with multiple commands. This only occurs when an exact match was not found,
    /// but multiple  abbreviated possible matches were found.
    AmbiguousCmd(&'arg OsStr),
    /// Long option match, but data argument missing
    LongMissingData(&'set str),
    /// Short option match, but data argument missing
    ShortMissingData(char),
    /// Long option match, but came with unexpected data. For example `--foo=bar` when `--foo` takes
    /// no data. (The first string is the option name, the second the data).
    LongWithUnexpectedData(&'set str, &'arg OsStr),
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
/// This type provides a set of “data-mining” methods for extracting information from the set of
/// wrapped items. Note that most such methods ignore problem items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSet<'set, 'arg> {
    /// Set of items describing what was found
    pub items: Vec<ItemResult<'set, 'arg>>,
    /// Quick indication of problems (e.g. unknown options, or missing arg data)
    pub problems: bool,
}

impl<'set, 'arg> Default for ItemSet<'set, 'arg> {
    fn default() -> Self {
        Self { items: Vec::new(), problems: false, }
    }
}

/// Used for breaking up an analysis by command use
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandBlockPart<'set, 'arg> {
    /// A command
    Command(&'set str),
    /// Set of items describing what was found, up to the next use of a command
    ItemSet(ItemSet<'set, 'arg>),
}

/// Analysis of parsing arguments, partitioned per command use
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandAnalysis<'set, 'arg> {
    /// Partitioned analysis
    pub parts: Vec<CommandBlockPart<'set, 'arg>>,
    /// Quick indication of problems (e.g. unknown option, or missing arg data)
    pub problems: bool,
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
    #[must_use]
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
    #[must_use]
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

impl<'r, 'set: 'r, 'arg: 'r> ItemSet<'set, 'arg> {
    /// Is the problems indicator attribute `true`?
    #[inline(always)]
    pub fn has_problems(&self) -> bool {
        self.problems
    }

    /// Gives an iterator over all items in the set
    #[inline]
    pub fn get_items(&'r self) -> impl Iterator<Item = &'r ItemResult<'set, 'arg>> {
        self.items.iter()
    }

    /// Gives an iterator over any good (non-problematic) items
    pub fn get_good_items(&'r self) -> impl Iterator<Item = &'r Item<'set, 'arg>> {
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
    pub fn get_problem_items(&'r self) -> impl Iterator<Item = &'r ProblemItem<'set, 'arg>> {
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
    /// # let item_set = gong::analysis::ItemSet::default();
    /// if let Some(problem) = item_set.get_first_problem() {
    ///     // Deal with it (print error and end program)
    /// }
    /// ```
    ///
    /// [`get_problem_items`]: #method.get_problem_items
    #[inline]
    #[must_use]
    pub fn get_first_problem(&'r self) -> Option<&'r ProblemItem<'set, 'arg>> {
        self.get_problem_items().next()
    }

    /// Get a specific *positional*
    ///
    /// The `index` to provide relates to the set of *positionals* only, it is **not** the argument
    /// index. I.e. use `0` to get the first *positional*, `1` for the second, etc. Note that the
    /// indexing order is identical to the order in which they were given by the user.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::default();
    /// let _3rd_positional = item_set.get_positional(2);
    /// ```
    pub fn get_positional(&'r self, index: usize) -> Option<&'arg OsStr> {
        self.get_positionals().skip(index).next()
    }

    /// Gives an iterator over any *positional* arguments
    ///
    /// They are served in the order given by the user.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::default();
    /// let positionals: Vec<_> = item_set.get_positionals().collect();
    /// ```
    pub fn get_positionals(&'r self) -> impl Iterator<Item = &'arg OsStr> + 'r {
        self.items.iter()
            .filter_map(|i| match i {
                Ok(Item::Positional(s)) => Some(*s),
                _ => None,
            })
    }

    /// Gives an iterator over any unexpected *positional* arguments
    ///
    /// Should you want them for any reason.
    pub fn get_unexpected_positionals(&'r self) -> impl Iterator<Item = &'arg OsStr> + 'r {
        self.items.iter()
            .filter_map(|i| match i {
                Err(ProblemItem::UnexpectedPositional(s)) => Some(*s),
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
    /// # let item_set = gong::analysis::ItemSet::default();
    /// if item_set.option_used(gong::findopt!(@pair 'h', "help")) {
    ///     // Print help output and exit...
    /// }
    /// ```
    ///
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    #[must_use]
    pub fn option_used(&self, option: FindOption<'_>) -> bool {
        for item in &self.items {
            match *item {
                Ok(Item::Long(n, _)) |
//TODO: possibly remove, if we take a strict stance against all problems
                Err(ProblemItem::LongWithUnexpectedData(n, _)) => {
                    if option.matches_long(n) { return true; }
                },
                Ok(Item::Short(c, _)) => {
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
    /// # let item_set = gong::analysis::ItemSet::default();
    /// let count = item_set.count_instances(gong::findopt!(@short 'v'));
    /// ```
    ///
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    #[must_use]
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
                Ok(Item::Long(n, _)) |
//TODO: possibly remove, if we take a strict stance against all problems
                Err(ProblemItem::LongWithUnexpectedData(n, _)) => {
                    if option.matches_long(n) { count += 1; }
                },
                Ok(Item::Short(c, _)) => {
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
    /// # let item_set = gong::analysis::ItemSet::default();
    /// let last_value = item_set.get_last_value(gong::findopt!(@long "output-format"));
    /// ```
    ///
    /// [`get_all_values`]: #method.get_all_values
    #[must_use]
    pub fn get_last_value(&'r self, option: FindOption<'r>) -> Option<&'arg OsStr> {
        for item in self.items.iter().rev() {
            match *item {
//TODO: for optional-data taking options, we have `None` when no data given (e.g. `--foo`); if this was the last instance, do we want to return `None` to signal that the last use was without a value? but then of course how to distinuish between used without a value and option not used at all? should we expect users to do an option-used check for that?
                Ok(Item::Long(n, Some(ref d))) => {
                    if option.matches_long(n) { return Some(d.clone()); }
                },
                Ok(Item::Short(c, Some(ref d))) => {
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
    /// # let item_set = gong::analysis::ItemSet::default();
    /// for val in item_set.get_all_values(gong::findopt!(@pair 'f', "foo")) {
    ///     // Do something with it...
    /// }
    /// ```
    #[must_use]
    pub fn get_all_values(&'r self, option: FindOption<'r>)
        -> impl Iterator<Item = &'arg OsStr> + 'r
    {
//TODO: for optional-data taking options, we have `None` when no data given (e.g. `--foo`); such instances would now be completely ignored here, but is this really what is wanted? what if lack of value signals 'use default', and user program needs to know that this was signalled here?
        self.items.iter()
            .filter_map(move |i| match i {
                Ok(Item::Long(n, Some(d))) => {
                    if option.matches_long(n) { Some(*d) } else { None }
                },
                Ok(Item::Short(c, Some(d))) => {
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
    /// # let item_set = gong::analysis::ItemSet::default();
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
    #[must_use]
    pub fn get_last_used(&'r self, options: &'r [FindOption<'r>]) -> Option<FoundOption<'r>> {
        for item in self.items.iter().rev() {
            match *item {
                Ok(Item::Long(n, _)) |
//TODO: possibly remove, if we take a strict stance against all problems
                Err(ProblemItem::LongWithUnexpectedData(n, _)) => {
                    for o in options.clone() {
                        if o.matches_long(n) { return Some(FoundOption::Long(&n)); }
                    }
                },
                Ok(Item::Short(c, _)) => {
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
    ///  - Only flag type options should be used with this. Use with other option types is undefined
    ///    behaviour.
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
    /// # let item_set = gong::analysis::ItemSet::default();
    /// let val = item_set.get_bool_flag_state(
    ///         gong::findopt!(@pair 'c', "color"), // Positive (true)
    ///         gong::findopt!(@long "no-color")    // Negative (false)
    ///     ).unwrap_or(true);                      // Default
    /// ```
    ///
    /// [`get_bool_flag_state_multi`]: #method.get_bool_flag_state_multi
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    #[must_use]
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
    /// # let item_set = gong::analysis::ItemSet::default();
    /// let val = item_set.get_bool_flag_state_multi(
    ///         &[ gong::findopt!(@pair 'c', "color") ],
    ///         &[ gong::findopt!(@long "no-color"), gong::findopt!(@long "nocolor") ]
    ///     ).unwrap_or(true);
    /// ```
    ///
    /// [`get_bool_flag_state`]: #method.get_bool_flag_state
    #[must_use]
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
    #[must_use]
    fn _get_bool_flag_state(&'r self, options: impl Iterator<Item = (FindOption<'r>, bool)> + Clone)
        -> Option<(FoundOption<'r>, bool)>
    {
        for item in self.items.iter().rev() {
            // Note, we deliberately ignore data-taking option variants here.
            match *item {
                // Note, we attempt to restrict to flag style items only by specifying that the data
                // attribute must be `None`, but this also actually covers mixed type options when
                // no data provided. It is not possible to distinguish between the two (without the
                // data-location attribute, which the engine sets for mixed options despite always
                // being same-arg, specifically to help with this sort of situation, but unavailable
                // since not stored in the data-mining objects). We just have to put this down to
                // being a quirk of this function. Nothing can be done unless the extra data were
                // stored in the data-mining objects and we cared to check it.
                Ok(Item::Long(n, None)) |
//TODO: possibly remove, if we take a strict stance against all problems
                Err(ProblemItem::LongWithUnexpectedData(n, _)) => {
                    for (o, tag) in options.clone() {
                        if o.matches_long(n) { return Some((FoundOption::Long(&n), tag)); }
                    }
                },
                Ok(Item::Short(c, None)) => {
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

impl<'set, 'arg> CommandAnalysis<'set, 'arg> {
    /// Create a new result set (mostly only useful internally and in test suite)
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            parts: Vec::with_capacity(2), //Would not normally expect more than 2 or 3
            problems: false,
        }
    }

    /// Is the problems indicator attribute `true`?
    #[inline(always)]
    pub fn has_problems(&self) -> bool {
        self.problems
    }
}

impl<'r, 'set, 'arg, A> From<crate::engine::ParseIter<'r, 'set, 'arg, A>>
    for ItemSet<'set, 'arg>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    #[inline]
    fn from(iter: crate::engine::ParseIter<'r, 'set, 'arg, A>) -> Self {
        ItemSet::from(iter.indexed())
    }
}


impl<'r, 'set, 'arg, A> From<crate::engine::CmdParseIter<'r, 'set, 'arg, A>>
    for CommandAnalysis<'set, 'arg>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    #[inline]
    fn from(iter: crate::engine::CmdParseIter<'r, 'set, 'arg, A>) -> Self {
        CommandAnalysis::from(iter.indexed())
    }
}

impl<'r, 'set, 'arg, A> From<crate::engine::ParseIterIndexed<'r, 'set, 'arg, A>>
    for ItemSet<'set, 'arg>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    fn from(mut iter: crate::engine::ParseIterIndexed<'r, 'set, 'arg, A>) -> Self {
        let stop_on_problem = iter.get_parse_settings().stop_on_problem;
        let mut item_set = ItemSet::default();
        item_set.items = Vec::with_capacity(iter.size_hint().0);
        while let Some((_index, item, _dataloc)) = iter.next() {
            if let Err(_) = item {
                item_set.problems = true;
            }
            item_set.items.push(item);
            if stop_on_problem && item_set.problems {
                break;
            }
        }
        item_set
    }
}

impl<'r, 'set, 'arg, A> From<crate::engine::CmdParseIterIndexed<'r, 'set, 'arg, A>>
    for CommandAnalysis<'set, 'arg>
    where A: AsRef<OsStr> + 'arg, 'set: 'r, 'arg: 'r
{
    fn from(mut iter: crate::engine::CmdParseIterIndexed<'r, 'set, 'arg, A>) -> Self {
        let stop_on_problem = iter.get_parse_settings().stop_on_problem;
        let mut analysis = CommandAnalysis::new();
        let mut item_set = None;
        while let Some((_index, item, _dataloc)) = iter.next() {
            if let Ok(Item::Command(name)) = item {
                if item_set.is_some() {
                    analysis.parts.push(CommandBlockPart::ItemSet(item_set.take().unwrap()));
                }
                analysis.parts.push(CommandBlockPart::Command(name));
            }
            else {
                let item_set_ref = item_set.get_or_insert(ItemSet::default());
                if let Err(_) = item {
                    item_set_ref.problems = true;
                    analysis.problems = true;
                }
                item_set_ref.items.push(item);
                if stop_on_problem && item_set_ref.problems {
                    break;
                }
            }
        }
        if item_set.is_some() {
            analysis.parts.push(CommandBlockPart::ItemSet(item_set.take().unwrap()));
        }
        analysis
    }
}
