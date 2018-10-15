// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Overview
//!
//! A common requirement of a program is the need to process command line arguments supplied to it.
//! This library was designed to *assist* Rust based programs in this area.
//!
//! # Motivation
//!
//! While other solutions already existed for Rust programs, they arguably tried to do too much;
//! their designs were geared towards completely taking over *every* aspect of argument handling.
//! While removing as much of the burden from the application programmer as possible is a desirable
//! goal, these existing solutions in attempting to do this imposed significant restrictions on
//! compatible program designs. It appeared that the Rust crate ecosystem was missing a more
//! fundamental and broadly applicable solution.
//!
//! Some/all of these solutions also forced a "builder" type pattern of describing options through
//! successive method calls, lacking a more efficient option of directly defining the data
//! structure. This includes preventing efficient declaration of a `static`/`const` set.
//!
//! # Design
//!
//! In C programming, the common `getopt` and `getopt_long` functions have, for a long time, served
//! to assist in this area. This library was inspired by them and designed as a similar fundamental
//! and broadly applicable *assistant*.
//!
//! The basic premise of usage is simple:
//!
//!  1. Provide the processing function with a description of available options along with the input
//!     arguments to be processed, and it returns the results of its analysis.
//!  2. You can then work through this analysis to respond as applicable: Output error information
//!     if the user made a mistake; output help/usage information if requested; store state
//!     information from recognised flag type options; and store data (converting values as
//!     necessary) from non-options and recognised "with data arg" options.
//!  3. Proceed on with whatever your program was designed to do.
//!
//! Differences to the old `getopt`/`getopt_long` C solution include:
//!
//!  * All processing can be done in one go, rather than with recursive function calls.
//!  * Use of static global variables for "position" tracking is avoided.
//!  * *Non-options* are **not** shuffled to the end of the list, unlike the default behaviour of
//!    `getopt`.
//!  * The "convenience" functionality of `-W foo` being treated as `--foo` is not supported
//!    (unnecessary complexity).
//!
//! ## Mismatch suggestions
//!
//! This library does not (currently) itself provide any suggestion mechanism for failed option
//! matches - i.e. the ability to take an unmatched *long option* and pick the most likely of the
//! available options that the user may have actually meant to use, to suggest to them when
//! reporting the error. There is nothing however stopping users of this library from running
//! unmatched options through a third-party library to obtain the suggestion to display.
//!
//! # Crate name origins
//!
//! The `gong` crate name was chosen in homage to the venerable C `getopt`, and can be considered a
//! "next-gen" solution for people moving to the Rust programming language.
//!
//! > **G**et-**O**pt **N**ext-**G**en → `gong`
//!
//! Hopefully this crate will become as common and universally adopted as `getopt` in C code.
