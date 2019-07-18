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

use gong::analysis::{ItemSet, CommandAnalysis, CommandBlockPart};
use gong::commands::CommandSet;
use gong::parser::Parser;

/// Wrapper for actual analysis result
#[derive(Debug)] pub struct Actual<'a, 'b>(pub ItemSet<'a, 'b>);
/// Wrapper for expected result, for comparison
#[derive(Debug)] pub struct Expected<'a, 'b>(pub ItemSet<'a, 'b>);
/// Wrapper for actual analysis result (command partitioned)
#[derive(Debug)] pub struct CmdActual<'a, 'b>(pub CommandAnalysis<'a, 'b>);
/// Wrapper for expected result, for comparison (command partitioned)
#[derive(Debug)] pub struct CmdExpected<'a, 'b>(pub CommandAnalysis<'a, 'b>);

/// Used for cleaner creation of set of test arguments
#[macro_export]
macro_rules! arg_list {
    ( $($e:expr),+ ) => { [ $(OsStr::new($e)),+ ] };
    ( $($e:expr,)+ ) => { [ $(OsStr::new($e)),+ ] };
}

/// Construct an `Expected`
macro_rules! expected {
    ( problems: $problems:expr, opt_set: $opt_set:expr, $items:expr ) => {
        Expected(item_set!(problems: $problems, opt_set: $opt_set, $items))
    };
}

/// Construct an `CmdExpected`
macro_rules! cmd_expected {
    ( problems: $problems:expr, $(@part $part:expr),*, cmd_set: $cmd_set:expr ) => {
        CmdExpected(CommandAnalysis {
            parts: vec![ $($part),* ],
            problems: $problems,
            cmd_set: $cmd_set,
        })
    };
}

/// Construct an `ItemSet`
macro_rules! item_set {
    ( problems: $problems:expr, opt_set: $opt_set:expr, $items:expr ) => {{
        let mut temp_vec = Vec::new();
        temp_vec.extend_from_slice(&$items);
        ItemSet { items: temp_vec, problems: $problems, opt_set: $opt_set }
    }};
}

/// Construct a `CommandBlockPart`
macro_rules! cmd_part {
    ( command: $i:expr, $c:expr ) => { CommandBlockPart::Command($i, $c) };
    ( item_set: $is:expr ) => { CommandBlockPart::ItemSet($is) };
}

