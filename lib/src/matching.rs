// Copyright 2019 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Item matching components
//!
//! This module contains components to do with finding matches for a given item.

use std::ffi::OsStr;
#[cfg(any(unix, target_os = "redox"))]
pub(crate) use std::os::unix::ffi::OsStrExt;

/// Result of performing a lookup in a set
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SearchResult<T: PartialEq> {
    /// A perfect match was found
    Match(T),
    /// No match was found
    NoMatch,
}

/// Result of performing a lookup in a set using a name, which could involve abbreviated matching
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NameSearchResult<T: PartialEq> {
    /// A perfect match was found
    Match(T),
    /// No match was found
    NoMatch,
    /// A unique abbreviated match was found
    AbbreviatedMatch(T),
    /// No perfect match, but multiple abbreviated matches found (ambiguous abbreviated match)
    AmbiguousMatch,
}

impl<T: PartialEq> From<SearchResult<T>> for NameSearchResult<T> {
    #[inline]
    fn from(rhs: SearchResult<T>) -> Self {
        match rhs {
            SearchResult::Match(i) => NameSearchResult::Match(i),
            SearchResult::NoMatch => NameSearchResult::NoMatch,
        }
    }
}

impl<T: PartialEq> From<NameSearchResult<T>> for SearchResult<T> {
    #[inline]
    fn from(rhs: NameSearchResult<T>) -> Self {
        match rhs {
            NameSearchResult::Match(i) |
            NameSearchResult::AbbreviatedMatch(i) => SearchResult::Match(i),
            NameSearchResult::NoMatch |
            NameSearchResult::AmbiguousMatch => SearchResult::NoMatch,
        }
    }
}

impl<T: Copy + PartialEq> From<SearchResult<&T>> for SearchResult<T> {
    #[inline]
    fn from(rhs: SearchResult<&T>) -> Self {
        match rhs {
            SearchResult::Match(i) => SearchResult::Match(*i),
            SearchResult::NoMatch => SearchResult::NoMatch,
        }
    }
}

impl<T: Copy + PartialEq> From<NameSearchResult<&T>> for NameSearchResult<T> {
    #[inline]
    fn from(rhs: NameSearchResult<&T>) -> Self {
        match rhs {
            NameSearchResult::Match(i) => NameSearchResult::Match(*i),
            NameSearchResult::AbbreviatedMatch(i) => NameSearchResult::AbbreviatedMatch(*i),
            NameSearchResult::NoMatch => NameSearchResult::NoMatch,
            NameSearchResult::AmbiguousMatch => NameSearchResult::AmbiguousMatch,
        }
    }
}

