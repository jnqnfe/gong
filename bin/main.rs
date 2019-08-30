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

#[macro_use]
extern crate gong;
extern crate term_ctrl;

#[cfg(feature = "osstr")]
use std::ffi::{OsStr, OsString};
use term_ctrl::predefined::*;
use gong::analysis::{ItemClass, Item, ItemW, ItemE, DataLocation};
use gong::parser::{Parser, OptionsMode};
use self::printers::*;

const COL_HEADER: &str = combinations::fg_bold::MAGENTA;
const COL_O: &str = colours::fg::GREEN;  //okay
const COL_E: &str = colours::fg::RED;    //error
const COL_W: &str = colours::fg::YELLOW; //warning
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
    let opts = gong_option_set!(
        @long [
            gong_longopt!("help"),
            gong_longopt!("foo"),
            gong_longopt!("version"),
            gong_longopt!("foobar"),
            gong_longopt!(@data "hah"),
            gong_longopt!("ábc"),
        ],
        @short [
            gong_shortopt!('h'),
            gong_shortopt!('❤'),
            gong_shortopt!('x'),
            gong_shortopt!(@data 'o'),
        ]
    );
    let mut parser = Parser::new(&opts, None);

    match cfg!(feature = "alt_mode") {
        true => { parser.settings.set_mode(OptionsMode::Alternate); },
        false => { parser.settings.set_mode(OptionsMode::Standard); },
    }
    match cfg!(feature = "no_abbreviations") {
        true => { parser.settings.set_allow_abbreviations(false); },
        false => { parser.settings.set_allow_abbreviations(true); },
    }
    debug_assert!(parser.is_valid());

    println!("\n[ {}Config{} ]\n", c!(COL_HEADER), c!(RESET));

    #[cfg(not(feature = "alt_mode"))]
    println!("Option style: {}Standard{}", c!(COL_MODE), c!(RESET));
    #[cfg(feature = "alt_mode")]
    println!("Option style: {}Alternate{}", c!(COL_MODE), c!(RESET));

    #[cfg(not(feature = "osstr"))]
    println!("String processing mode: {}`str` based{}", c!(COL_MODE), c!(RESET));
    #[cfg(feature = "osstr")]
    println!("String processing mode: {}`OsStr` based{}", c!(COL_MODE), c!(RESET));

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

    #[cfg(not(feature = "osstr"))]
    let args = std::env::args();
    #[cfg(feature = "osstr")]
    let args = std::env::args_os();

    #[cfg(not(feature = "keep_prog_name"))]
    let args = args.skip(1);

    #[cfg(not(feature = "osstr"))]
    let args: Vec<String> = args.collect();
    #[cfg(feature = "osstr")]
    let args: Vec<OsString> = args.collect();

    println!("\n[ {}Your input arguments{} ]\n", c!(COL_HEADER), c!(RESET));

    match args.len() {
        0 => println!("None!"),
        _ => for (i, arg) in args.iter().enumerate() {
            #[cfg(not(feature = "osstr"))]
            { println!("[{}]: {}", i, arg); }
            #[cfg(feature = "osstr")]
            { println!("[{}]: {:?}", i, arg); }
        },
    }

    #[cfg(feature = "osstr")]
    let results = parser.parse_os(&args[..]);
    #[cfg(not(feature = "osstr"))]
    let results = parser.parse(&args[..]);

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

    // `OsStr` wrapper
    #[cfg(not(feature = "osstr"))]
    let ow = |s| { s };
    #[cfg(feature = "osstr")]
    let ow = |s| { OsStr::new(s) };

    // `OsStr` wrapper
    #[cfg(not(feature = "osstr"))]
    let ow2 = |s| { s };
    #[cfg(feature = "osstr")]
    let ow2 = |s| { OsString::from(s) };

    println!("Items: {}\n", results.items.len());
    for result in &results.items {
        let printer = match *result {
            ItemClass::Ok(_) => print_arg_ok,
            ItemClass::Err(_) => print_arg_err,
            ItemClass::Warn(_) => print_arg_warn,
        };
        match *result {
            ItemClass::Ok(Item::Positional(i, s)) => printer(i, "Positional", s),
            ItemClass::Ok(Item::EarlyTerminator(i)) => printer(i, "EarlyTerminator", ow("")),
            ItemClass::Ok(Item::Long(i, n)) => printer(i, "Long", ow(n)),
            ItemClass::Ok(Item::LongWithData { i, n, d, ref l }) => {
                printer(i, "LongWithData", ow(n));
                print_data(*l, d);
            },
            ItemClass::Err(ItemE::LongMissingData(i, n)) => printer(i, "LongMissingData", ow(n)),
            ItemClass::Warn(ItemW::LongWithUnexpectedData { i, n, d }) => {
                printer(i, "LongWithUnexpectedData", ow(n));
                #[cfg(not(feature = "osstr"))]
                { println!("    data: {}", d) }
                #[cfg(feature = "osstr")]
                { println!("    data: {:?}", d) }
            },
            ItemClass::Err(ItemE::AmbiguousLong(i, n)) => printer(i, "AmbiguousLong", n),
            ItemClass::Warn(ItemW::LongWithNoName(i)) => printer(i, "LongWithNoName", ow("")),
            ItemClass::Warn(ItemW::UnknownLong(i, n)) => printer(i, "UnknownLong", n),
            ItemClass::Ok(Item::Short(i, c)) => {
                let desc = ow2(desc_char(c));
                printer(i, "Short", &desc);
            },
            ItemClass::Ok(Item::ShortWithData { i, c, d, ref l }) => {
                let desc = ow2(desc_char(c));
                printer(i, "ShortWithData", &desc);
                print_data(*l, d);
            },
            ItemClass::Err(ItemE::ShortMissingData(i, c)) =>{
                let desc = ow2(desc_char(c));
                printer(i, "ShortMissingData", &desc);
            },
            ItemClass::Warn(ItemW::UnknownShort(i, c)) =>{
                let desc = ow2(desc_char(c));
                printer(i, "UnknownShort", &desc);
            },
            ItemClass::Ok(Item::Command(i, n)) => printer(i, "Command", ow(n)),
        }
    }
    if results.items.len() != 0 {
        println!();
    }
}

