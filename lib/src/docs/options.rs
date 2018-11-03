// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Option support
//!
//! This crate has been designed around standard option conventions, and can process an argument
//! list in two different common styles:
//!
//! - Standard: supporting traditional *long* and *short* options
//! - Alternate: supporting *long options* only, with a single-dash prefix
//!
//! Basic feature support is on par with the C `getopt_long` function. (See the [overview] section
//! for mention of the small differences).
//!
//! # Standard style (default)
//!
//! This mode supports traditional *long* and *short* options. The fundamental argument processing
//! logic follows this model:
//!
//!  - An argument either **not** starting with a dash (`-`) or consisting only of a single dash is
//!    a *non-option* (a generic argument).
//!  - An argument of exactly two dashes (`--`) only is called an *early terminator* and is
//!    described below.
//!  - An argument starting with two dashes (`--`) followed by additional characters is a *long
//!    option*. The portion after the double-dash prefix is the *long option name* (or possibly the
//!    name combined with an "in-argument" *data value*, as discussed below).
//!  - An argument starting with a single dash (`-`) followed by additional (non-dash) characters is
//!    a *short option set*, where each character (`char`) after the single-dash prefix represents a
//!    *short option* (except with respect to "in-argument" *data values*, as discussed below).
//!    (More than one *short option* can be specified in a single argument). Note that a dash (`-`)
//!    itself is not permitted to be a valid program *short option* (it would be misinterpreted in
//!    some cases). Note also that interpretation of what consists of a "character" may surprise you
//!    as actually being a complicated matter, as per the dedicated "Utf-8" discussion later.
//!
//! Processing of each argument may alter how one or more subsequent arguments are interpreted,
//! deviating from the above. Specifically this applies to the *early terminator* and where an
//! option is recognised as an "available" program option that takes a *data value*, as discussed
//! below.
//!
//! ## Data values
//!
//! *Long* and *short* options can be configured as either "flag" style (used to signal a condition)
//! or as "data taking" style, accepting an accompanying single *data value*. *Data values* for both
//! *long* and *short* options can either be supplied within the same argument as the option itself
//! ("in-argument"), or as the next argument in which case that will thus be consumed as the
//! option's *data value* and otherwise ignored.
//!
//!  - For *long options*, "next-arg" style looks like `--foo bar`, while "in-argument" style uses
//!    an equals (`=`) character between the option name and value components, e.g. `--foo=bar`.
//!    Note that "available" program *long options* are forbidden from containing an equals (`=`)
//!    character in their name as this would otherwise introduce significant problems.
//!
//!    When processing a *long option* argument, if the argument contains one or more equals (`=`)
//!    characters then it is considered to have an "in-argument" *data value* (since names are not
//!    permitted to contain them), and is split into two components, thus the left hand portion
//!    (minus the double-dash prefix) is taken as the name, and the right as the "in-argument" *data
//!    value* (e.g. `--foo=bar` → name: "foo", value: "bar"). This naturally occurs **before**
//!    checking for a matching "available" program option.
//!
//!     - If the name component does not match any "available" *long option*, then it is reported as
//!       unknown, with any "in-argument" *data value* component ignored.
//!     - If a match is found which **does not** take a *data value*, then if an "in-argument" *data
//!       value* component was supplied, its presence is reported as unexpected, otherwise all is
//!       good.
//!     - If a match is found that **does** take a *data value*, then if an "in-argument" *data
//!       value* component was present, this is consumed as such, otherwise the next argument is
//!       consumed. If an "in-argument" *data value* component was present, but the actual value is
//!       missing (e.g. as in `--foo=`), this does not matter, the *data value* is accepted as being
//!       an empty string (it does not consume the next argument). If no "in-argument" component was
//!       supplied and this is the last argument, then the *data value* is reported as missing.
//!
//!  - For *short options*, "next-arg" style looks like `-o arg`, while "in-argument" style looks
//!    like `-oarg`.
//!
//!    When a *short option set* is encountered (remember, more than one *short option* can be
//!    grouped in the same argument), the characters are gone through in sequence, looking for
//!    matching program *short options*.
//!
//!     - If no match is found then it is reported as unknown.
//!     - If a match is found that **does not** take a *data value*, then great.
//!     - If a match is found that **does** take a *data value*, then one needs to be found. If this
//!       character is **not** the last in the set, then the remaining portion of the argument is
//!       consumed as this option's *data value* (e.g. if 'o' is such an option then in `-oarg`,
//!       `arg` is it's *data value*). If it is the last character in the set, then the next
//!       argument is consumed (e.g. `-o arg` → `arg`). If it is the last in the set and there is no
//!       next argument, then the *data value* is reported as missing.
//!
//!    Naturally when multiple *short options* are grouped together in the same argument, only the
//!    last in that group can be one that takes a *data value*, and users must be careful when
//!    constructing such groups.
//!
//! ## Early terminator
//!
//! An *early terminator* is used by a user of a program to request early termination of argument
//! interpretation, meaning that all subsequent arguments in the argument list should be considered
//! to be *non-options*. This is useful for instance if a program passes along some or all
//! *non-options* to something else, and the user wants arguments that are formatted as options to
//! be passed along (i.e. passing along of options); An early terminator blocks the program from
//! interpreting anything following it as options targetted towards itself.
//!
//! For example, in the following command the `--release` argument is consumed by `cargo` (as is the
//! `run` *non-option* which it treats as a "command" mode indicator), while `--foo` is treated as a
//! *non-option*. Cargo in "run" mode passes on all *non-options* (except `run`) to the program it
//! runs (equivalent to running `<my-prog> --foo` directly).
//!
//! ```text
//! cargo run --release -- --foo
//! ```
//!
//! # Alternate style
//!
//! This mode is very similar to *standard* style, with the main difference simply being that *short
//! options* are **not** supported, and *long options* use a single dash (`-`) as a prefix rather
//! than two, i.e. `-help` rather than `--help`. Some people simply prefer this style, and support
//! for it was both trivial to add and involves very little overhead.
//!
//! **Note**: *Short options* can still be added to the option set in this mode, and it will still
//! pass as valid; they will simply be ignored when performing matching.
//!
//! # Abbreviated long option name matching
//!
//! Abbreviated *long option* name matching is supported, i.e. the feature than users can use an
//! abbreviated form of a *long option's* name and get a match, so long as the abbreviation uniquely
//! matches a single *long option*.
//!
//! As an example, with the input arguments from the following command:
//!
//! ```text
//! <progname> --f --fo --foo --foob --fooba --foobar
//! ```
//!
//! If `foo` and `foobar` are available *long options* then:
//!
//!  - `--foo` and `--foobar` are exact matches for the available `foo` and `foobar` options
//!    respectively.
//!  - `--f` and `--fo` are invalid as being ambiguous (and noted as such in the analysis).
//!  - `--foob` and `--fooba` both uniquely match `foobar` and so are valid.
//!
//! This is enabled by default, but can be opted out of when processing if not desired.
//!
//! # Utf-8 notes
//!
//! Native Utf-8 support in Rust makes handling Utf-8 strings largely trivial. It is important to
//! understand that in Rust a `char` is four bytes (it was only one byte in older languages like C);
//! but a sequence of `char`s are typically stored more efficiently than this in a string. This
//! widened `char` type broadens the range of possible characters that can be used as *short
//! options*, without us worrying about any multi-byte complexity. This allows for instance `ð`
//! (the "sparkle heart" `char`) to be a *short option*, if you wanted, along with a huge set of
//! other characters of various types to choose from. (The "sparkle heart" `char` take three bytes
//! in a Utf-8 string, and would not have been easy to support in C with the legacy `getopt`
//! solution).
//!
//! With respect to *long options*, `--foo`, `--föö` and `--föö` are all different options (the last
//! two may look the same, but read on), and are all perfectly valid options to make available. The
//! first consists of simple latin characters only. The second and third use "umlauts" (diaeresis)
//! above the `o`'s, however the first of these uses a `char` with the umlaut built in (`U+F6`) and
//! the second uses the standard `o` (`U+6F`) followed by the special umlaut combining `char`
//! (`U+0308`), thus they appear the same but are actually different "under the hood". (It would not
//! be efficient or worthwhile to try to handle the latter two as being the same option).
//!
//! Only single `char`s are supported for *short options*. A `char` paired with one or more special
//! combinator/selector `char`s thus cannot be specified as an available *short option*. Such
//! special `char`s are treated by this library as perfectly valid available *short options* in
//! their own right. Thus, whilst `-ö` (using `U+F6`) results in a single matched/unmatched entry in
//! the analysis, `-ö` (using `U+6F` followed by the `U+0308` combinator) will result in two
//! entries, for what looks visibly to be one character. As another example, `❤` is the "black
//! heart" character, and `❤️` is it along with the `U+FE0F` "variant #16 - emoji" selector `char`;
//! with the selector, `--❤️` is a single matched/unmatched *long option*, while `-❤️` is a pair of
//! matched/unmatched *short options*, one for the "black heart" `char` and one for the selector
//! `char`.
//!
//! [overview]: ../overview/index.html
