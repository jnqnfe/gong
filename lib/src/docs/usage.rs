// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Usage
//!
//! # Step #0: Preparation
//!
//! To use this library, start by adding a dependency entry for it in your project's `Cargo.toml`
//! file; then make sure to declare use of the crate at the root of the module hierarchy
//! (`src/main.rs` or `src/lib.rs`):
//!
//! ```rust
//! //#[macro_use] # Uncomment if you want to use the macros
//! extern crate gong;
//! ```
//!
//! Now proceed with the following steps.
//!
//! # Step #1: Describe the available options
//!
//! First, you need to create a description of the options to be made available to users of your
//! program. Here you actually have some choices as to how you do this. You can use the "builder"
//! style, using `add_*` methods on an option structure, or you can use the macros.
//!
//! "Builder" style:
//!
//! ```rust
//! let mut opts = gong::Options::new(6, 4); //Estimate counts for efficiency
//! opts.add_long("help")
//!     .add_short('h')
//!     .add_long("foo")
//!     .add_long("version")
//!     .add_long("foobar")
//!     .add_long("ábc")      // Using a combining char (accent)
//!     .add_long_data("hah") // This one expects a data arg
//!     .add_short('❤')
//!     .add_short('x')
//!     .add_short_data('o'); // So does this one
//! debug_assert!(opts.is_valid());
//! ```
//!
//! ## Set mode
//!
//! If you want to use *alternate* option mode rather than *standard* (default), as discussed above,
//! the [`Options::set_mode`] method is available.
//!
//! You can control whether or not to allow abbreviated matching with the
//! [`Options::set_allow_abbreviations`] method.
//!
//! ## Validation
//!
//! Some validation is performed by the `add_*` methods, but for full validation (including checking
//! for duplicates) the [`Options::is_valid`] method is provided, as used above. Details of any
//! problems identified by this method are output to `stderr`. It is recommended that you only use
//! it in a *debug* assert variant, as here, to allow catching mistakes in development, but
//! otherwise avoid wasting energy for option sets in release builds that you know must be perfectly
//! valid.
//!
//! **Note**: With respect to what is or is not a duplicate, only the name/`char` matters; the
//! `expects_data` attribute makes no difference.
//!
//! # Step #2: Gather arguments to be processed
//!
//! You also need to retrieve (or build) a set of arguments to be processed. Slices of both `&str`
//! and `String` are supported. (Rust provides actual program args from the environment as
//! `String`). You can collect program arguments as follows:
//!
//! ```rust
//! let args: Vec<String> = std::env::args().collect();
//! ```
//!
//! The very first entry in the list is the program path/name, and often you will not be interested
//! in it. You can skip it in two easy ways, either: a) providing `&args[1..]` instead of
//! `&args[..]` to the processing function in the next step, or b) using the iterator `skip` method,
//! as here:
//!
//! ```rust
//! let args: Vec<String> = std::env::args().skip(1).collect();
//! ```
//!
//! **Note**: Of course you do not have to provide the real program args, you can provide any set of
//! `String` objects, and you can even of course take the real set and modify it first if you wish.
//!
//! # Step #3: Processing
//!
//! With input args gathered and "available" option set constructed, now you're ready for analysis.
//! All you need to do is feed these two data sets to the [`process`] function and it will spit out
//! an analysis that describes what it identified.
//!
//! ```rust
//! # let opts: gong::Options = Default::default();
//! # let args: Vec<String> = std::env::args().collect();
//! let analysis = gong::process(&args[..], &opts);
//! ```
//!
//! Of course if for any reason you do **not** want to process all arguments in one go, you always
//! have the option of processing one argument at a time (or in groups of whatever number you
//! choose), calling [`process`] for each. (Naturally though you must beware the complications
//! handling "in-next-arg" *data values* doing this).
//!
//! # Step #4: Take action
//!
//! It is now up to you to take appropriate action in response to what was found.
//!
//! The [`Analysis`] object returned by the [`process`] function contains `error` and `warn`
//! booleans, which give a quick indication of problems. It also contains a list of items,
//! describing in detail what was found. The items in the item list are stored in the same order as
//! found in the input arguments.
//!
//! The entries in the item list are [`ItemClass`] variants, which wrap variants of [`Item`],
//! [`ItemW`] or [`ItemE`] \(okay/warn/error), thus making it simple to match by class. All variants
//! of each item class hold a `usize` value to be used for indicating the index of the argument in
//! which the item was found, should you want to know that. Similarly, information is returned where
//! applicable with *data values* as to whether the data arg was located in the same argument or the
//! next.
//!
//! **Note**: some item variants that may be returned in the [`Analysis`] struct hold `&str`
//! references to strings that were provided in the argument and option data provided to
//! [`process`]. This is done for efficiency. Beware of this with respect to lifetimes.
//!
//! # Have a play
//!
//! The source code repository that houses this project includes a small test application for trying
//! out the library's analysis capabilities. It has a small set of built-in example options of
//! different kinds, and when run, outputs details of them along with details of analysing any
//! provided arguments against them. Instruction on using it are provided in the `README.md` file
//! that accompanies it.
//!
//! [`process`]: ../../fn.process.html
//! [`ItemClass`]: ../../analysis/enum.ItemClass.html
//! [`Item`]: ../../analysis/enum.Item.html
//! [`ItemW`]: ../../analysis/enum.ItemW.html
//! [`ItemE`]: ../../analysis/enum.ItemE.html
//! [`Analysis`]: ../../analysis/struct.Analysis.html
//! [`Options::is_valid`]: ../../options/struct.Options.html#method.is_valid
//! [`Options::set_mode`]: ../../options/struct.Options.html#method.set_mode
//! [`Options::set_allow_abbreviations`]: ../../options/struct.Options.html#method.set_allow_abbreviations
