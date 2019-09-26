// Copyright 2017 Lyndon Brown
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Test program for the `gong` library
//!
//! This test program takes user supplied command args, parses them with the library against a
//! set of example available options, and outputs a description of the results generated by the
//! parsing library.
//!
//! The analysis is output in color, where supported and applicable.
//!
//! See the `README.md` file for instructions.

#![doc(html_logo_url = "https://github.com/jnqnfe/gong/raw/master/logo.png",
       html_favicon_url = "https://github.com/jnqnfe/gong/raw/master/favicon.ico")]
#![doc(html_no_source)]

extern crate gong;
extern crate term_ctrl;

use std::ffi::OsStr;
use term_ctrl::predefined::*;
use gong::{longopt, shortopt, option_set};
use gong::analysis::{Item, ProblemItem, DataLocation, OptID};
use gong::arguments::Args;
use gong::options::{OptionType, OptionSet};
use gong::parser::{Parser, OptionsMode};
use gong::positionals::Policy as PositionalsPolicy;

const COL_HEADER: &str = combinations::fg_bold::MAGENTA;
const COL_O: &str = colours::fg::GREEN;  //okay
const COL_E: &str = colours::fg::RED;    //error
const COL_CHAR: &str = colours::fg::bright::BLUE;
const COL_MODE: &str = colours::fg::bright::BLUE;
const COL_DATA: &str = colours::fg::bright::YELLOW;

/// Config: Used for holding state of stdout formatting support
pub mod config {
    use std::sync::Once;
    use term_ctrl::support::use_fmt_stdout;

    static mut FORMATTED_STDOUT: bool = false;
    static INIT: Once = Once::new();

    pub fn init() {
        unsafe {
            INIT.call_once(|| {
                FORMATTED_STDOUT = use_fmt_stdout(cfg!(feature = "color"));
            });
        }
    }

    pub fn formatted_stdout() -> bool {
        unsafe { FORMATTED_STDOUT }
    }
}

// Color? Filter the provided “formatted-stdout-ctrl-seq” string
macro_rules! c {
    ( $code:expr ) => { if config::formatted_stdout() { $code } else { "" } };
}

static REAL_OPTIONS: OptionSet = option_set!(
    @long [
        longopt!(@flag "help"),
        longopt!(@flag "version"),
        longopt!(@flag "config"),
        longopt!(@flag "verbose"),
    ],
    @short [
        shortopt!(@flag 'h'),
        shortopt!(@flag 'V'),
        shortopt!(@flag 'c'),
        shortopt!(@flag 'v'),
    ]
);

static OPTIONS: OptionSet = option_set!(
    @long [
        longopt!(@flag "foo"),
        longopt!(@flag "verbose"), // Needed so verbose mode does not give error
        longopt!(@flag "joker"),
        longopt!(@flag "foobar"),
        longopt!(@data "hah"),
        longopt!(@flag "ábc"),
        longopt!(@mixed "delay"),
    ],
    @short [
        shortopt!(@flag 'k'),
        shortopt!(@flag '❤'),
        shortopt!(@flag 'x'),
        shortopt!(@flag 'v'), // Needed so verbose mode does not give error
        shortopt!(@data 'o'),
        shortopt!(@mixed 'p'),
        shortopt!(@mixed '💧'),
        shortopt!(@data 'Ɛ'),
    ]
);

#[cfg(not(feature = "pos_policy2"))]
const POSITIONALS_POLICY: PositionalsPolicy = PositionalsPolicy::Max(2);
#[cfg(feature = "pos_policy2")]
const POSITIONALS_POLICY: PositionalsPolicy = PositionalsPolicy::Min(2);

