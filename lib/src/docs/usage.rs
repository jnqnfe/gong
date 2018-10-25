// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument processing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Usage
//!
//! # Preparation
//!
//! To use this library, start by adding a dependency entry for it in your project’s `Cargo.toml`
//! file; then make sure to declare use of the crate at the root of the module hierarchy
//! (`src/main.rs` or `src/lib.rs`), including importing the macros if you want to use them:
//!
//! ```rust
//! #[macro_use]
//! extern crate gong;
//! # fn main() {}
//! ```
//!
//! Now proceed with the following steps.
//!
//! # Step #1: Describe the available options
//!
//! First, you need to create a description of the options to be made available to users of your
//! program.
//!
//!  - [`OptionSetEx`] is the “extendible” type, which uses `Vec` for holding the described options,
//!    and can thus be extended with additional ones at any time. It is thus suitable for “builder”
//!    style construction, where a set is to be built dynamically at runtime.
//!  - [`OptionSet`] is designed for describing a “fixed” set of options, using a slice reference
//!    instead of `Vec`. It is primarily intended for achieving greater efficiency in designs not
//!    requiring dynamic construction, where a set can be declared as a `static` (though is not
//!    limited to use in `static`s).
//!
//! “Builder” style:
//!
//! ```rust
//! use gong::options::OptionSetEx;
//! let mut opts = OptionSetEx::new();
//! opts.add_long("help")
//!     .add_short('h')
//!     .add_long("foo")
//!     .add_long("version")
//!     .add_long("foobar")
//!     .add_long("ábc")
//!     .add_long_data("hah") // This one expects a data arg
//!     .add_short('❤')
//!     .add_short('x')
//!     .add_short_data('o'); // So does this one
//! debug_assert!(opts.is_valid());
//! ```
//!
//! “Fixed” style:
//!
//! ```rust
//! # #[macro_use]
//! # extern crate gong;
//! static OPTS: gong::options::OptionSet = gong_option_set_fixed!(
//!     [
//!         gong_longopt!("help"),
//!         gong_longopt!("foo"),
//!         gong_longopt!("version"),
//!         gong_longopt!("foobar"),
//!         gong_longopt!("ábc"),
//!         gong_longopt!("hah", true), // This one expects a data arg
//!     ], [
//!         gong_shortopt!('h'),
//!         gong_shortopt!('❤'),
//!         gong_shortopt!('x'),
//!         gong_shortopt!('o', true),  // So does this one
//!     ]
//! );
//! # fn main() {
//! debug_assert!(OPTS.is_valid());
//! # }
//! ```
//!
//! Notes:
//!
//!  - An [`OptionSetEx`] can be created from an [`OptionSet`] with
//!    [`to_extendible`][`OptionSet::to_extendible`].
//!  - An [`OptionSet`] can similarly be created from an [`OptionSetEx`] with
//!    [`as_fixed`][`OptionSetEx::as_fixed`]. It will hold slice references to the [`OptionSetEx`]’s
//!    `Vec` lists, with the lifetime tied to it (thus the set cannot be modified whilst the
//!    [`OptionSet`] exists).
//!  - Macros are provided for constructing both as a convenience.
//!
//! ## Set mode
//!
//! If you want to use *alternate* option mode rather than *standard* (default), as discussed above,
//! a `set_mode` method is available.
//!
//! You can control whether or not to allow abbreviated matching with the `set_allow_abbreviations`
//! method.
//!
//! ## Validation
//!
//! Once an option set has been described, it should be validated before use. The `is_valid` and
//! `validate` methods are provided for this. It is recommended that you typically only check
//! validity it in a *debug* assert variant, as here, to allow catching mistakes in development, but
//! otherwise avoid wasting energy for option sets in release builds that you know must be perfectly
//! valid.
//!
//! Some basic validation is also performed directly by the `add_*` methods on [`OptionSetEx`], but
//! this does not cover checking for duplicates.
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
//! With input args gathered and “available” option set constructed, now you’re ready for analysis.
//! All you need to do is feed the argument list to the option set’s `process` method and it will
//! spit out an analysis that describes what it identified.
//!
//! ```rust
//! # let opts: gong::options::OptionSetEx = Default::default();
//! # let args: Vec<String> = std::env::args().collect();
//! let analysis = opts.process(&args[..]);
//! ```
//!
//! Of course if for any reason you do **not** want to process all arguments in one go, you always
//! have the option of processing one argument at a time (or in groups of whatever number you
//! choose), calling `process` for each. (Naturally though you must beware the complications
//! handling “in-next-arg” *data values* doing this).
//!
//! # Step #4: Take action
//!
//! It is now up to you to take appropriate action in response to what was found.
//!
//! The [`Analysis`] object returned by the `process` method contains `error` and `warn` booleans,
//! which give a quick indication of problems. It also contains a list of items, describing in
//! detail what was found. The items in the item list are stored in the same order as found in the
//! input arguments.
//!
//! The entries in the item list are [`ItemClass`] variants, which wrap variants of [`Item`],
//! [`ItemW`] or [`ItemE`] \(okay/warn/error), thus making it simple to match by class. All variants
//! of each item class hold a `usize` value to be used for indicating the index of the argument in
//! which the item was found, should you want to know that. Similarly, information is returned where
//! applicable with *data values* as to whether the data arg was located in the same argument or the
//! next.
//!
//! **Note**: some item variants that may be returned in the [`Analysis`] struct hold `&str`
//! references to strings that were provided in the argument and option data provided to `process`.
//! This is done for efficiency. Beware of this with respect to lifetimes.
//!
//! # Have a play
//!
//! The source code repository that houses this project includes a small test application for trying
//! out the library’s analysis capabilities. It has a small set of built-in example options of
//! different kinds, and when run, outputs details of them along with details of analysing any
//! provided arguments against them. Instruction on using it are provided in the `README.md` file
//! that accompanies it.
//!
//! [`ItemClass`]: ../../analysis/enum.ItemClass.html
//! [`Item`]: ../../analysis/enum.Item.html
//! [`ItemW`]: ../../analysis/enum.ItemW.html
//! [`ItemE`]: ../../analysis/enum.ItemE.html
//! [`Analysis`]: ../../analysis/struct.Analysis.html
//! [`OptionSet`]: ../../options/struct.OptionSet.html
//! [`OptionSetEx`]: ../../options/struct.OptionSetEx.html
//! [`OptionSet::to_extendible`]: ../../options/struct.OptionSet.html#method.to_extendible
//! [`OptionSetEx::as_fixed`]: ../../options/struct.OptionSetEx.html#method.as_fixed
