// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Unicode & UTF-8 support
//!
//! Native Utf-8 support in Rust makes handling Utf-8 strings largely trivial. It is important to
//! understand that in Rust a `char` is four bytes (it was only one byte in older languages like C);
//! but a sequence of `char`s are typically stored more efficiently than this in a string. This
//! widened `char` type broadens the range of possible characters that can be used as *short
//! options*, without us worrying about any multi-byte complexity. This allows for instance `💖`
//! (the “sparkle heart” `char`) to be a *short option*, if you wanted, along with a huge set of
//! other characters of various types to choose from. (The “sparkle heart” `char` take four bytes in
//! a Utf-8 string, and would not have been easy to support in C with the legacy `getopt` solution).
//!
//! With respect to *long options*, `--foo`, `--föö` and `--föö` are all different options (the last
//! two may look the same, but read on), and are all perfectly valid options to make available. The
//! first consists of simple latin characters only. The second and third use “umlauts” (diaeresis)
//! above the `o`’s, however the first of these uses a `char` with the umlaut built in (`U+F6`) and
//! the second uses the standard `o` (`U+6F`) followed by the special umlaut combining `char`
//! (`U+0308`), thus they appear the same but are actually different “under the hood”. (It would not
//! be efficient or worthwhile to try to handle the latter two as being the same option).
//!
//! Only single `char`s are supported for *short options*. A `char` paired with one or more special
//! combinator/selector `char`s thus cannot be specified as an available *short option*. Such
//! special `char`s are treated by this library as perfectly valid available *short options* in
//! their own right. Thus, whilst `-ö` (using `U+F6`) results in a single matched/unmatched entry in
//! the analysis, `-ö` (using `U+6F` followed by the `U+0308` combinator) will result in two
//! entries, for what looks visibly to be one character. As another example, `❤` is the “black
//! heart” character, and `❤️` is it along with the `U+FE0F` “variant #16 - emoji” selector `char`;
//! with the selector, `--❤️` is a single matched/unmatched *long option*, while `-❤️` is a pair of
//! matched/unmatched *short options*, one for the “black heart” `char` and one for the selector
//! `char`.