fn main() {
    config::init();

    let mut verbose = false;

    /* -- Handle real options -- */

    let args = Args::new();

    let mut real_parser = Parser::new(&REAL_OPTIONS);
    real_parser.set_positionals_policy(PositionalsPolicy::Fixed(0));
    real_parser.settings().set_mode(OptionsMode::Standard);
    debug_assert!(real_parser.is_valid());

    let mut iter = real_parser.parse_iter(&args);

    while let Some(item) = iter.next() {
        match item {
            Ok(Item::Option(id, None)) => {
                match id {
                    OptID::Short('h') | OptID::Long("help")    => { help();    return; },
                    OptID::Short('V') | OptID::Long("version") => { version(); return; },
                    OptID::Short('c') | OptID::Long("config")  => { config();  return; },
                    OptID::Short('v') | OptID::Long("verbose") => { verbose = true; },
                    _ => continue,
                }
            },
            // Could only happen with early terminator
            Ok(_) => { continue; },
            // If an error returns, it is likely just that user is not trying to use with real args
            // but instead use test mode, so fallthrough. Afterall, if a problem occurred, we could
            // not trust further parsing results to keep looking out for one of these options.
            Err(_) => { break; },
        }
    }

    /* -- Enter test parsing -- */

    let mut parser = Parser::new(&OPTIONS);
    parser.set_positionals_policy(POSITIONALS_POLICY);

    match cfg!(feature = "alt_mode") {
        true => { parser.settings().set_mode(OptionsMode::Alternate); },
        false => { parser.settings().set_mode(OptionsMode::Standard); },
    }
    parser.settings().set_allow_opt_abbreviations(!cfg!(feature = "no_opt_abbreviations"))
                     .set_posixly_correct(cfg!(feature = "posixly_correct"))
                     .set_report_earlyterm(true);

    debug_assert!(parser.is_valid());

    if verbose {
        println!();
        config();
        println!();
    }

    #[cfg(feature = "keep_prog_name")]
    let args = Args::from_vec(std::env::args_os().collect());
    #[cfg(not(feature = "keep_prog_name"))]
    let args = Args::from_vec(std::env::args_os().skip(1).collect());

    println!("[ {}Input arguments{} ]\n", c!(COL_HEADER), c!(RESET));

    match args.as_slice().len() {
        0 => println!("None!"),
        _ => for (i, arg) in args.as_slice().iter().enumerate() {
            println!("[{}]: {:?}", i, arg);
        },
    }

    let mut iter = parser.parse_iter(&args).indexed();
    let mut problems = false;
    let mut count_zero = true;

    println!("\n[ {}Analysis{} ]\n", c!(COL_HEADER), c!(RESET));

    while let Some((i, item, l)) = iter.next() {
        let printer = match item {
            Ok(_) => print_arg_ok,
            Err(_) => {
                problems = true;
                print_arg_err
            },
        };
        count_zero = false;
        match item {
            Ok(Item::Positional(s)) => printer(i, "Positional", s),
            Ok(Item::EarlyTerminator) => printer(i, "EarlyTerminator", OsStr::new("")),
            Ok(Item::Option(OptID::Long(n), None)) => {
                match l.is_none() {
                    true => printer(i, "Long", OsStr::new(&n)),
                    false => {
                        printer(i, "LongWithoutData", OsStr::new(&n));
                        print_data(l.unwrap(), None);
                    },
                }
            },
            Ok(Item::Option(OptID::Long(n), Some(d))) => {
                printer(i, "LongWithData", OsStr::new(&n));
                print_data(l.unwrap(), Some(d));
            },
            Err(ProblemItem::UnexpectedPositional(s)) => printer(i, "UnexpectedPositional", s),
            Err(ProblemItem::MissingPositionals(q)) => {
                print_arg_na_err("MissingPositionals");
                println!("    quantity: {}", q)
            },
            Err(ProblemItem::MissingOptionData(OptID::Long(n))) => printer(i, "LongMissingData", OsStr::new(&n)),
            Err(ProblemItem::LongWithUnexpectedData(n, d)) => {
                printer(i, "LongWithUnexpectedData", OsStr::new(&n));
                println!("    data: {:?}", d)
            },
            Err(ProblemItem::AmbiguousLong(n)) => printer(i, "AmbiguousLong", n),
            Err(ProblemItem::AmbiguousCmd(n)) => printer(i, "AmbiguousCmd", n),
            Err(ProblemItem::UnknownOption(OptID::Long(n), _)) => printer(i, "UnknownLong", OsStr::new(&n)),
            Ok(Item::Option(OptID::Short(c), None)) => {
                let desc = desc_char(c);
                match l.is_none() {
                    true => printer(i, "Short", OsStr::new(&desc)),
                    false => {
                        printer(i, "ShortWithoutData", OsStr::new(&desc));
                        print_data(l.unwrap(), None);
                    },
                }
            },
            Ok(Item::Option(OptID::Short(c), Some(d))) => {
                let desc = desc_char(c);
                printer(i, "ShortWithData", OsStr::new(&desc));
                print_data(l.unwrap(), Some(d));
            },
            Err(ProblemItem::MissingOptionData(OptID::Short(c))) => {
                let desc = desc_char(c);
                printer(i, "ShortMissingData", OsStr::new(&desc));
            },
            Err(ProblemItem::UnknownOption(OptID::Short(c), _)) => {
                let desc = desc_char(c);
                printer(i, "UnknownShort", OsStr::new(&desc));
            },
            Ok(Item::Command(n)) => printer(i, "Command", OsStr::new(&n)),
            Err(ProblemItem::UnknownCommand(n, _)) => printer(i, "UnknownCommand", OsStr::new(&n)),
        }
        #[cfg(feature = "stop_on_problem")] {
            if problems {
                break;
            }
        }
    }
    if !count_zero {
        println!();
    }
    else {
        println!("No arguments given!\n");
    }

    #[cfg(feature = "stop_on_problem")] {
        if problems {
            println!("Problem found, stopped early!\n");
        }
    }

    match problems {
        true => { println!("Problems: {}true{}", c!(COL_E), c!(RESET)); },
        false => {
            println!("Problems: {}false{}", c!(COL_O), c!(RESET));
        },
    }
}

