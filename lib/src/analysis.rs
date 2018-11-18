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

/// Analysis of parsing arguments
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis<'s, S: 's + ?Sized> {
    /// Set of items describing what was found
    pub items: Vec<ItemClass<'s, S>>,
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
#[derive(Debug, PartialEq, Eq)]
pub enum ItemClass<'s, S: 's + ?Sized> {
    /// Non-problematic item
    Ok(Item<'s, S>),
    /// Warn-level item
    Warn(ItemW<'s, S>),
    /// Error-level item
    Err(ItemE<'s, S>),
}

impl<'a, S: 'a + ?Sized> Copy for ItemClass<'a, S> {}
impl<'a, S: 'a + ?Sized> Clone for ItemClass<'a, S> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Non-problematic items. See [`ItemClass`](enum.ItemClass.html) documentation for details.
#[derive(Debug, PartialEq, Eq)]
pub enum Item<'a, S: 'a + ?Sized> {
    /// Argument not considered an option, command, or early terminator.
    NonOption(usize, &'a S),
    /// Early terminator (`--`) encountered.
    EarlyTerminator(usize),
    /// Long option match.
    Long(usize, &'a str),
    /// Long option match, with expected data argument.
    LongWithData{ i: usize, n: &'a str, d: &'a S, l: DataLocation },
    /// Short option match.
    Short(usize, char),
    /// Short option match, with expected data argument.
    ShortWithData{ i: usize, c: char, d: &'a S, l: DataLocation },
    /// Command match.
    Command(usize, &'a str),
}

impl<'a, S: 'a + ?Sized> Copy for Item<'a, S> {}
impl<'a, S: 'a + ?Sized> Clone for Item<'a, S> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Error-level items. See [`ItemClass`](enum.ItemClass.html) documentation for details.
#[derive(Debug, PartialEq, Eq)]
pub enum ItemE<'a, S: 'a + ?Sized> {
    /// Long option match, but data argument missing [ERROR]
    LongMissingData(usize, &'a str),
    /// Short option match, but data argument missing [ERROR]
    ShortMissingData(usize, char),
    /// Ambiguous match with multiple long options. This only occurs when an exact match was not
    /// found, but multiple  abbreviated possible matches were found. [ERROR]
    AmbiguousLong(usize, &'a S),
}

impl<'a, S: 'a + ?Sized> Copy for ItemE<'a, S> {}
impl<'a, S: 'a + ?Sized> Clone for ItemE<'a, S> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Warn-level items. See [`ItemClass`](enum.ItemClass.html) documentation for details.
#[derive(Debug, PartialEq, Eq)]
pub enum ItemW<'a, S: 'a + ?Sized> {
    /// Looked like a long option, but no match [WARN]
    UnknownLong(usize, &'a S),
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
    LongWithUnexpectedData{ i: usize, n: &'a str, d: &'a S },
}

impl<'a, S: 'a + ?Sized> Copy for ItemW<'a, S> {}
impl<'a, S: 'a + ?Sized> Clone for ItemW<'a, S> {
    fn clone(&self) -> Self {
        *self
    }
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

impl<'s, S: 's + ?Sized> Analysis<'s, S> {
    /// Create a new result set (mostly only useful internally)
    pub fn new(size_guess: usize) -> Self {
        Self {
            items: Vec::with_capacity(size_guess),
            error: false,
            warn: false,
        }
    }

    /// Add a new item to the analysis (mostly only useful internally)
    #[inline]
    pub fn add(&mut self, item: ItemClass<'s, S>) {
        self.items.push(item);
    }
}
