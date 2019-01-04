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
use gong::analysis::{Item, ProblemItem, DataLocation};
use gong::parser::{Parser, OptionsMode};

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

fn main() {
    config::init();

    // Set up valid option descriptions
    let opts = option_set!(
        @long [
            longopt!("help"),
            longopt!("foo"),
            longopt!("version"),
            longopt!("foobar"),
            longopt!(@data "hah"),
            longopt!("ábc"),
        ],
        @short [
            shortopt!('h'),
            shortopt!('❤'),
            shortopt!('x'),
            shortopt!(@data 'o'),
        ]
    );
    let mut parser = Parser::new(&opts, None);

    match cfg!(feature = "alt_mode") {
        true => { parser.settings.set_mode(OptionsMode::Alternate); },
        false => { parser.settings.set_mode(OptionsMode::Standard); },
    }
    parser.settings.set_allow_abbreviations(!cfg!(feature = "no_abbreviations"))
                   .set_posixly_correct(cfg!(feature = "posixly_correct"));

    debug_assert!(parser.is_valid());

    println!("\n[ {}Config{} ]\n", c!(COL_HEADER), c!(RESET));

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

    #[cfg(not(feature = "no_abbreviations"))]
    println!("Abbreviated option name matching: {}on{}", c!(COL_MODE), c!(RESET));
    #[cfg(feature = "no_abbreviations")]
    println!("Abbreviated option name matching: {}off{}", c!(COL_MODE), c!(RESET));

    println!("\nCompile with different features to change the config!\n");

    println!("[ {}Available options for test{} ]\n", c!(COL_HEADER), c!(RESET));

    for item in opts.long {
        match item.expects_data {
            true => println!("LONG {} {}[expects data!]{}", item.name, c!(COL_DATA), c!(RESET)),
            false => println!("LONG {}", item.name),
        }
    }
    for item in opts.short {
        match item.expects_data {
            true => println!("SHORT {} {}[expects data!]{}", desc_char(item.ch), c!(COL_DATA), c!(RESET)),
            false => println!("SHORT {}", desc_char(item.ch)),
        }
    }

    #[cfg(feature = "alt_mode")]
    println!("\nNote: Short options will be ignored in `alternative` mode. They are still printed \
              so you can test and see this is so!");

    #[cfg(feature = "keep_prog_name")]
    let args: Vec<_> = std::env::args_os().collect();
    #[cfg(not(feature = "keep_prog_name"))]
    let args: Vec<_> = std::env::args_os().skip(1).collect();

    println!("\n[ {}Your input arguments{} ]\n", c!(COL_HEADER), c!(RESET));

    match args.len() {
        0 => println!("None!"),
        _ => for (i, arg) in args.iter().enumerate() {
            println!("[{}]: {:?}", i, arg);
        },
    }

    let results: Vec<_> = parser.parse_iter(&args[..]).collect();

    println!("\n[ {}Analysis{} ]\n", c!(COL_HEADER), c!(RESET));

    let mut problems = false;

    for item in &results {
        match *item {
            Err(_) => { problems = true; },
            Ok(_) => {},
        }
    }

    match problems {
        true => { println!("Problems: {}true{}", c!(COL_E), c!(RESET)); },
        false => {
            println!("Problems: {}false{}", c!(COL_O), c!(RESET));
        },
    }

    println!("Items: {}\n", results.len());
    for result in &results {
        let printer = match *result {
            Ok(_) => print_arg_ok,
            Err(_) => print_arg_err,
        };
        match result {
            Ok(Item::Positional(i, s)) => printer(*i, "Positional", s),
            Ok(Item::EarlyTerminator(i)) => printer(*i, "EarlyTerminator", OsStr::new("")),
            Ok(Item::Long(i, n)) => printer(*i, "Long", OsStr::new(&n)),
            Ok(Item::LongWithData { i, n, d, ref l }) => {
                printer(*i, "LongWithData", OsStr::new(&n));
                print_data(*l, d);
            },
            Err(ProblemItem::LongMissingData(i, n)) => printer(*i, "LongMissingData", OsStr::new(&n)),
            Err(ProblemItem::LongWithUnexpectedData { i, n, d }) => {
                printer(*i, "LongWithUnexpectedData", OsStr::new(&n));
                println!("    data: {:?}", d)
            },
            Err(ProblemItem::AmbiguousLong(i, n)) => printer(*i, "AmbiguousLong", n),
            Err(ProblemItem::UnknownLong(i, n)) => printer(*i, "UnknownLong", OsStr::new(&n)),
            Ok(Item::Short(i, c)) => {
                let desc = desc_char(*c);
                printer(*i, "Short", OsStr::new(&desc));
            },
            Ok(Item::ShortWithData { i, c, d, ref l }) => {
                let desc = desc_char(*c);
                printer(*i, "ShortWithData", OsStr::new(&desc));
                print_data(*l, d);
            },
            Err(ProblemItem::ShortMissingData(i, c)) =>{
                let desc = desc_char(*c);
                printer(*i, "ShortMissingData", OsStr::new(&desc));
            },
            Err(ProblemItem::UnknownShort(i, c)) =>{
                let desc = desc_char(*c);
                printer(*i, "UnknownShort", OsStr::new(&desc));
            },
            Ok(Item::Command(i, n)) => printer(*i, "Command", OsStr::new(&n)),
            Err(ProblemItem::UnknownCommand(i, n)) => printer(*i, "UnknownCommand", OsStr::new(&n)),
        }
    }
    if results.len() != 0 {
        println!();
    }
}

fn print_arg(col: &str, index: usize, ty: &str, desc: &str) {
    println!("[arg {}] {}{}{}: {}", index, col, ty, c!(RESET), desc)
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

fn print_data(loc: DataLocation, data: &OsStr) {
    match loc {
        DataLocation::SameArg =>
            println!("    {}data found in SAME arg!{}", c!(effects::ITALIC), c!(RESET)),
        DataLocation::NextArg =>
            println!("    {}data found in NEXT arg!{}", c!(effects::ITALIC), c!(RESET)),
    }
    match data.is_empty() {
        true => println!("    {}empty-data{}", c!(effects::ITALIC), c!(RESET)),
        false => println!("    data: {:?}", data),
    }
}
