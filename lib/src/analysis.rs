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
//! [`ItemSet`], [`FindOption`] and [`OptID`], along with [`CommandAnalysis`] and
//! [`CommandBlockPart`] if your program makes use of *command arguments*.
//!
//! The “all in one” model is basically a wrapper around the iterative model, collecting the results
//! of running an iterator into a vector within an object ([`ItemSet`]) that exposes methods for
//! retrieving information (aka “data-mining”). For programs using *command arguments*, the items
//! are partitioned into blocks per use of commands, making use of the extra types just mentioned.
//!
//! The [`FindOption`] type is simply one used with certain data-mining methods, for specifying an
//! option to be searched for. Note that a pair of a related long option and short option can be
//! specified together, which data mining methods will correctly consider. The [`OptID`] type is the
//! return type equivalent, which differs only in not having a long+short pair variant.
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
//! [`OptID`]: enum.OptID.html
//! [commands]: ../docs/ch4_commands/index.html

use std::ffi::OsStr;
use crate::positionals::Quantity as PositionalsQuantity;

pub type ItemResult<'set, 'arg> = Result<Item<'set, 'arg>, ProblemItem<'set, 'arg>>;
pub type ItemResultIndexed<'set, 'arg> = (usize, ItemResult<'set, 'arg>, Option<DataLocation>);

/// Option identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptID<'a> {
    /// Long option identifier
    Long(&'a str),
    /// Short option identifier
    Short(char),
}

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
    /// Option match (long or short), possibly with a data argument.
    Option(OptID<'set>, Option<&'arg OsStr>),
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
    /// A matched command
    Command(&'set str),
    /// A failed match against known commands - unknown (mismatch suggestion possibly provided)
    UnknownCommand(&'arg OsStr, Option<&'set str>),
    /// A failed abbreviated match against known commands - ambiguous
    AmbiguousCmd(&'arg OsStr),
    /// Set of all items that were found, up to the next use of a command name
    ItemSet(ItemSet<'set, 'arg>),
}

/// Analysis of parsing arguments, partitioned per command use
///
/// Note that:
///
///  - You will never see a sequence of two or more [`CommandBlockPart::ItemSet`]s; there will
///    always be a command name item inbetween.
///  - An unknown or ambiguous command name match variant will only ever be followed by an item set,
///    and only if the `stop_on_problem` setting is `false`.
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

impl<'a> FindOption<'a> {
    /// Check whether instance matches a given `OptID`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use gong::analysis::{FindOption, OptID};
    /// assert_eq!(true,  FindOption::Pair('a', "foo").matches(OptID::Long("foo")));
    /// assert_eq!(true,  FindOption::Pair('a', "foo").matches(OptID::Short('a')));
    /// assert_eq!(false, FindOption::Pair('a', "foo").matches(OptID::Long("bar")));
    /// assert_eq!(false, FindOption::Pair('a', "foo").matches(OptID::Short('b')));
    /// assert_eq!(true,  FindOption::Long("foo").matches(OptID::Long("foo")));
    /// assert_eq!(true,  FindOption::Short('a').matches(OptID::Short('a')));
    /// assert_eq!(false, FindOption::Long("foo").matches(OptID::Long("bar")));
    /// assert_eq!(false, FindOption::Short('a').matches(OptID::Short('b')));
    /// assert_eq!(false, FindOption::Short('a').matches(OptID::Long("foo")));
    /// assert_eq!(false, FindOption::Long("foo").matches(OptID::Short('a')));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn matches(&self, optid: OptID) -> bool {
        self.eq(&optid)
    }

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
        FindOption::Long(o.ident())
    }
}

impl From<super::options::ShortOption> for FindOption<'_> {
    #[inline(always)]
    fn from(o: super::options::ShortOption) -> Self {
        FindOption::Short(o.ident())
    }
}

impl<'a> From<super::options::OptionPair<'a>> for FindOption<'a> {
    #[inline(always)]
    fn from(o: super::options::OptionPair<'a>) -> Self {
        o.as_findopt()
    }
}

impl<'a> From<super::options::LongOption<'a>> for OptID<'a> {
    #[inline(always)]
    fn from(o: super::options::LongOption<'a>) -> Self {
        OptID::Long(o.ident())
    }
}

impl From<super::options::ShortOption> for OptID<'_> {
    #[inline(always)]
    fn from(o: super::options::ShortOption) -> Self {
        OptID::Short(o.ident())
    }
}

impl<'a> PartialEq<super::options::LongOption<'a>> for OptID<'a> {
    /// Tests that an `OptID` result matches a given long option
    #[inline]
    fn eq(&self, o: &super::options::LongOption<'a>) -> bool {
        match *self {
            OptID::Short(_) => false,
            OptID::Long(name) => name == o.ident(),
        }
    }
}

