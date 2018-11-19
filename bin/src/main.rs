//! Test program for the `gong` library
//!
//! This test program takes user supplied command args, processes them with the library against a
//! set of example available options, and outputs a description of the results generated by the
//! processing library.
//!
//! See the `README.md` file for instructions.

// Copyright (c) 2017 Lyndon Brown
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

#![doc(html_logo_url = "https://github.com/jnqnfe/gong/raw/master/logo.png",
       html_favicon_url = "https://github.com/jnqnfe/gong/raw/master/favicon.ico")]
#![doc(html_no_source)]

extern crate gong;
extern crate libc;

mod console;

use console as con;
use gong::{Options, ItemClass, Item, ItemW, ItemE, DataLocation};
#[cfg(feature = "alt_mode")]
use gong::OptionsMode;

fn main() {
    // Color output
    let color = con::color_supported();
    let col_header = con::MAGENTA_B;

    #[cfg(not(windows))]
    macro_rules! c {
        ( $c:expr ) => { if color { $c } else { "" } };
    }
    #[cfg(windows)]
    macro_rules! c {
        ( $c:expr ) => { "" };
    }

    // Set up valid option descriptions
    let mut opts = Options::new(6, 5);
    opts.add_long("help")
        .add_short('h')
        .add_long("foo")
        .add_long("version")
        .add_long("foobar")
        .add_long("ábc")
        .add_long_data("hah")
        .add_short('❤')
        .add_short('x')
        .add_short_data('o')
        .add_short('\u{030A}'); // A lone combinator ("ring above")

    #[cfg(feature = "alt_mode")]
    opts.set_mode(OptionsMode::Alternate);

    #[cfg(feature = "no_abbreviations")]
    opts.set_allow_abbreviations(false);

    debug_assert!(opts.is_valid());

    println!("\n[ {}Mode{} ]\n", c!(col_header), c!(con::RESET));

    #[cfg(not(feature = "alt_mode"))]
    println!("{}STANDARD{}\n", c!(con::BLUE_2), c!(con::RESET));
    #[cfg(feature = "alt_mode")]
    println!("{}ALTERNATE{}\n", c!(con::BLUE_2), c!(con::RESET));

    #[cfg(not(feature = "keep_prog_name"))]
    println!("Skipping auto prog-name entry!\n(Compile with the `keep_prog_name` feature to not skip)\n");
    #[cfg(feature = "keep_prog_name")]
    println!("NOT skipping auto prog-name entry!\n");

    #[cfg(feature = "no_abbreviations")]
    println!("Abbreviated matching disabled!\n");

    println!("Standard = {0}Short options with single dash prefix and long options with double dash \
              prefix; compile with `alt_mode` feature for `alternate` mode.{1}\nAlternate = {0}Long \
              options with single dash prefix only.{1}\n", c!(con::ITALIC), c!(con::RESET));

    println!("[ {}Available options for test{} ]\n", c!(col_header), c!(con::RESET));

    for item in &opts.long {
        match item.expects_data {
            true => println!("LONG {} [expects data!]", item.name),
            false => println!("LONG {}", item.name),
        }
    }
    for item in &opts.short {
        match item.expects_data {
            true => println!("SHORT {} ({}) [expects data!]", item.ch, item.ch.escape_unicode()),
            false => println!("SHORT {} ({})", item.ch, item.ch.escape_unicode()),
        }
    }

    #[cfg(feature = "alt_mode")]
    println!("\nNote: Short options will be ignored in `alternative` mode. They are still printed \
              so you can test and see this is so!");

    #[cfg(feature = "keep_prog_name")]
    let args: Vec<String> = std::env::args().collect();
    #[cfg(not(feature = "keep_prog_name"))]
    let args: Vec<String> = std::env::args().skip(1).collect();

    println!("\n[ {}Your input arguments{} ]\n", c!(col_header), c!(con::RESET));

    for (i, arg) in args.iter().enumerate() {
        println!("[{}]: {}", i, arg);
    }
    if args.len() == 0 {
        println!("None!");
    }

    let results = gong::process(&args[..], &opts);

    println!("\n[ {}Analysis{} ]\n", c!(col_header), c!(con::RESET));

    let col_o = con::GREEN; //okay
    let col_e = con::RED; //error
    let col_w = con::YELLOW; //warning
    match results.error {
        true => { println!("Errors: {}true{}", c!(col_e), c!(con::RESET)); },
        false => { println!("Errors: {}false{}", c!(con::GREEN), c!(con::RESET)); },
    }
    match results.warn {
        true => { println!("Warnings: {}true{}", c!(col_w), c!(con::RESET)); },
        false => { println!("Warnings: {}false{}", c!(con::GREEN), c!(con::RESET)); },
    }
    println!("Items: {}\n", results.items.len());
    for result in &results.items {
        match *result {
            ItemClass::Ok(Item::NonOption(i, s)) =>
                println!("[arg {}] {}NonOption{}: {}", i, c!(col_o), c!(con::RESET), s),
            ItemClass::Ok(Item::EarlyTerminator(i)) =>
                println!("[arg {}] {}EarlyTerminator{}", i, c!(col_o), c!(con::RESET)),
            ItemClass::Ok(Item::Long(i, n)) =>
                println!("[arg {}] {}Long{}: {}", i, c!(col_o), c!(con::RESET), n),
            ItemClass::Ok(Item::LongWithData { i, n, d, ref l }) => {
                println!("[arg {}] {}LongWithData{}: {}", i, c!(col_o), c!(con::RESET), n);
                match *l {
                    DataLocation::SameArg =>
                        println!("    {}data found in SAME arg!{}", c!(con::ITALIC), c!(con::RESET)),
                    DataLocation::NextArg =>
                        println!("    {}data found in NEXT arg!{}", c!(con::ITALIC), c!(con::RESET)),
                }
                match d.is_empty() {
                    true => println!("    {}empty-data{}", c!(con::ITALIC), c!(con::RESET)),
                    false => println!("    data: {}", d),
                }
            },
            ItemClass::Err(ItemE::LongMissingData(i, n)) =>
                println!("[arg {}] {}LongMissingData{}: {}", i, c!(col_e), c!(con::RESET), n),
            ItemClass::Warn(ItemW::LongWithUnexpectedData { i, n, d }) =>
                println!("[arg {}] {}LongWithUnexpectedData{}: {}\n    data: {}", i, c!(col_w),
                    c!(con::RESET), n, d),
            ItemClass::Err(ItemE::AmbiguousLong(i, n)) =>
                println!("[arg {}] {}AmbiguousLong{}: {}", i, c!(col_e), c!(con::RESET), n),
            ItemClass::Warn(ItemW::LongWithNoName(i)) =>
                println!("[arg {}] {}LongWithNoName{}", i, c!(col_w), c!(con::RESET)),
            ItemClass::Warn(ItemW::UnknownLong(i, n)) =>
                println!("[arg {}] {}UnknownLong{}: {}", i, c!(col_w), c!(con::RESET), n),
            ItemClass::Ok(Item::Short(i, c)) =>
                println!("[arg {}] {}Short{}: {} {}({}){}", i, c!(col_o), c!(con::RESET), c,
                    c!(con::BLUE_2), c.escape_unicode(), c!(con::RESET)),
            ItemClass::Ok(Item::ShortWithData { i, c, d, ref l }) => {
                println!("[arg {}] {}ShortWithData{}: {} {}({}){}", i, c!(col_o), c!(con::RESET),
                    c, c!(con::BLUE_2), c.escape_unicode(), c!(con::RESET));
                match *l {
                    DataLocation::SameArg =>
                        println!("    {}data found in SAME arg!{}", c!(con::ITALIC), c!(con::RESET)),
                    DataLocation::NextArg =>
                        println!("    {}data found in NEXT arg!{}", c!(con::ITALIC), c!(con::RESET)),
                }
                match d.is_empty() {
                    true => println!("    {}empty-data{}", c!(con::ITALIC), c!(con::RESET)),
                    false => println!("    data: {}", d),
                }
            },
            ItemClass::Err(ItemE::ShortMissingData(i, c)) =>
                println!("[arg {}] {}ShortMissingData{}: {} {}({}){}", i, c!(col_e), c!(con::RESET),
                    c, c!(con::BLUE_2), c.escape_unicode(), c!(con::RESET)),
            ItemClass::Warn(ItemW::UnknownShort(i, c)) =>
                println!("[arg {}] {}UnknownShort{}: {} {}({}){}", i, c!(col_w), c!(con::RESET), c,
                    c!(con::BLUE_2), c.escape_unicode(), c!(con::RESET)),
        }
    }
    if results.items.len() != 0 {
        println!();
    }
}
