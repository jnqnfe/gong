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
use gong::parser::{Parser, CmdParser};

/// Wrapper for actual analysis result
#[derive(Debug)] pub struct Actual<'a, 'b, 'c>(pub ItemSet<'a, 'b, 'c>);
/// Wrapper for expected result, for comparison
#[derive(Debug)] pub struct Expected<'a, 'b, 'c>(pub ItemSet<'a, 'b, 'c>);
/// Wrapper for actual analysis result (command partitioned)
#[derive(Debug)] pub struct CmdActual<'a, 'b, 'c>(pub CommandAnalysis<'a, 'b, 'c>);
/// Wrapper for expected result, for comparison (command partitioned)
#[derive(Debug)] pub struct CmdExpected<'a, 'b, 'c>(pub CommandAnalysis<'a, 'b, 'c>);

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

/// Construct an `ItemResult`.
///
/// There is one matcher for each item type. The first param for each is the label of the unique
/// type. The final params as necessary allow for: [<name/char>[, <data-value>]].
macro_rules! item {
    ( Positional, $s:expr )             => { Ok(Item::Positional(OsStr::new($s))) };
    ( EarlyTerminator )                 => { Ok(Item::EarlyTerminator) };
    ( Long, $n:expr )                   => { Ok(Item::Long($n, None)) };
    ( Short, $c:expr )                  => { Ok(Item::Short($c, None)) };
    ( LongWithData, $n:expr, $d:expr )  => { Ok(Item::Long($n, Some(OsStr::new($d)))) };
    ( ShortWithData, $c:expr, $d:expr ) => { Ok(Item::Short($c, Some(OsStr::new($d)))) };
    ( Command, $n:expr )                => { Ok(Item::Command($n)) };
    ( UnknownLong, $n:expr )            => { Err(ProblemItem::UnknownLong(OsStr::new($n))) };
    ( UnknownShort, $c:expr )           => { Err(ProblemItem::UnknownShort($c)) };
    ( UnknownCommand, $n:expr )         => { Err(ProblemItem::UnknownCommand(OsStr::new($n))) };
    ( LongWithUnexpectedData, $n:expr, $d:expr )
                                        => { Err(ProblemItem::LongWithUnexpectedData($n, OsStr::new($d))) };
    ( LongMissingData, $n:expr )        => { Err(ProblemItem::LongMissingData($n)) };
    ( ShortMissingData, $c:expr )       => { Err(ProblemItem::ShortMissingData($c)) };
    ( AmbiguousLong, $n:expr )          => { Err(ProblemItem::AmbiguousLong(OsStr::new($n))) };
    ( AmbiguousCmd, $n:expr )           => { Err(ProblemItem::AmbiguousCmd(OsStr::new($n))) };
}

