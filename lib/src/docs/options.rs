// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Functionality
//!
//! Basic feature support is on par with legacy 'getopt_long'. See the [overview] section for an
//! overview of design and for mention of the small differences.
//!
//! # Option support
//!
//! Two option processing modes are available, supporting two different popular styles of options.
//!
//! ## Mode 1 - Standard (default)
//!
//! This mode supports traditional *long* and *short* options.
//!
//! An argument starting with a dash (`-`) and followed by additional characters, is treated as an
//! *option* argument, anything else is a *non-option*. An argument of `--` followed by additional
//! characters is a *long* option, which, after the `--` prefix, consists of an option *name*
//! (followed optionally by a *data sub-argument*, as discussed below). An argument of a single dash
//! (`-`) followed by additional (non-dash) characters is a *short option set*, where each character
//! after the `-` prefix is a *short* option *character* (except with respect to *data
//! sub-arguments*, as mentioned below). An argument of exactly `--` only is special - an *early
//! terminator* - symbolising early termination of argument interpretation, meaning that all
//! subsequent arguments should be assumed to be non-options (useful in some situations/designs for
//! separating your option arguments from those to be passed on to something else).
//!
//! Options may have a single mandatory *data sub-argument*. For long options, data is provided
//! either in the next argument (e.g. `--foo bar`) or in the same argument, separated from the name
//! by an `=` (e.g. `--foo=bar`). For short options, data is provided either in the next argument
//! (e.g. `-o arg`), or if the option character is not the last character in the argument, the
//! remaining characters are taken to be its data arg (e.g. `-oarg`). An argument can contain
//! multiple short options grouped together as a *set* (e.g. `-abc`), but of course users need to be
//! careful doing so with those requiring data - for correct interpretation only one short option
//! with data can be grouped, and it must be the last one in the set. (If in `-abc` all three
//! characters are valid options, and `b` takes data, `c` will be consumed as `b`'s data instead of
//! being interpreted as an option).
//!
//! If a long option is encountered where the argument contains one or more `=` characters, then the
//! left hand portion of the first `=` character is taken to be the long option name, and the right
//! hand portion as a data sub-argument, thus valid available option names cannot contain `=`
//! characters. If the name does not match any available long option, a failed match is reported and
//! the data sub-arg is completely ignored. If there is a match and it requires a data sub-arg, but
//! the `=` was the last character in the argument, (e.g. `--foo=`), then the data sub-arg is taken
//! to be an empty string. If there is a match with an option that does not require a data sub-arg,
//! but one was provided and it is not an empty string, this will be noted as unexpected in the
//! results of analysis.
//!
//! Abbreviated long option name matching is supported, i.e. the feature than users can use an
//! abbreviated form of a long option's name and get a match, so long as the abbreviation uniquely
//! matches a single long option. As an example, if `foo` and `foobar` are available long options,
//! then for the possible input arguments of { `--f`, `--fo`, `--foo`, `--foob`, `--fooba`, and
//! `--foobar` }, `--foo` and `--foobar` are exact matches for `foo` and `foobar` respectively;
//! `--f` and `--fo` are invalid as being ambiguous (and noted as such in the results); and `--foob`
//! and `--fooba` both uniquely match `foobar` and so are valid. This feature is enabled by default,
//! but can be disabled if desired.
//!
//! ## Mode 2 - Alternate
//!
//! This mode is very similar to mode 1, with the main difference simply being that only long
//! options are supported, and long options use a single dash (`-`) as a prefix rather than two,
//! i.e. `-help` rather than `--help`. Some people simply prefer this style, and support for it was
//! very easy to add.
//!
//! **Note**: Short options can still be added to the option set in this mode, and it will still
//! pass as valid; they will simply be ignored when performing matching.
//!
//! # Mismatch suggestions
//!
//! This library does not (currently) itself provide any suggestion mechanism for failed option
//! matches - i.e. the ability to take an unmatched long option and pick the most likely of the
//! available options that the user may have actually meant to use, to suggest to them when
//! reporting the error. There is nothing however stopping users of this library from running
//! unmatched options through a third-party library to obtain the suggestion to display.
//!
//! # Utf-8 support
//!
//! Native Utf-8 support in Rust makes handling Utf-8 strings largely trivial. It is important to
//! understand that in Rust a `char` is four bytes (it was only one byte in older languages like C);
//! but a sequence of `char`s are typically stored more efficiently than this in a string. This
//! widened `char` type broadens the range of possible characters that can be used as short options,
//! without us worrying about any multi-byte complexity. This allows for instance `💖` (the
//! "sparkle heart" `char`) to be a short option, if you wanted, along with a huge set of other
//! characters of various types to choose from. (The "sparkle heart" `char` take three bytes in a
//! Utf-8 string, and would not have been easy to support in C with the legacy 'getopt' solution).
//!
//! With respect to long options, `--foo`, `--föö` and `--föö` are all different options (the last
//! two may look the same, but read on), and are all perfectly valid options to make available. The
//! first consists of simple latin characters only. The second and third use "umlauts" (diaeresis)
//! above the `o`'s, however the first of these uses a `char` with the umlaut built in (`U+F6`) and
//! the second uses the standard `o` (`U+6F`) followed by the special umlaut combining `char`
//! (`U+0308`), thus they appear the same but are actually different "under the hood". (It would not
//! be efficient or worthwhile to try to handle the latter two as being the same option).
//!
//! Only single `char`s are supported for short options. A `char` paired with one or more special
//! combinator/selector `char`s thus cannot be specified as an available short option. Such special
//! `char`s are treated by this library as perfectly valid available short options in their own
//! right. Thus, whilst `-ö` (using `U+F6`) results in a single matched/unmatched entry in the
//! results returned from the [`process`] function, `-ö` (using `U+6F` followed by the `U+0308`
//! combinator) will result in two entries, for what looks visibly to be one character. As another
//! example, `❤` is the "black heart" character, and `❤️` is it along with the `U+FE0F` "variant #16
//! \- emoji" selector `char`; with the selector, `--❤️` is a single matched/unmatched long
//! option, while `-❤️` is a pair of matched/unmatched short options, one for the "black heart"
//! `char` and one for the selector `char`.
//!
//! [overview]: ../overview/index.html
//! [`process`]: ../../fn.process.html