fn print_arg(col: &str, index: usize, ty: &str, desc: &str) {
    println!("[arg {}] {}{}{}: {}", index, col, ty, c!(RESET), desc)
}

fn print_arg_na_err(ty: &str) {
    println!("[arg _] {}{}{}:", c!(COL_E), ty, c!(RESET))
}

fn print_arg_ok(index: usize, ty: &str, desc: &OsStr) {
    print_arg(c!(COL_O), index, ty, &desc.to_string_lossy());
}

fn print_arg_err(index: usize, ty: &str, desc: &OsStr) {
    print_arg(c!(COL_E), index, ty, &desc.to_string_lossy());
}

fn desc_char(ch: char) -> String {
    format!("{} {}({}){}", ch, c!(COL_CHAR), ch.escape_unicode(), c!(RESET))
}

fn print_data(loc: DataLocation, data: Option<&OsStr>) {
    match loc {
        DataLocation::SameArg =>
            println!("    {}data from SAME arg!{}", c!(effects::ITALIC), c!(RESET)),
        DataLocation::NextArg =>
            println!("    {}data from NEXT arg!{}", c!(effects::ITALIC), c!(RESET)),
    }
    match data {
        Some(d) => match d.is_empty() {
            true => println!("    {}empty-data{}", c!(effects::ITALIC), c!(RESET)),
            false => println!("    data: {:?}", d),
        },
        None => println!("    {}no-data{}", c!(effects::ITALIC), c!(RESET)),
    }
}