/// Construct an `ItemResultIndexed`.
///
/// There is one matcher for each item type. The first param for each is the index to expect it to
/// be found at in the analysis. The second param is the label of the unique type. The final params
/// as necessary allow for: [<name/char>[, <data-value>, <data-location>]].
macro_rules! indexed_item {
    ( $i:expr, Positional, $s:expr )       => { indexed_item!(@n $i, item!(Positional, $s)) };
    ( $i:expr, EarlyTerminator )           => { indexed_item!(@n $i, item!(EarlyTerminator)) };
    ( $i:expr, Long, $n:expr )             => { indexed_item!(@n $i, item!(Long, $n)) };
    ( $i:expr, Short, $c:expr )            => { indexed_item!(@n $i, item!(Short, $c)) };
    ( $i:expr, LongWithData, $n:expr, $d:expr, $l:expr )
                                           => { indexed_item!(@s $i, item!(LongWithData, $n, $d), $l) };
    ( $i:expr, ShortWithData, $c:expr, $d:expr, $l:expr )
                                           => { indexed_item!(@s $i, item!(ShortWithData, $c, $d), $l) };
    ( $i:expr, Command, $n:expr )          => { indexed_item!(@n $i, item!(Command, $n)) };
    ( $i:expr, UnknownLong, $n:expr )      => { indexed_item!(@n $i, item!(UnknownLong, $n)) };
    ( $i:expr, UnknownShort, $c:expr )     => { indexed_item!(@n $i, item!(UnknownShort, $c)) };
    ( $i:expr, UnknownCommand, $n:expr )   => { indexed_item!(@n $i, item!(UnknownCommand, $n)) };
    ( $i:expr, LongWithUnexpectedData, $n:expr, $d:expr )
                                           => { indexed_item!(@n $i, item!(LongWithUnexpectedData, $n, $d)) };
    ( $i:expr, LongMissingData, $n:expr )  => { indexed_item!(@n $i, item!(LongMissingData, $n)) };
    ( $i:expr, ShortMissingData, $c:expr ) => { indexed_item!(@n $i, item!(ShortMissingData, $c)) };
    ( $i:expr, AmbiguousLong, $n:expr )    => { indexed_item!(@n $i, item!(AmbiguousLong, $n)) };
    ( $i:expr, AmbiguousCmd, $n:expr )     => { indexed_item!(@n $i, item!(AmbiguousCmd, $n)) };

    // Inner: @n and @s are short for `None` and `Some(<data-location>)` respectively
    ( @n $i:expr, $item:expr )          => { ($i, $item, Option::<DataLocation>::None) };
    ( @s $i:expr, $item:expr, $l:expr ) => { ($i, $item, Some($l)) };
}

/// Construct an expected item for use in constructing data-mining objects.
///
/// There is one matcher for each item type. The first param for each is the index to expect it to
/// be found at in the analysis. The second param is the label of the unique type. The final params
/// as necessary allow for: [<name/char>[, <data-value>, <data-location>]].
macro_rules! dm_item {
    ( $i:expr, Positional, $s:expr )       => { indexed_item!($i, Positional, $s) };
    ( $i:expr, EarlyTerminator )           => { indexed_item!($i, EarlyTerminator) };
    ( $i:expr, Long, $n:expr )             => { indexed_item!($i, Long, $n) };
    ( $i:expr, Short, $c:expr )            => { indexed_item!($i, Short, $c) };
    ( $i:expr, LongWithData, $n:expr, $d:expr, $l:expr )
                                           => { indexed_item!($i, LongWithData, $n, $d, $l) };
    ( $i:expr, ShortWithData, $c:expr, $d:expr, $l:expr )
                                           => { indexed_item!($i, ShortWithData, $c, $d, $l) };
    ( $i:expr, Command, $n:expr )          => { indexed_item!($i, Command, $n) };
    ( $i:expr, UnknownLong, $n:expr )      => { indexed_item!($i, UnknownLong, $n) };
    ( $i:expr, UnknownShort, $c:expr )     => { indexed_item!($i, UnknownShort, $c) };
    ( $i:expr, UnknownCommand, $n:expr )   => { indexed_item!($i, UnknownCommand, $n) };
    ( $i:expr, LongWithUnexpectedData, $n:expr, $d:expr )
                                           => { indexed_item!($i, LongWithUnexpectedData, $n, $d) };
    ( $i:expr, LongMissingData, $n:expr )  => { indexed_item!($i, LongMissingData, $n) };
    ( $i:expr, ShortMissingData, $c:expr ) => { indexed_item!($i, ShortMissingData, $c) };
    ( $i:expr, AmbiguousLong, $n:expr )    => { indexed_item!($i, AmbiguousLong, $n) };
    ( $i:expr, AmbiguousCmd, $n:expr )     => { indexed_item!($i, AmbiguousCmd, $n) };
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
    let mut parser = Parser::new(base::get_base_opts());
    parser.settings.set_stop_on_problem(false);
    parser
}

/// Get common base `Parser` set with common base option and command sets
pub fn get_parser_cmd() -> CmdParser<'static, 'static> {
    let mut parser = CmdParser::new(base::get_base_opts(), base::get_base_cmds());
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

impl<'a, 'b, 'c> Actual<'a, 'b, 'c> {
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

impl<'a, 'b, 'c> CmdActual<'a, 'b, 'c> {
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
