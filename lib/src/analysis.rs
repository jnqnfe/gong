// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Analysis components

/// Analysis of processing arguments against an option set
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis<'a> {
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

impl<'a> Analysis<'a> {
    /// Create a new result set (mostly only useful internally)
    pub fn new(size_guess: usize) -> Self {
        Self {
            items: Vec::with_capacity(size_guess),
            error: false,
            warn: false,
        }
    }

    /// Add a new item to the analysis (mostly only useful internally)
    pub fn add(&mut self, item: ItemClass<'a>) {
        self.items.push(item);
    }
}
