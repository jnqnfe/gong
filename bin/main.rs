// Copyright 2017 Lyndon Brown
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Test program for the `gong` library
//!
//! This test program takes user supplied command args, processes them with the library against a
//! set of example available options, and outputs a description of the results generated by the
//! processing library.
//!
//! The analysis is output in color, where supported and applicable.
//!
//! See the `README.md` file for instructions.

#![doc(html_logo_url = "https://github.com/jnqnfe/gong/raw/master/logo.png",
       html_favicon_url = "https://github.com/jnqnfe/gong/raw/master/favicon.ico")]
#![doc(html_no_source)]

#[macro_use]
extern crate gong;
extern crate term_ctrl;

use term_ctrl::predefined::*;
use gong::analysis::{ItemClass, Item, ItemW, ItemE, DataLocation};
use gong::options::OptionsMode;

const COL_HEADER: &str = color1_bold::MAGENTA;
const COL_O: &str = color1::GREEN;  //okay
const COL_E: &str = color1::RED;    //error
const COL_W: &str = color1::YELLOW; //warning
const COL_CHAR: &str = color2::BLUE;
const COL_MODE: &str = color2::BLUE;
const COL_DATA: &str = color2::YELLOW;

/// Config: Used for holding state of stdout formatting support
pub mod config {
    use std::sync::Once;
    use term_ctrl::use_fmt_stdout;

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

// Color? Filter the provided 'formatted-stdout-ctrl-seq' string
macro_rules! c {
    ( $code:expr ) => { if config::formatted_stdout() { $code } else { "" } };
}

fn main() {
    config::init();

    // Set up valid option descriptions
    #[allow(unused_mut)] //Needed when no special features set
    let mut opts = gong_option_set_fixed!(
        [
            gong_longopt!("help"),
            gong_longopt!("foo"),
            gong_longopt!("version"),
            gong_longopt!("foobar"),
            gong_longopt!("hah", true),
            gong_longopt!("ábc"),        // Using a combinator char (accent)
        ],
        [
            gong_shortopt!('h'),
            gong_shortopt!('❤'),
            gong_shortopt!('x'),
            gong_shortopt!('o', true),
            gong_shortopt!('\u{030A}'),   // A lone combinator ("ring above")
        ],
        OptionsMode::Standard,
        true
    );

    #[cfg(feature = "alt_mode")]
    opts.set_mode(OptionsMode::Alternate);
    #[cfg(feature = "no_abbreviations")]
    opts.set_allow_abbreviations(false);

    debug_assert!(opts.is_valid());

    println!("\n[ {}Mode{} ]\n", c!(COL_HEADER), c!(RESET));

    #[cfg(not(feature = "alt_mode"))]
    println!("{}STANDARD{}\n", c!(COL_MODE), c!(RESET));
    #[cfg(feature = "alt_mode")]
    println!("{}ALTERNATE{}\n", c!(COL_MODE), c!(RESET));

    #[cfg(not(feature = "keep_prog_name"))]
    println!("Skipping auto prog-name entry!\n(Compile with the `keep_prog_name` feature to not skip)\n");
    #[cfg(feature = "keep_prog_name")]
    println!("NOT skipping auto prog-name entry!\n");

    #[cfg(feature = "no_abbreviations")]
    println!("Abbreviated matching disabled!\n");

    println!("Standard = {0}Short options with single dash prefix and long options with double dash \
              prefix; compile with `alt_mode` feature for `alternate` mode.{1}\nAlternate = {0}Long \
              options with single dash prefix only.{1}\n", c!(effects::ITALIC), c!(RESET));

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
    let args: Vec<String> = std::env::args().collect();
    #[cfg(not(feature = "keep_prog_name"))]
    let args: Vec<String> = std::env::args().skip(1).collect();

    println!("\n[ {}Your input arguments{} ]\n", c!(COL_HEADER), c!(RESET));

    match args.len() {
        0 => println!("None!"),
        _ => for (i, arg) in args.iter().enumerate() {
            println!("[{}]: {}", i, arg);
        },
    }

    let results = opts.process(&args[..]);

    println!("\n[ {}Analysis{} ]\n", c!(COL_HEADER), c!(RESET));

    match results.error {
        true => { println!("Errors: {}true{}", c!(COL_E), c!(RESET)); },
        false => {
            println!("Errors: {}false{}", c!(COL_O), c!(RESET));
        },
    }
    match results.warn {
        true => { println!("Warnings: {}true{}", c!(COL_W), c!(RESET)); },
        false => {
            println!("Warnings: {}false{}", c!(COL_O), c!(RESET));
        },
    }

    println!("Items: {}\n", results.items.len());
    for result in &results.items {
        let printer = match *result {
            ItemClass::Ok(_) => print_arg_ok,
            ItemClass::Err(_) => print_arg_err,
            ItemClass::Warn(_) => print_arg_warn,
        };
        match *result {
            ItemClass::Ok(Item::NonOption(i, s)) => printer(i, "NonOption", s),
            ItemClass::Ok(Item::EarlyTerminator(i)) => printer(i, "EarlyTerminator", ""),
            ItemClass::Ok(Item::Long(i, n)) => printer(i, "Long", n),
            ItemClass::Ok(Item::LongWithData { i, n, d, ref l }) => {
                printer(i, "LongWithData", n);
                print_data(*l, d);
            },
            ItemClass::Err(ItemE::LongMissingData(i, n)) => printer(i, "LongMissingData", n),
            ItemClass::Warn(ItemW::LongWithUnexpectedData { i, n, d }) => {
                printer(i, "LongWithUnexpectedData", n);
                println!("\n    data: {}", d)
            },
            ItemClass::Err(ItemE::AmbiguousLong(i, n)) => printer(i, "AmbiguousLong", n),
            ItemClass::Warn(ItemW::LongWithNoName(i)) => printer(i, "LongWithNoName", ""),
            ItemClass::Warn(ItemW::UnknownLong(i, n)) => printer(i, "UnknownLong", n),
            ItemClass::Ok(Item::Short(i, c)) => {
                let desc = desc_char(c);
                printer(i, "Short", &desc);
            },
            ItemClass::Ok(Item::ShortWithData { i, c, d, ref l }) => {
                let desc = desc_char(c);
                printer(i, "ShortWithData", &desc);
                print_data(*l, d);
            },
            ItemClass::Err(ItemE::ShortMissingData(i, c)) =>{
                let desc = desc_char(c);
                printer(i, "ShortMissingData", &desc);
            },
            ItemClass::Warn(ItemW::UnknownShort(i, c)) =>{
                let desc = desc_char(c);
                printer(i, "UnknownShort", &desc);
            },
        }
    }
    if results.items.len() != 0 {
        println!();
    }
}

fn print_arg(col: &str, index: usize, ty: &str, desc: &str) {
    println!("[arg {}] {}{}{}: {}", index, col, ty, c!(RESET), desc)
}

fn print_arg_ok(index: usize, ty: &str, desc: &str) {
    print_arg(c!(COL_O), index, ty, desc);
}

fn print_arg_err(index: usize, ty: &str, desc: &str) {
    print_arg(c!(COL_E), index, ty, desc);
}

fn print_arg_warn(index: usize, ty: &str, desc: &str) {
    print_arg(c!(COL_W), index, ty, desc);
}

fn desc_char(ch: char) -> String {
    format!("{} {}({}){}", ch, c!(COL_CHAR), ch.escape_unicode(), c!(RESET))
}

fn print_data(loc: DataLocation, data: &str) {
    match loc {
        DataLocation::SameArg =>
            println!("    {}data found in SAME arg!{}", c!(effects::ITALIC), c!(RESET)),
        DataLocation::NextArg =>
            println!("    {}data found in NEXT arg!{}", c!(effects::ITALIC), c!(RESET)),
    }
    match data.is_empty() {
        true => println!("    {}empty-data{}", c!(effects::ITALIC), c!(RESET)),
        false => println!("    data: {}", data),
    }
}
