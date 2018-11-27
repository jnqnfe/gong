// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Overview
//!
//! A common requirement of a program is the need to parse command line arguments supplied to it.
//! This library was designed to *assist* Rust based programs in this area.
//!
//! # Motivation
//!
//! While other solutions already existed for Rust programs, they arguably tried to do too much;
//! their designs were geared towards completely taking over *every* aspect of argument handling.
//! While removing as much of the burden from the application programmer as possible is a desirable
//! goal, these existing solutions in attempting to do this imposed significant restrictions on
//! compatible program designs.
//!
//! Some/all of these solutions also forced a “builder” type pattern of describing options through
//! successive method calls, lacking a more efficient option of directly defining the data
//! structure. This includes preventing efficient declaration of a `static`/`const` set.
//!
//! It appeared that the Rust crate ecosystem was missing a more fundamental and broadly applicable
//! solution, and one which favoured efficiency.
//!
//! # Design
//!
//! In C programming, the common `getopt` and `getopt_long` functions have, for a long time, served
//! to assist in this area. This library was inspired by them and designed as a similar fundamental
//! and broadly applicable *assistant*.
//!
//! The basic premise of usage is simple:
//!
//!  1. Describe the *options* (and optionally *command arguments*) to be “available” in your
//!     program.
//!  2. Provide the input argument(s) to be parsed to either the `parse` method or the `parse_iter`
//!     method, for them to be parsed against the *option* (and *command arg*) descriptions. The
//!     `parse` method does this all in one go, returning the full *analysis*, while the
//!     `parse_iter` method returns an iterator, with which you can retrieve one recognised item at
//!     a time.
//!  3. You can then use this analysis to respond as applicable.
//!
//! What it does not attempt to do includes: Automating help/usage/version request response (though
//! it may in future provide assistance for text generation); Data/state conversion/storage, value
//! range limiting; required/single-use option enforcement; option relationships (i.e. use of one
//! option means that use of another is invalid); and automatic response to conditions such as
//! unrecognised options. It avoids these sorts of things to avoid bloat; unnecessary complexity;
//! taking over control; inefficiency; etc. See the [FAQ](../faq/index.html) for more info.
//!
//! Differences to the old `getopt`/`getopt_long` C solution include:
//!
//!  * All parsing can be done in one go, rather than with recursive function calls, though this is
//!    optional.
//!  * Use of static global variables for “position” tracking is avoided.
//!  * *Non-options* are **not** shuffled to the end of the list, unlike the default behaviour of
//!    `getopt`.
//!  * The “convenience” functionality of `-W foo` being treated as `--foo` is not supported
//!    (unnecessary complexity).
//!
//! # Features
//!
//! <table>
//!     <thead>
//!         <tr><th>Feature</th><th>Supported/provided?</th></tr>
//!     </thead>
//!     <tbody>
//!         <tr><td>Iterative parsing style</td><td>Yes</td></tr>
//!         <tr><td>All-in-one (collect and data-mine) parsing style</td><td>Yes</td></tr>
//!         <tr><td>“Posixly correct” parsing</td><td>Yes, available as a parser setting</td></tr>
//!         <tr><td>Traditional style options (‘long’ and ‘short’)</td><td>Yes</td></tr>
//!         <tr><td>Alternate style options (‘long’ only, with single dash)</td><td>Yes</td></tr>
//!         <tr><td>Data-value taking options</td><td>Yes, both ‘in-same-arg’ and ‘in-next-arg’</td></tr>
//!         <tr><td>‘Positional’ (‘non-option’) arguments</td><td>Yes</td></tr>
//!         <tr><td>‘Early terminator’</td><td>Yes</td></tr>
//!         <tr><td>‘Command’ options</td><td>Yes, including nested</td></tr>
//!         <tr><td>Abbreviated long option name matching</td><td>Yes (optional)</td></tr>
//!         <tr><td>Mismatch suggestions</td><td>Yes*, for both (long) options and commands</td></tr>
//!         <tr><td>Dynamic ‘builder’ style option set construction</td><td>Yes</td></tr>
//!         <tr><td>Efficient ‘static’ option sets</td><td>Yes</td></tr>
//!         <tr><td>Help/usage output generation</td><td>Not currently</td></tr>
//!         <tr><td>Multi-use options</td><td>Yes, naturally, see more below!</td></tr>
//!         <tr><td>Single-use enforcement</td><td>Not done for you, see more below!</td></tr>
//!         <tr><td>Option relationships (e.g. conflicts, required, and such)</td><td>Not handed for you, not worth the complexity</td></tr>
//!         <tr><td>Data value range/set checking</td><td>Not done for you</td></tr>
//!         <tr><td>Data value type conversion</td><td>Not done for you</td></tr>
//!         <tr><td>`&str`/`String` based argument parsing</td><td>Yes, naturally</td></tr>
//!         <tr><td>`&OsStr`/`OsString` based argument parsing</td><td>Yes, with no platform restrictions, though only Unix (Linux &amp; Mac OS X) and Windows confirmed</td></tr>
//!         <tr><td>Tab completion</td><td>No, not currently</td></tr>
//!     </tbody>
//! </table>
//!
//! *Optional feature, controlled via the `Cargo` feature `suggestions`
//!
//! # Crate name origins
//!
//! The `gong` crate name was chosen in homage to the venerable C `getopt`, and can be considered a
//! “next-gen” solution for people moving to the Rust programming language.
//!
//! > **G**et-**O**pt **N**ext-**G**en → `gong`
//!
//! Hopefully this crate will become as common and universally adopted as `getopt` in C code.
