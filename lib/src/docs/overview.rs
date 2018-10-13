// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-APACHE and LICENSE-MIT files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: About & design
//!
//! A common requirement of a program is the need to process command line arguments supplied to it.
//! This library was designed to *assist* Rust based programs with this.
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
//! # Design
//!
//! In C programming the common `getopt` and related `getopt_long` functions have, for a long time,
//! served to assist in this area. This library was designed as a similar *assistant*.
//!
//! The basic premise of usage is simple - provide the processing function with a set of available
//! options and the input arguments to be processed, and it returns the results of its analysis.
//! From there you can take further action - output error information if the user made a mistake;
//! output help/usage information if requested; store state information from flag type options; and
//! store data (converting values as necessary) from non-options and options with data, before
//! proceeding with whatever your program was designed to do.
//!
//! Some major differences to the old `getopt`/`getopt_long` C solution include:
//!
//!  - All processing can be done in one go, rather than with recursive function calls;
//!  - "Non-options" are **not** shuffled to the end of the list, unlike the default behaviour of
//!    'getopt';
//!  - The "convenience" functionality of `-W foo` being treated as `--foo` is not supported
//!    (unnecessary complexity).