impl PartialEq<super::options::ShortOption> for OptID<'_> {
    /// Tests that an `OptID` result matches a given short option
    #[inline]
    fn eq(&self, o: &super::options::ShortOption) -> bool {
        match *self {
            OptID::Long(_) => false,
            OptID::Short(ch) => ch == o.ident(),
        }
    }
}

impl<'a> PartialEq<super::options::OptionPair<'a>> for OptID<'a> {
    /// Tests that an `OptID` result matches the corresponding identifier in a given option pair
    #[inline]
    fn eq(&self, o: &super::options::OptionPair<'a>) -> bool {
        match *self {
            OptID::Long(name) => name == o.ident_long(),
            OptID::Short(ch) => ch == o.ident_short(),
        }
    }
}

impl<'a> PartialEq<OptID<'a>> for super::options::LongOption<'a> {
    #[inline(always)]
    fn eq(&self, f: &OptID<'a>) -> bool {
        f.eq(self)
    }
}

impl PartialEq<OptID<'_>> for super::options::ShortOption {
    #[inline]
    fn eq(&self, f: &OptID<'_>) -> bool {
        f.eq(self)
    }
}

impl<'a> PartialEq<OptID<'a>> for super::options::OptionPair<'a> {
    #[inline]
    fn eq(&self, f: &OptID<'a>) -> bool {
        f.eq(self)
    }
}

impl<'a> PartialEq<FindOption<'a>> for OptID<'a> {
    /// Tests that an `OptID` result matches the corresponding identifier in a given `FindOption`
    fn eq(&self, f: &FindOption<'a>) -> bool {
        match *self {
            OptID::Long(name) => f.matches_long(name),
            OptID::Short(ch) => f.matches_short(ch),
        }
    }
}