/// Construct an `ItemResult` result item, for an `Expected`.
///
/// There is one matcher for each item type. The first param for each is the index to expect it to
/// be found at in the analysis. The second param is the label of the unique type. The final params
/// as necessary allow for: [<name/char>[, <data-value>, <data-location>]]
macro_rules! expected_item {
    ( $i:expr, Positional, $s:expr ) => { Ok(Item::Positional($i, OsStr::new($s))) };
    ( $i:expr, EarlyTerminator ) => { Ok(Item::EarlyTerminator($i)) };
    ( $i:expr, Long, $n:expr ) => { Ok(Item::Long($i, $n, None)) };
    ( $i:expr, Short, $c:expr ) => { Ok(Item::Short($i, $c, None)) };
    ( $i:expr, LongWithData, $n:expr, $d:expr, $l:expr ) => {
        Ok(Item::Long($i, $n, Some((OsStr::new($d), $l))))
    };
    ( $i:expr, ShortWithData, $c:expr, $d:expr, $l:expr ) => {
        Ok(Item::Short($i, $c, Some((OsStr::new($d), $l))))
    };
    ( $i:expr, Command, $n:expr ) => { Ok(Item::Command($i, $n)) };
    ( $i:expr, UnknownLong, $n:expr ) => { Err(ProblemItem::UnknownLong($i, OsStr::new($n))) };
    ( $i:expr, UnknownShort, $c:expr ) => { Err(ProblemItem::UnknownShort($i, $c)) };
    ( $i:expr, UnknownCommand, $n:expr ) => { Err(ProblemItem::UnknownCommand($i, OsStr::new($n))) };
    ( $i:expr, LongWithUnexpectedData, $n:expr, $d:expr ) => {
        Err(ProblemItem::LongWithUnexpectedData { i: $i, n: $n, d: OsStr::new($d) })
    };
    ( $i:expr, LongMissingData, $n:expr ) => { Err(ProblemItem::LongMissingData($i, $n)) };
    ( $i:expr, ShortMissingData, $c:expr ) => { Err(ProblemItem::ShortMissingData($i, $c)) };
    ( $i:expr, AmbiguousLong, $n:expr ) => { Err(ProblemItem::AmbiguousLong($i, OsStr::new($n))) };
    ( $i:expr, AmbiguousCmd, $n:expr ) => { Err(ProblemItem::AmbiguousCmd($i, OsStr::new($n))) };
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

/// Get common base `Parser` set with common base option set and an empty command set
pub fn get_parser() -> Parser<'static, 'static> {
    get_parser_common(None)
}

/// Get common base `Parser` set with common base option and command sets
pub fn get_parser_cmd() -> Parser<'static, 'static> {
    get_parser_common(Some(base::get_base_cmds()))
}

#[inline]
fn get_parser_common(commands: Option<&'static CommandSet<'static, 'static>>) -> Parser<'static, 'static> {
    let mut parser = Parser::new(base::get_base_opts(), commands);
    parser.settings.set_stop_on_problem(false);
    parser
}

/// Check actual result matches expected
///
/// On failure, has details printed, and test failed via assert failure (with this being a macro the
/// source file and line number of where this was used will be printed, which is very helpful!)
macro_rules! check_result {
    ( $actual:expr, $expected:expr) => {{
        if $actual.as_expected($expected) == false {
            assert!(false, "analysis does not match what was expected!");
        }
    }}
}

impl<'a, 'b> Actual<'a, 'b> {
    pub fn as_expected(&self, expected: &Expected) -> bool {
        let equal = self.0 == expected.0;
        if !equal {
            eprintln!("Actual:");
            pretty_print_results(&self.0);
            eprintln!("Expected:");
            pretty_print_results(&expected.0);
        }
        equal
    }
}

impl<'a, 'b> CmdActual<'a, 'b> {
    pub fn as_expected(&self, expected: &CmdExpected) -> bool {
        let equal = self.0 == expected.0;
        if !equal {
            eprintln!("Actual:");
            pretty_print_cmd_results(&self.0);
            eprintln!("Expected:");
            pretty_print_cmd_results(&expected.0);
        }
        equal
    }
}

/// Prints a pretty description of an `Analysis` struct, used in debugging for easier comparison
/// than with the raw output dumped by the test env.
///
/// Note, the `:#?` formatter is available as the “pretty” version of `:?`, but this is too sparse
/// an output, so we custom build a more compact version here.
fn pretty_print_results(analysis: &ItemSet) {
    let mut items = String::new();
    for item in &analysis.items {
        items.push_str(&format!("\n            {:?},", item));
    }
    eprintln!("\
ItemSet {{
    items: [{}
    ],
    problems: {},
    opt_set: {:p},
}}",
    items, analysis.problems, analysis.opt_set);
}

/// Prints a pretty description of an `Analysis` struct, used in debugging for easier comparison
/// than with the raw output dumped by the test env.
///
/// Note, the `:#?` formatter is available as the “pretty” version of `:?`, but this is too sparse
/// an output, so we custom build a more compact version here.
fn pretty_print_cmd_results(analysis: &CommandAnalysis) {
    let mut parts = String::new();
    for part in &analysis.parts {
        match part {
            CommandBlockPart::Command(i, c) => {
                parts.push_str(&format!("\n        Command: ({}, {}),", i, c));
            },
            CommandBlockPart::ItemSet(s) => {
                let mut items = String::new();
                for item in &s.items {
                    items.push_str(&format!("\n                {:?},", item));
                }
                parts.push_str(&format!("
        ItemSet {{
            items: [{}
            ],
            problems: {},
            opt_set: {:p},
        }}",
                    items, s.problems, s.opt_set));
            },
        }
    }
    let cmd_set = match analysis.cmd_set {
        Some(cs) => format!("{:p}", cs),
        None => String::from("none"),
    };
    eprintln!("\
CommandAnalysis {{
    parts: [{}
    ],
    problems: {},
    cmd_set: {},
}}", parts, analysis.problems, cmd_set);
}
