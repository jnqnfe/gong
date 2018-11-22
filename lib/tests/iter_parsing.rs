// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Iterative style parsing tests
//!
//! Most of the test suite uses “all in one” style, and considering how it is built upon “iterative”
//! style, we do not need to do much testing of “iterative” style itself, but some things should be
//! checked, which is done here.

#[macro_use]
extern crate gong;

#[allow(unused_macros)]
#[allow(dead_code)] //Mod shared across test crates
#[macro_use]
mod common;

use std::ffi::OsStr;
use gong::analysis::*;
use common::get_parser;

/// Some general, basic argument handling
#[test]
fn basic() {
    let args = arg_list!(
        "abc",          // Non-option
        "--help",       // Long option
        "-bxs",         // Short option set, two unknown, one known (`x`)
        "--hah=xyz",    // Data taking option, in-same-arg
        "--ƒƒ", "cba",  // Data taking option, in-next-arg
        "-o123",        // Data taking short option, in-same-arg
        "-Ɛ", "456",    // Data taking short option, in-next-arg
    );
    let parser = get_parser();
    let mut parse_iter = parser.parse_iter(&args);
    assert_eq!(parse_iter.next(), Some(expected_item!(0, NonOption, "abc")));
    assert_eq!(parse_iter.next(), Some(expected_item!(1, Long, "help")));
    assert_eq!(parse_iter.next(), Some(expected_item!(2, UnknownShort, 'b')));
    assert_eq!(parse_iter.next(), Some(expected_item!(2, Short, 'x')));
    assert_eq!(parse_iter.next(), Some(expected_item!(2, UnknownShort, 's')));
    assert_eq!(parse_iter.next(), Some(expected_item!(3, LongWithData, "hah", "xyz", DataLocation::SameArg)));
    assert_eq!(parse_iter.next(), Some(expected_item!(4, LongWithData, "ƒƒ", "cba", DataLocation::NextArg)));
    assert_eq!(parse_iter.next(), Some(expected_item!(6, ShortWithData, 'o', "123", DataLocation::SameArg)));
    assert_eq!(parse_iter.next(), Some(expected_item!(7, ShortWithData, 'Ɛ', "456", DataLocation::NextArg)));
    assert_eq!(parse_iter.next(), None);
    assert_eq!(parse_iter.next(), None);
}

/// Some general, basic argument handling, same as before, but using `OsStr` based parsing
#[test]
fn basic_os() {
    let args = arg_list_os!(
        "abc",          // Non-option
        "--help",       // Long option
        "-bxs",         // Short option set, two unknown, one known (`x`)
        "--hah=xyz",    // Data taking option, in-same-arg
        "--ƒƒ", "cba",  // Data taking option, in-next-arg
        "-o123",        // Data taking short option, in-same-arg
        "-Ɛ", "456",    // Data taking short option, in-next-arg
    );
    let parser = get_parser();
    let mut parse_iter = parser.parse_iter_os(&args);
    assert_eq!(parse_iter.next(), Some(expected_item!(0, NonOption, OsStr::new("abc"))));
    assert_eq!(parse_iter.next(), Some(expected_item!(1, Long, "help")));
    assert_eq!(parse_iter.next(), Some(expected_item!(2, UnknownShort, 'b')));
    assert_eq!(parse_iter.next(), Some(expected_item!(2, Short, 'x')));
    assert_eq!(parse_iter.next(), Some(expected_item!(2, UnknownShort, 's')));
    assert_eq!(parse_iter.next(), Some(expected_item!(3, LongWithData, "hah", OsStr::new("xyz"), DataLocation::SameArg)));
    assert_eq!(parse_iter.next(), Some(expected_item!(4, LongWithData, "ƒƒ", OsStr::new("cba"), DataLocation::NextArg)));
    assert_eq!(parse_iter.next(), Some(expected_item!(6, ShortWithData, 'o', OsStr::new("123"), DataLocation::SameArg)));
    assert_eq!(parse_iter.next(), Some(expected_item!(7, ShortWithData, 'Ɛ', OsStr::new("456"), DataLocation::NextArg)));
    assert_eq!(parse_iter.next(), None);
    assert_eq!(parse_iter.next(), None);
}
