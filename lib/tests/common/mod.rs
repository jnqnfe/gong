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

use gong::analysis::Analysis;
use gong::parser::Parser;

/// Wrapper for actual analysis result
#[derive(Debug)] pub struct Actual<'a, 'b>(pub Analysis<'a, 'b>);
/// Wrapper for expected result, for comparison
#[derive(Debug)] pub struct Expected<'a, 'b>(pub Analysis<'a, 'b>);

/// Used for cleaner creation of set of test arguments
#[macro_export]
macro_rules! arg_list {
    ( $($e:expr),+ ) => { [ $(OsStr::new($e)),+ ] };
    ( $($e:expr,)+ ) => { [ $(OsStr::new($e)),+ ] };
}

/// Construct an `Expected`
macro_rules! expected {
    ( problems: $problems:expr, $(@itemset $item_set:expr),*, cmd_set: $cmd_set:expr ) => {
        Expected(Analysis {
            cmd_set: $cmd_set,
            problems: $problems,
            item_sets: vec![ $($item_set),* ],
        })
    };
}

/// Construct an `ItemSet`
macro_rules! item_set {
    ( cmd: $cmd:expr, opt_set: $opt_set:expr, problems: $problems:expr, $items:expr ) => {{
        let mut temp_vec = Vec::new();
        temp_vec.extend_from_slice(&$items);
        ItemSet { command: $cmd, opt_set: $opt_set, problems: $problems, items: temp_vec, }
    }};
}

/// Construct an `ItemClass` result item, for an `Expected`.
///
/// There is one matcher for each item type. The first param for each is the index to expect it to
/// be found at in the analysis. The second param is the label of the unique type. The final params
/// as necessary allow for: [<name/char>[, <data-value>, <data-location>]]
macro_rules! expected_item {
    ( $i:expr, Positional, $s:expr ) => { ItemClass::Ok(Item::Positional($i, OsStr::new($s))) };
    ( $i:expr, EarlyTerminator ) => { ItemClass::Ok(Item::EarlyTerminator($i)) };
    ( $i:expr, Long, $n:expr ) => { ItemClass::Ok(Item::Long($i, $n)) };
    ( $i:expr, Short, $c:expr ) => { ItemClass::Ok(Item::Short($i, $c)) };
    ( $i:expr, LongWithData, $n:expr, $d:expr, $l:expr ) => {
        ItemClass::Ok(Item::LongWithData { i: $i, n: $n, d: OsStr::new($d), l: $l })
    };
    ( $i:expr, ShortWithData, $c:expr, $d:expr, $l:expr ) => {
        ItemClass::Ok(Item::ShortWithData { i: $i, c: $c, d: OsStr::new($d), l: $l })
    };
    ( $i:expr, Command, $n:expr ) => { ItemClass::Ok(Item::Command($i, $n)) };
    ( $i:expr, UnknownLong, $n:expr ) => { ItemClass::Err(ProblemItem::UnknownLong($i, OsStr::new($n))) };
    ( $i:expr, UnknownShort, $c:expr ) => { ItemClass::Err(ProblemItem::UnknownShort($i, $c)) };
    ( $i:expr, UnknownCommand, $n:expr ) => { ItemClass::Err(ProblemItem::UnknownCommand($i, OsStr::new($n))) };
    ( $i:expr, LongWithUnexpectedData, $n:expr, $d:expr ) => {
        ItemClass::Err(ProblemItem::LongWithUnexpectedData { i: $i, n: $n, d: OsStr::new($d) })
    };
    ( $i:expr, LongMissingData, $n:expr ) => { ItemClass::Err(ProblemItem::LongMissingData($i, $n)) };
    ( $i:expr, ShortMissingData, $c:expr ) => { ItemClass::Err(ProblemItem::ShortMissingData($i, $c)) };
    ( $i:expr, AmbiguousLong, $n:expr ) => { ItemClass::Err(ProblemItem::AmbiguousLong($i, OsStr::new($n))) };
}

/// Construct a reference to an option set within a nested structure, from a base command set
///
/// E.g. ```cmdset_optset_ref!(get_base_cmds(), 2, 0)``` should give:
/// get_base_cmds().commands[2].sub_commands.commands[0].options
macro_rules! cmdset_optset_ref {
    ( @inner $base:expr, $index_last:expr ) => {
        $base.commands[$index_last].options
    };
    ( @inner $base:expr, $index_first:expr, $($index:expr),* ) => {
        cmdset_optset_ref!(@inner $base.commands[$index_first].sub_commands, $($index),*)
    };
    ( $base:expr, $($index:expr),* ) => {
        cmdset_optset_ref!(@inner $base, $($index),*)
    };
}

/// Construct a reference to a command set within a nested structure, from a base command set
///
/// E.g. ```cmdset_subcmdset_ref!(get_base_cmds(), 2, 0)``` should give:
/// &get_base_cmds().commands[2].sub_commands.commands[0].sub_commands
macro_rules! cmdset_subcmdset_ref {
    ( $base:expr, $($index:expr),* ) => { &$base$(.commands[$index].sub_commands)* }
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

/// Prints a pretty description of an `Analysis` struct, used in debugging for easier comparison
/// than with the raw output dumped by the test env.
///
/// Note, the `:#?` formatter is available as the “pretty” version of `:?`, but this is too sparse
/// an output, so we custom build a more compact version here.
fn pretty_print_results(analysis: &Analysis) {
    let mut item_sets = String::new();
    for item_set in &analysis.item_sets {
        let mut items = String::new();
        for item in &item_set.items {
            items.push_str(&format!("\n                {:?},", item));
        }
        item_sets.push_str(&format!("
        ItemSet {{
            command: {},
            items: [{}
            ],
            problems: {},
            opt_set: {:p},
        }}", item_set.command, items, item_set.problems, item_set.opt_set));
    }
    let cmd_set = match analysis.cmd_set {
        Some(cs) => format!("{:p}", cs),
        None => String::from("none"),
    };
    eprintln!("\
Analysis {{
    item_sets: [{}
    ],
    problems: {},
    cmd_set: {},
}}",
    item_sets, analysis.problems, cmd_set);
}