pub fn print_arg(col: &str, index: usize, ty: &str, desc: &str) {
    println!("[arg {}] {}{}{}: {}", index, col, ty, c!(RESET), desc)
}

pub fn desc_char(ch: char) -> String {
    format!("{} {}({}){}", ch, c!(COL_CHAR), ch.escape_unicode(), c!(RESET))
}

#[cfg(not(feature = "osstr"))]
mod printers {
    use super::*;

    pub fn print_arg_ok(index: usize, ty: &str, desc: &str) {
        print_arg(c!(COL_O), index, ty, desc);
    }

    pub fn print_arg_err(index: usize, ty: &str, desc: &str) {
        print_arg(c!(COL_E), index, ty, desc);
    }

    pub fn print_arg_warn(index: usize, ty: &str, desc: &str) {
        print_arg(c!(COL_W), index, ty, desc);
    }

    pub fn print_data(loc: DataLocation, data: &str) {
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
}

#[cfg(feature = "osstr")]
mod printers {
    use super::*;

    pub fn print_arg_ok(index: usize, ty: &str, desc: &OsStr) {
        print_arg(c!(COL_O), index, ty, &desc.to_string_lossy());
    }

    pub fn print_arg_err(index: usize, ty: &str, desc: &OsStr) {
        print_arg(c!(COL_E), index, ty, &desc.to_string_lossy());
    }

    pub fn print_arg_warn(index: usize, ty: &str, desc: &OsStr) {
        print_arg(c!(COL_W), index, ty, &desc.to_string_lossy());
    }

    pub fn print_data(loc: DataLocation, data: &OsStr) {
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
}