/// Helper for finding an item with a particular `char` identifier
///
/// This looks for an item (of type `I`) within the set provided by the `haystack` iterator, where
/// the item’s `char` identifier matches that given by `needle`. The `get_char` parameter takes a
/// function (or closure) with which to access the identifier attribute of each item. Returned is
/// the reference to the (first) matching item, if there is one.
///
/// This is intended to be used for locating an exact match of a short option within a set.
///
/// # Examples
///
/// ```rust,ignore
/// # use std::ffi::OsStr;
/// use gong::matching::{find_by_char, SearchResult};
///
/// // A basic set, simply using an array of `ShortOption`
/// let set = [ gong::shortopt!(@flag 'a'), gong::shortopt!(@flag 'b') ];
///
/// assert_eq!(SearchResult::Match(&set[0]), find_by_char('a', set.iter(), |&o| o.ch));
/// assert_eq!(SearchResult::Match(&set[1]), find_by_char('b', set.iter(), |&o| o.ch));
/// assert_eq!(SearchResult::NoMatch,        find_by_char('c', set.iter(), |&o| o.ch));
/// ```
#[inline]
#[must_use]
pub fn find_by_char<'a, I>(needle: char, haystack: impl Iterator<Item = &'a I>,
    get_char: fn(&'a I) -> char) -> SearchResult<&'a I>
    where I: PartialEq + 'a
{
    for candidate in haystack {
        if get_char(candidate) == needle { return SearchResult::Match(candidate); }
    }
    SearchResult::NoMatch
}

/// Helper for finding an item with a particular name identifier
///
/// This looks for an item (of type `I`) within the set provided by the `haystack` iterator, where
/// the item’s name identifier matches exactly that given by `needle`. The `get_name` parameter
/// takes a function (or closure) with which to access the identifier attribute of each item.
/// Returned is the reference to the (first) matching item, if there is one.
///
/// This is intended to be used for locating an exact match of a long option or command name within
/// a set.
///
/// # Examples
///
/// ```rust,ignore
/// # use std::ffi::OsStr;
/// use gong::matching::{find_by_name, SearchResult};
///
/// // A basic set, simply using an array of `LongOption`
/// let set = [ gong::longopt!(@flag "foo"), gong::longopt!(@flag "bar") ];
///
/// // An example set of long option names to try and match
/// let names = [ &OsStr::new("foo"), &OsStr::new("bar"), &OsStr::new("foobar") ];
///
/// assert_eq!(SearchResult::Match(&set[0]), find_by_name(names[0], set.iter(), |&o| o.name));
/// assert_eq!(SearchResult::Match(&set[1]), find_by_name(names[1], set.iter(), |&o| o.name));
/// assert_eq!(SearchResult::NoMatch,        find_by_name(names[2], set.iter(), |&o| o.name));
/// ```
#[inline]
#[must_use]
pub fn find_by_name<'a, I>(needle: &OsStr, haystack: impl Iterator<Item = &'a I>,
    get_name: fn(&'a I) -> &'a str) -> SearchResult<&'a I>
    where I: PartialEq + 'a
{
    for candidate in haystack {
        if get_name(candidate) == needle { return SearchResult::Match(candidate); }
    }
    SearchResult::NoMatch
}

/// Helper for finding an item with a particular name identifier, allowing abbreviations
///
/// This is an alternate to [`find_by_name`], allowing for matching `needle` as an abbreviation of
/// an item’s name. For example, `fo` could match as an abbreviation of `foo`. See the documentation
/// of that function for general details.
///
/// An exact match will always override any abbreviated matches, and as an abbreviation it must
/// uniquely match a single item only, otherwise it will be considered ambiguous.
///
/// # Examples
///
/// ```rust,ignore
/// # use std::ffi::OsStr;
/// use gong::matching::{find_by_abbrev_name, NameSearchResult};
///
/// // A basic set, simply using an array of `LongOption`
/// let set = [ gong::longopt!(@flag "foo"), gong::longopt!(@flag "foobar") ];
///
/// // An example set of long option names to try and match
/// let names = [
///     &OsStr::new("foo"), &OsStr::new("fo"), &OsStr::new("foob"), &OsStr::new("bar"),
/// ];
///
/// assert_eq!(find_by_abbrev_name(names[0], set.iter(), |&o| o.name),
///            NameSearchResult::Match(&set[0]));
///
/// assert_eq!(find_by_abbrev_name(names[1], set.iter(), |&o| o.name),
///            NameSearchResult::AmbiguousMatch);
///
/// assert_eq!(find_by_abbrev_name(names[2], set.iter(), |&o| o.name),
///            NameSearchResult::AbbreviatedMatch(&set[1]));
///
/// assert_eq!(find_by_abbrev_name(names[3], set.iter(), |&o| o.name),
///            NameSearchResult::NoMatch);
/// ```
///
/// [`find_by_name`]: fn.find_by_name.html
#[must_use]
pub fn find_by_abbrev_name<'a, I>(needle: &OsStr, haystack: impl Iterator<Item = &'a I>,
    get_name: fn(&'a I) -> &'a str) -> NameSearchResult<&'a I>
    where I: PartialEq + 'a
{
    let mut result = NameSearchResult::NoMatch;
    for candidate in haystack {
        let cand_name = get_name(candidate);
        // Exact
        if cand_name == needle {
            // An exact match overrules a previously found partial match and ambiguity found with
            // multiple partial matches.
            result = NameSearchResult::Match(candidate);
            break;
        }
        // Abbreviated
        if result == NameSearchResult::AmbiguousMatch {
            continue;
        }
        let cand_name_osstr = OsStr::new(cand_name);
        if needle.len() < cand_name_osstr.len() {
            if &cand_name_osstr.as_bytes()[..needle.len()] == needle.as_bytes() {
                if result == NameSearchResult::NoMatch {
                    result = NameSearchResult::AbbreviatedMatch(candidate);
                }
                else if let NameSearchResult::AbbreviatedMatch(_) = result {
                    result = NameSearchResult::AmbiguousMatch;
                }
            }
        }
    }
    result
}

/// Suggestion matching implementation
///
/// Finds the best matching candidate, if any, from the given set, for the given string.
///
/// This is intended to be used when an unknown long option or command is encountered. Rather than
/// simply display an error to the users telling them it was unknown, you can include a helpful hint
/// in that error message suggesting an alternative option/command name that perhaps they meant to
/// use (whereby you suggest the closest match from the set available, if any is a close enough of a
/// match). I.e.:
///
/// > “Error: Unknown option ‘halp’, did you mean to use ‘help’?”
///
/// This helps you obtain that suggestion.
///
/// Specifically, this uses the `jaro_winkler` algorithm from the `strsim` crate; It filters
/// out any candidates with a metric calculated as less than `0.8`, and returns the first candidate
/// with the highest metric.
///
/// Similar to the “find_*” functions available in this module, this takes an iterator over a
/// generic set of items, along with a function for accessing the relevant string from them to
/// measure against.
///
/// # Examples
///
/// ```rust,ignore
/// use gong::matching::suggest;
///
/// // A basic set, simply using an array of `LongOption`
/// let set = [ gong::longopt!(@flag "foo"), gong::longopt!(@flag "bar") ];
///
/// assert_eq!(Some("bar"), suggest("bat", set.iter(), |&o| o.name));
/// assert_eq!(None,        suggest("xyz", set.iter(), |&o| o.name));
/// ```
#[cfg(feature = "suggestions")]
pub fn suggest<'a, 'b, I>(unknown: &str, available: impl Iterator<Item = &'a I>,
    get_name: fn(&'a I) -> &'b str) -> Option<&'b str>
    where 'b: 'a, I: 'a
{
    let filter = 0.8;
    let mut best_metric: f64 = filter;
    let mut best: Option<&str> = None;
    for candidate in available {
        let cand_name = get_name(candidate);
        let metric = strsim::jaro_winkler(unknown, cand_name);
        if metric > best_metric || (best.is_none() && metric >= filter) {
            best = Some(cand_name);
            best_metric = metric;
        }
    }
    best
}

#[cfg(windows)]
pub(crate) trait OsStrExt {
    fn from_bytes(slice: &[u8]) -> &Self;
    fn as_bytes(&self) -> &[u8];
}

#[cfg(windows)]
impl OsStrExt for OsStr {
    #[inline(always)]
    fn from_bytes(slice: &[u8]) -> &OsStr {
        unsafe { std::mem::transmute(slice) }
    }
    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::mem::transmute(self) }
    }
}
