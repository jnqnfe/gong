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
