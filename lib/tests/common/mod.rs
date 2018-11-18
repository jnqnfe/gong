// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Shared stuff

pub mod base;
pub use self::base::{get_base_opts, get_base_cmds};

use std::ffi::OsStr;
use gong::analysis::Analysis;
use gong::parser::Parser;

/// Wrapper for actual analysis result
#[derive(Debug)] pub struct Actual<'a>(pub Analysis<'a, str>);
/// Wrapper for expected result, for comparison
#[derive(Debug)] pub struct Expected<'a>(pub Analysis<'a, str>);

/// Wrapper for actual analysis result
#[derive(Debug)] pub struct ActualOs<'a>(pub Analysis<'a, OsStr>);
/// Wrapper for expected result, for comparison
#[derive(Debug)] pub struct ExpectedOs<'a>(pub Analysis<'a, OsStr>);

/// Used for cleaner creation of set of test arguments
#[macro_export]
macro_rules! arg_list {
    ( $($e:expr),+ ) => { [ $($e),+ ] };
    ( $($e:expr,)+ ) => { [ $($e),+ ] };
}

/// Used for cleaner creation of set of test arguments
#[macro_export]
macro_rules! arg_list_os {
    ( $($e:expr),+ ) => { [ $(OsStr::new($e)),+ ] };
    ( $($e:expr,)+ ) => { [ $(OsStr::new($e)),+ ] };
}

/// Construct an `Expected`
macro_rules! expected {
    ( error: $e:expr, warn: $w:expr, $items:expr ) => {{
        let mut temp_vec = Vec::new();
        temp_vec.extend_from_slice(&$items);
        Expected(Analysis { error: $e, warn: $w, items: temp_vec, })
    }};
}

/// Construct an `ExpectedOs`
macro_rules! expected_os {
    ( error: $e:expr, warn: $w:expr, $items:expr ) => {{
        let mut temp_vec = Vec::new();
        temp_vec.extend_from_slice(&$items);
        ExpectedOs(Analysis { error: $e, warn: $w, items: temp_vec, })
    }};
}

/// Construct an `ItemClass` result item, for an `Expected`.
///
/// There is one matcher for each item type. The first param for each is the index to expect it to
/// be found at in the analysis. The second param is the label of the unique type. The final params
/// as necessary allow for: [<name/char>[, <data-value>, <data-location>]]
macro_rules! expected_item {
    ( $i:expr, NonOption, $s:expr ) => { ItemClass::Ok(Item::NonOption($i, $s)) };
    ( $i:expr, EarlyTerminator ) => { ItemClass::Ok(Item::EarlyTerminator($i)) };
    ( $i:expr, Long, $n:expr ) => { ItemClass::Ok(Item::Long($i, $n)) };
    ( $i:expr, Short, $c:expr ) => { ItemClass::Ok(Item::Short($i, $c)) };
    ( $i:expr, LongWithData, $n:expr, $d:expr, $l:expr ) => {
        ItemClass::Ok(Item::LongWithData { i: $i, n: $n, d: $d, l: $l })
    };
    ( $i:expr, ShortWithData, $c:expr, $d:expr, $l:expr ) => {
        ItemClass::Ok(Item::ShortWithData { i: $i, c: $c, d: $d, l: $l })
    };
    ( $i:expr, Command, $n:expr ) => { ItemClass::Ok(Item::Command($i, $n)) };
    ( $i:expr, UnknownLong, $n:expr ) => { ItemClass::Warn(ItemW::UnknownLong($i, $n)) };
    ( $i:expr, UnknownShort, $c:expr ) => { ItemClass::Warn(ItemW::UnknownShort($i, $c)) };
    ( $i:expr, LongWithNoName ) => { ItemClass::Warn(ItemW::LongWithNoName($i)) };
    ( $i:expr, LongWithUnexpectedData, $n:expr, $d:expr ) => {
        ItemClass::Warn(ItemW::LongWithUnexpectedData { i: $i, n: $n, d: $d })
    };
    ( $i:expr, LongMissingData, $n:expr ) => { ItemClass::Err(ItemE::LongMissingData($i, $n)) };
    ( $i:expr, ShortMissingData, $c:expr ) => { ItemClass::Err(ItemE::ShortMissingData($i, $c)) };
    ( $i:expr, AmbiguousLong, $n:expr ) => { ItemClass::Err(ItemE::AmbiguousLong($i, $n)) };
}

/// Get common base `Parser` set with common base option and command sets
pub fn get_parser() -> Parser<'static, 'static> {
    Parser::new(base::get_base_opts(), Some(base::get_base_cmds()))
}

/// Common central function for comparing actual analysis result with expected.
///
/// Benefits:
///
/// - Fewer uses of `assert_eq`, less likely to make a typo, putting `assert_ne` by mistake
/// - `Actual` and `Expected` wrappers help ensure correct comparison
/// - Central place where `pretty_print_results` can be enabled and called when desired in debugging
pub fn check_result(actual: &Actual, expected: &Expected) {
    if actual.0 != expected.0 {
        eprintln!("Actual:");
        pretty_print_results(&actual.0);
        eprintln!("Expected:");
        pretty_print_results(&expected.0);

        assert!(false, "analysis does not match what was expected!");
    }
}

/// Common central function for comparing actual analysis result with expected.
///
/// Benefits:
///
/// - Fewer uses of `assert_eq`, less likely to make a typo, putting `assert_ne` by mistake
/// - `Actual` and `Expected` wrappers help ensure correct comparison
/// - Central place where `pretty_print_results` can be enabled and called when desired in debugging
pub fn check_result_os(actual: &ActualOs, expected: &ExpectedOs) {
    if actual.0 != expected.0 {
        eprintln!("Actual:");
        pretty_print_results_os(&actual.0);
        eprintln!("Expected:");
        pretty_print_results_os(&expected.0);

        assert!(false, "analysis does not match what was expected!");
    }
}

/// Prints a pretty description of an `Analysis` struct, used in debugging for easier comparison
/// than with the raw output dumped by the test env.
///
/// Note, the `:#?` formatter is available as the “pretty” version of `:?`, but this is too sparse
/// an output, so we custom build a more compact version here.
fn pretty_print_results(analysis: &Analysis<str>) {
    let mut items = String::new();
    for item in &analysis.items {
        items.push_str(&format!("\n        {:?},", item));
    }
    eprintln!("\
Analysis {{
    items: [{}
    ],
    error: {},
    warn: {},
}}",
    items, analysis.error, analysis.warn);
}

/// Prints a pretty description of an `Analysis` struct, used in debugging for easier comparison
/// than with the raw output dumped by the test env.
///
/// Note, the `:#?` formatter is available as the “pretty” version of `:?`, but this is too sparse
/// an output, so we custom build a more compact version here.
fn pretty_print_results_os(analysis: &Analysis<OsStr>) {
    let mut items = String::new();
    for item in &analysis.items {
        items.push_str(&format!("\n        {:?},", item));
    }
    eprintln!("\
Analysis {{
    items: [{}
    ],
    error: {},
    warn: {},
}}",
    items, analysis.error, analysis.warn);
}