impl<'a> PartialEq<OptID<'a>> for FindOption<'a> {
    #[inline(always)]
    fn eq(&self, f: &OptID<'a>) -> bool {
        f.eq(self)
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
    /// Note that problematic items are ignored, including [`LongWithUnexpectedData`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::default();
    /// let help_opt = gong::optpair!(@flag 'h', "help");
    /// if item_set.option_used(help_opt.into()) {
    ///     // Print help output and exit...
    /// }
    /// ```
    ///
    /// [`LongWithUnexpectedData`]: enum.ProblemItem.html#variant.LongWithUnexpectedData
    #[must_use]
    pub fn option_used(&self, option: FindOption<'_>) -> bool {
        for item in &self.items {
            match *item {
                Ok(Item::Option(id, _)) if option.matches(id) => { return true; },
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
    /// Note that problematic items are ignored, including [`LongWithUnexpectedData`].
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
                Ok(Item::Option(id, _)) if option.matches(id) => { count += 1; },
                _ => {},
            }
        }
        count
    }

    /// Count the number of times a specified option was used, up to a limit
    ///
    /// This is identical to [`count_instances`] except once the specified cap is reached, it stops
    /// counting.
    ///
    /// This if useful if you only need an accurate count up to a certain limit. For instance,
    /// revisiting the example of the common use of short option `-v` for verbosity, where multiple
    /// uses increases the amount of verbosity requested, there is naturally going to be a limit to
    /// the levels of verbosity possible, thus you may be interested in whether `-v` was used zero,
    /// one, two or three times, but three represents the maximum verbosity offered by your program
    /// and it would be pointless to keep counting beyond this. While [`count_instances`] would just
    /// keep pointlessly counting, this alternative will stop once the cap is reached.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::default();
    /// let cap = 3;
    /// let count = item_set.count_instances_capped(gong::findopt!(@short 'v'), cap);
    /// ```
    ///
    /// [`count_instances`]: #method.count_instances
    #[must_use]
    pub fn count_instances_capped(&self, option: FindOption<'_>, cap: usize) -> usize {
        let mut count = 0;
        for item in &self.items {
            if count == cap { break; } // Must be done fist to cover cap=0
            match *item {
                // Reminder: one use of this function may be for users who actually want to enforce
                // a single-use option property, giving users of their program an error if certain
                // options are used multiple times. They may use this function to retrieve a count,
                // thus we must keep that in mind with what item types we respond to here, even
                // though we discourage such enforcement, prefering to just ignore for
                // non-data-taking options and just taking the last value provided otherwise.
                Ok(Item::Option(id, _)) if option.matches(id) => { count += 1; },
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
    /// Note that with *mixed* type options (those where providing a value is optional) instances
    /// where no value is provided will be ignored and only the last actual value returned. If this
    /// is not what you want, use [`get_last_value_mixed`] instead.
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
    /// [`get_last_value_mixed`]: #method.get_last_value_mixed
    #[must_use]
    pub fn get_last_value(&'r self, option: FindOption<'r>) -> Option<&'arg OsStr> {
        for item in self.items.iter().rev() {
            // Note, this deliberately ignores mixed type options used without a value
            match *item {
                Ok(Item::Option(id, Some(ref d))) if option.matches(id) => {
                    return Some(d.clone());
                },
                _ => {},
            }
        }
        None
    }

    /// Gets the last value provided for the specified *mixed* type option
    ///
    /// This is identical to the [`get_last_value`] method except in one key respect: it takes a
    /// different approach to items that represent matches of *mixed* type options (those that
    /// optionally take data), where no data was provided. While the [`get_last_value`] method would
    /// ignore such items and retrieve the actual last value given, if any, this does not ignore
    /// such items and will consider the non-value of such an item to be the result sought.
    ///
    /// Note the double-`Option` wrapper on the return type:
    ///
    ///  - `None` signals that the given option was not used at all.
    ///  - `Some(None)` signals that the option was used and the last use was without a value.
    ///  - `Some(Some(_))` signals that the option was used and the last use was with the given
    ///     value.
    ///
    /// To clarify with an example, with arguments of `--foo=bar --foo` for a mixed type option
    /// `foo`, this method will return `Some(None)`, whilst [`get_last_value`] will return
    /// `Some("bar")`.
    ///
    /// Only use this method for *mixed* type options, and only if the above is what you seek. If
    /// you want non-value instances to be ignored then use [`get_last_value`] instead.
    ///
    /// Use of this function with *flag* type options is undefined behaviour. Use with mandatory
    /// data-taking options is less efficient.
    ///
    /// [`get_last_value`]: #method.get_last_value
    #[must_use]
    pub fn get_last_value_mixed(&'r self, option: FindOption<'r>) -> Option<Option<&'arg OsStr>> {
        for item in self.items.iter().rev() {
            match *item {
                Ok(Item::Option(id, d)) if option.matches(id) => { return Some(d.clone()); },
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
    /// Note that with *mixed* type options (those where providing a value is optional) instances
    /// where no value is provided will be ignored. If this is not what you want, use
    /// [`get_all_values_mixed`] instead.
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
    ///
    /// [`get_all_values_mixed`]: #method.get_all_values_mixed
    #[must_use]
    pub fn get_all_values(&'r self, option: FindOption<'r>)
        -> impl Iterator<Item = &'arg OsStr> + 'r
    {
        self.items.iter()
            // Note, this deliberately ignores mixed type options used without a value
            .filter_map(move |i| match i {
                Ok(Item::Option(id, Some(d))) => {
                    if option.matches(*id) { Some(*d) } else { None }
                },
                _ => None,
            })
    }

    /// Gives an iterator over any and all values for the specified *mixed* type option
    ///
    /// This is a variation of [`get_all_values`] in exactly the same respect as how
    /// [`get_last_value_mixed`] is a variation of [`get_last_value`], i.e this does not ignore
    /// instances of *mixed* type options used without a value. Note the `Option` wrapper around the
    /// string in the return type.
    ///
    /// For each iteration of the iterator:
    ///
    ///  - `None` indicates that there are no more results, i.e. the iterator is consumed, naturally.
    ///  - `Some(None)` indicates a use of the option without a value.
    ///  - `Some(Some(_))` indicates a use of the option with the provided value.
    ///
    /// Use of this function with *flag* type options is undefined behaviour. Use with mandatory
    /// data-taking options is less efficient.
    ///
    /// [`get_all_values`]: #method.get_all_values
    /// [`get_last_value_mixed`]: #method.get_last_value_mixed
    /// [`get_last_value`]: #method.get_last_value
    #[must_use]
    pub fn get_all_values_mixed(&'r self, option: FindOption<'r>)
        -> impl Iterator<Item = Option<&'arg OsStr>> + 'r
    {
        self.items.iter()
            .filter_map(move |i| match i {
                Ok(Item::Option(id, d)) => {
                    match option.matches(*id) {
                        false => None,
                        true => match d {
                            Some(d) => Some(Some(*d)),
                            None => Some(None),
                        },
                    }
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
    /// let col_pos_opt = gong::findopt!(@pair 'c', "color");
    /// let col_neg_opt = gong::findopt!(@long "no-color");
    /// if let Some(last) = item_set.get_last_used(&[ col_pos_opt, col_neg_opt ]) {
    ///     if last == col_pos_opt {
    ///         // positive flag used...
    ///     }
    ///     else if last == col_neg_opt {
    ///         // negative flag used...
    ///     }
    /// }
    /// else {
    ///     // None of those were used...
    /// }
    /// ```
    ///
    /// [`FindOption`]: enum.FindOption.html
    /// [`get_bool_flag_state_multi`]: #method.get_bool_flag_state_multi
    #[must_use]
    pub fn get_last_used(&'r self, options: &'r [FindOption<'r>]) -> Option<OptID<'r>> {
        for item in self.items.iter().rev() {
            match *item {
                Ok(Item::Option(id, _)) => {
                    if let OptID::Long(n) = id {
                        for o in options {
                            if o.matches_long(n) { return Some(OptID::Long(&n)); }
                        }
                    }
                    else if let OptID::Short(c) = id {
                        for o in options {
                            if o.matches_short(c) { return Some(OptID::Short(c)); }
                        }
                    }
                },
                _ => {},
            }
        }
        None
    }

    /// Determines which of the specified options was used first
    ///
    /// This is the same as the [`get_last_used`] method except that it obviously finds the first
    /// used instead of the last. It may be useful for instance in finding out which info option was
    /// requested first, if any, out of help and version, in order to respond to the first
    /// requested.
    ///
    /// Note that if a pair of a short and a long option are described together within a single
    /// [`FindOption`], and one of these are matched, only the one actually matched will be returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// # let opt_set = gong::option_set!();
    /// # let item_set = gong::analysis::ItemSet::default();
    /// let help_opt = gong::findopt!(@pair 'h', "help");
    /// let ver_opt = gong::findopt!(@pair 'V', "version");
    /// if let Some(first) = item_set.get_first_used(&[ help_opt, ver_opt ]) {
    ///     if first == help_opt {
    ///         // print help and exit
    ///     }
    ///     else if first == ver_opt {
    ///         // print version and exit
    ///     }
    /// }
    /// else {
    ///     // None of those were used...
    /// }
    /// ```
    ///
    /// [`FindOption`]: enum.FindOption.html
    /// [`get_last_used`]: #method.get_last_used
    #[must_use]
    pub fn get_first_used(&'r self, options: &'r [FindOption<'r>]) -> Option<OptID<'r>> {
        for item in self.items.iter() {
            match *item {
                Ok(Item::Option(id, _)) => {
                    if let OptID::Long(n) = id {
                        for o in options {
                            if o.matches_long(n) { return Some(OptID::Long(&n)); }
                        }
                    }
                    else if let OptID::Short(c) = id {
                        for o in options {
                            if o.matches_short(c) { return Some(OptID::Short(c)); }
                        }
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
    ///  - The [`LongWithUnexpectedData`] problematic item variant does not count as a match.
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
                debug_assert_eq!(false, negative.matches(o), "you put it in both forms");
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
                let mut ambiguous = false;
                for n in negative {
                    if n.matches(o) {
                        ambiguous = true;
                        break;
                    }
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
        -> Option<(OptID<'r>, bool)>
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
                Ok(Item::Option(id, None)) => {
                    if let OptID::Long(n) = id {
                        for (o, tag) in options.clone() {
                            if o.matches_long(n) { return Some((OptID::Long(&n), tag)); }
                        }
                    }
                    else if let OptID::Short(c) = id {
                        for (o, tag) in options.clone() {
                            if o.matches_short(c) { return Some((OptID::Short(c), tag)); }
                        }
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
            match item {
                Ok(Item::Command(_)) |
                Err(ProblemItem::UnknownCommand(_, _)) |
                Err(ProblemItem::AmbiguousCmd(_)) => {
                    if item_set.is_some() {
                        analysis.parts.push(CommandBlockPart::ItemSet(item_set.take().unwrap()));
                    }

                    if let Ok(Item::Command(name)) = item {
                        analysis.parts.push(CommandBlockPart::Command(name));
                    }
                    else {
                        analysis.problems = true;
                        if let Err(ProblemItem::UnknownCommand(name, sug)) = item {
                            analysis.parts.push(CommandBlockPart::UnknownCommand(name, sug));
                        }
                        else if let Err(ProblemItem::AmbiguousCmd(name)) = item {
                            analysis.parts.push(CommandBlockPart::AmbiguousCmd(name));
                        }
                        if stop_on_problem {
                            break;
                        }
                    }
                },
                _ => {
                    let item_set_ref = item_set.get_or_insert(ItemSet::default());
                    if let Err(_) = item {
                        item_set_ref.problems = true;
                        analysis.problems = true;
                    }
                    item_set_ref.items.push(item);
                    if stop_on_problem && item_set_ref.problems {
                        break;
                    }
                },
            }
        }
        if item_set.is_some() {
            analysis.parts.push(CommandBlockPart::ItemSet(item_set.take().unwrap()));
        }
        analysis
    }
}