fn help() {
    println!("\
Playground test app for the `gong` argument parser.

This test app has an example application parser configuration (a built-in set of options and a
positionals policy). It parses any given input arguments against this and simply outputs the results
of parsing. Its purpose is for playing with the `gong` parser being used behind the scenes, testing
its capabilities.

USAGE:
    gong-playground [REAL-OPTIONS]
    gong-playground [-v|--verbose] [ARGUMENTS]

REAL-OPTIONS:
    -h, --help      Outputs this usage info.
    -c, --config    Outputs info regarding compile-time configuration and
                    the built-in options and such to test against.
    -V, --version   Outputs the version number of this test app.
    -v, --verbose   Include the config details with the main output.");
}

fn version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}

fn config() {
    println!("[ {}Config{} ]\n", c!(COL_HEADER), c!(RESET));

    #[cfg(not(feature = "alt_mode"))]
    println!("Option style: {}Standard{}", c!(COL_MODE), c!(RESET));
    #[cfg(feature = "alt_mode")]
    println!("Option style: {}Alternate{}", c!(COL_MODE), c!(RESET));

    #[cfg(not(feature = "posixly_correct"))]
    println!("Posixly correct?: {}No{}", c!(COL_MODE), c!(RESET));
    #[cfg(feature = "posixly_correct")]
    println!("Posixly correct?: {}Yes{}", c!(COL_MODE), c!(RESET));

    #[cfg(not(feature = "keep_prog_name"))]
    println!("Skip first argument (program name): {}true{}", c!(COL_MODE), c!(RESET));
    #[cfg(feature = "keep_prog_name")]
    println!("Skip first argument (program name): {}false{}", c!(COL_MODE), c!(RESET));

    #[cfg(not(feature = "no_opt_abbreviations"))]
    println!("Abbreviated option name matching: {}on{}", c!(COL_MODE), c!(RESET));
    #[cfg(feature = "no_opt_abbreviations")]
    println!("Abbreviated option name matching: {}off{}", c!(COL_MODE), c!(RESET));

    #[cfg(not(feature = "stop_on_problem"))]
    println!("Stop parsing upon problem: {}off{}", c!(COL_MODE), c!(RESET));
    #[cfg(feature = "stop_on_problem")]
    println!("Stop parsing upon problem: {}on{}", c!(COL_MODE), c!(RESET));

    println!("Positionals policy: {}{:?}{}", c!(COL_MODE), POSITIONALS_POLICY, c!(RESET));

    println!("\nCompile with different features to change the config!\n");

    println!("[ {}Test conditions{} ]\n", c!(COL_HEADER), c!(RESET));

    println!("Positionals policy: {:?}", POSITIONALS_POLICY);

    println!("Available options:");

    #[cfg(not(feature = "alt_mode"))]
    for item in OPTIONS.long {
        match item.ty() {
            OptionType::Flag => println!("    --{}", item.ident()),
            OptionType::Data => println!("    --{} {}[Data-taking]{}", item.ident(), c!(COL_DATA), c!(RESET)),
            OptionType::Mixed => println!("    --{} {}[Mixed]{}", item.ident(), c!(COL_DATA), c!(RESET)),
        }
    }
    #[cfg(feature = "alt_mode")]
    for item in OPTIONS.long {
        match item.ty() {
            OptionType::Flag => println!("    -{}", item.ident()),
            OptionType::Data => println!("    -{} {}[Data-taking]{}", item.ident(), c!(COL_DATA), c!(RESET)),
            OptionType::Mixed => println!("    -{} {}[Mixed]{}", item.ident(), c!(COL_DATA), c!(RESET)),
        }
    }
    #[cfg(not(feature = "alt_mode"))]
    for item in OPTIONS.short {
        match item.ty() {
            OptionType::Flag => println!("    -{}", desc_char(item.ident())),
            OptionType::Data => println!("    -{} {}[Data-taking]{}", desc_char(item.ident()), c!(COL_DATA), c!(RESET)),
            OptionType::Mixed => println!("    -{} {}[Mixed]{}", desc_char(item.ident()), c!(COL_DATA), c!(RESET)),
        }
    }

    println!("Available commands:");
    println!("    None!");
}
