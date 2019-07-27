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

/// Find a match for something with a name (long option or command), optionally allowing for
/// abbreviations
pub fn find_name_match<'a, T>(needle: &OsStr, haystack: impl Iterator<Item = &'a T>,
    get_name: fn(&'a T) -> &'a str, abbreviations: bool) -> Result<Option<&'a T>, ()>
{
    let mut matched: Option<&T> = None;
    let mut ambiguity = false;
    for candidate in haystack {
        let cand_name = get_name(candidate);
        // Exact
        if cand_name == needle {
            // An exact match overrules a previously found partial match and ambiguity found with
            // multiple partial matches.
            matched = Some(candidate);
            ambiguity = false;
            break;
        }
        // Abbreviated
        else if abbreviations && !ambiguity {
            let cand_name_osstr = OsStr::new(cand_name);
            if needle.len() < cand_name_osstr.len() {
                if &cand_name_osstr.as_bytes()[..needle.len()] == needle.as_bytes() {
                    match matched {
                        Some(_) => { ambiguity = true; },
                        None => { matched = Some(candidate); },
                    }
                }
            }
        }
    }
    match ambiguity {
        true => Err(()),
        false => Ok(matched),
    }
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
