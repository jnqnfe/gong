// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! This file holds no tests itself, it is used as a submodule for other test files, providing a
//! common base set of options to work with.

use gong::*;

//TODO: now that processing accepts both `String` and `&str` type "available option" sets, cover both somehow
/// Used for cleaner creation of set of test arguments
///
/// Note, arguments will normally be obtained from the environment, and Rust provides these to use
/// as String objects, not &str, hence why we create a vector of Strings!
#[macro_export]
macro_rules! arg_list {
    ( $($e:expr),+ ) => {
        vec![$(String::from($e)),+]
    };
    ( $($e:expr,)+ ) => {
        vec![$(String::from($e)),+]
    };
}

// NB: These are not publically accessible from the crate, so we duplicate them. We do have a test
// to ensure they are correct however!
pub const ABBR_SUP_DEFAULT: bool = true;
pub const MODE_DEFAULT: OptionsMode = OptionsMode::Standard;

#[derive(Debug)] pub struct Actual<'a>(pub Results<'a>);
#[derive(Debug)] pub struct Expected<'a>(pub Results<'a>);

/// Provides a base set of options for common usage in tests
pub fn get_base<'a>() -> Options<'a> {
    // Note, the macro tests were written with the expectation that this function is constructing
    // this base set using the macros. Thus if this were changed (though there is no conceivable
    // reason to) to not use the macros, that needs addressing.
    gong_option_set!(
        vec![
            gong_longopt!("help"),
            gong_longopt!("foo"),
            gong_longopt!("version"),
            gong_longopt!("foobar"),
            gong_longopt!("hah", true),
            gong_longopt!("ábc"),       // Using a combinator char (accent)
            gong_longopt!("ƒƒ", true),  // For multi-byte with-data long option component split checking
        ],
        vec![
            gong_shortopt!('h'),
            gong_shortopt!('❤'),
            gong_shortopt!('x'),
            gong_shortopt!('o', true),
            gong_shortopt!('\u{030a}'), // A lone combinator ("ring above")
            gong_shortopt!('Ɛ', true),  // For multi-byte with-data calculation checking
        ],
        OptionsMode::Standard,
        true
    )
}

/// Construct an `Expected`
macro_rules! expected {
    ( error: $e:expr, warn: $w:expr, $items:expr ) => {
        Expected(Results { error: $e, warn: $w, items: $items, })
    };
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

/// Common central function for comparing actual results with expected.
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

        assert!(false, "actual results do not match expected!");
    }
}

/// Prints a pretty description of a `Results` struct, used in debugging for easier comparison than
/// with the raw output dumped by the test env.
///
/// Note, the `:#?` formatter is available as the "pretty" version of `:?`, but this is too sparse
/// an output, so we custom build a more compact version here.
fn pretty_print_results(results: &Results) {
    let mut items = String::new();
    for item in &results.items {
        items.push_str(&format!("\n        {:?},", item));
    }
    eprintln!("\
Results {{
    items: [{}
    ],
    error: {},
    warn: {},
}}",
    items, results.error, results.warn);
}
