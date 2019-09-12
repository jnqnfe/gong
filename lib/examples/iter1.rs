// Copyright 2019 Lyndon Brown
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Iterative example #1 for the `gong` library
//!
//! This example demonstrates use of:
//!
//!  - Iterative style parsing using a match block
//!  - A very simple **static** option set (for maximum efficiency)
//!  - A fixed string help text (for maximum efficiency)

extern crate gong;

use gong::{longopt, shortopt, option_set};
use gong::analysis::{Item, ProblemItem};
use gong::options::OptionSet;
use gong::parser::{Parser, OptionsMode};
use gong::positionals::Policy as PositionalsPolicy;

// Our options
//
// These are divided up into two lists, one for long options and one for short, for efficiency.
static OPTIONS: OptionSet = option_set!(
    @long [
        longopt!(@flag "help"),
        longopt!(@flag "version"),
    ],
    @short [
        shortopt!(@flag 'h'),
        shortopt!(@flag 'V'),
    ]
);

// A static help text string, for efficiency - no need to waste resources generating one dynamically
static HELP_TEXT: &str = "\
Simple example for the `gong` argument parser.

OPTIONS:
    -h, --help      Outputs this usage info.
    -V, --version   Outputs the version number of this test app.";

// Our program entry point
fn main() {
    // Collect our arguments
    let args: Vec<_> = std::env::args_os().skip(1).collect();

    // Setup our parser
    let mut parser = Parser::new(&OPTIONS);
    parser.set_positionals_policy(PositionalsPolicy::Fixed(0));
    parser.settings().set_mode(OptionsMode::Standard)
                     .set_allow_opt_abbreviations(true)
                     .set_report_earlyterm(false);
    debug_assert!(parser.is_valid());

    // Create the parsing iterator for our arguments
    let mut iter = parser.parse_iter(&args[..]);

    // Handle the results, iteratively
    //
    // Note how here we react to options like `--help` immediately upon encountering them,
    // disregarding any problem items that occur in subsequent arguments. It of course would be
    // trivial to remodel this.
    while let Some(item) = iter.next() {
        match item {
            Ok(Item::Short('h', None)) |
            Ok(Item::Long("help", None)) => {
                println!("{}", HELP_TEXT);
                return;
            },
            Ok(Item::Short('V', None)) |
            Ok(Item::Long("version", None)) => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return;
            },
            Err(ProblemItem::UnexpectedPositional(arg)) => {
                eprintln!("Error: Unexpected argument {:?}", arg);
                return;
            },
            Err(ProblemItem::UnknownLong(opt, _)) => {
                eprintln!("Error: Unknown option `{}`", opt.to_string_lossy());
                return;
            },
            Err(ProblemItem::UnknownShort(opt)) => {
                eprintln!("Error: Unknown option `{}`", opt);
                return;
            },
            Err(ProblemItem::AmbiguousLong(opt)) => {
                eprintln!("Error: Ambiguous abbreviated option name `{}`", opt.to_string_lossy());
                return;
            },
            Err(ProblemItem::LongWithUnexpectedData(opt, _)) => {
                eprintln!("Error: Option `{}` does not take data, but some was provided", opt);
                return;
            },
            Ok(Item::Short(_, _))                   |   // All real ones covered above
            Ok(Item::Long(_, _))                    |   // All real ones covered above
            Ok(Item::EarlyTerminator)               |   // Do not need to know
            Err(ProblemItem::LongMissingData(_))    |   // No data taking options!
            Err(ProblemItem::ShortMissingData(_))   |   // No data taking options!
            Ok(Item::Positional(_))                 |   // We do not take any here!
            Err(ProblemItem::MissingPositionals(_)) |   // Minimum is zero!
            Ok(Item::Command(_))                    |   // No commands here!
            Err(ProblemItem::AmbiguousCmd(_))       |   // No commands here!
            Err(ProblemItem::UnknownCommand(_, _)) => { // No commands here!
                unreachable!();
            },
        }
    }

    // The main functionality of our app here - just output a traditional friendly greeting
    println!("Hello, World!");
}
