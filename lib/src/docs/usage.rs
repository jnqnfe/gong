// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
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
//! # Step #1: Create a `Parser`
//!
//! A [`Parser`] holds: a description of the available [*options*][options_doc]; a description of
//! the available [*commands*][commands_doc]; and settings to control parsing. It also provides the
//! [`parse`][`Parser::parse`] method, that performs the actual parsing.
//!
//! One of the first things you need to do therefore, is construct a [`Parser`].
//!
//! ## Describe the available options
//!
//! First, you need to create a description of the options to be made available to users of your
//! program.
//!
//!  - [`OptionSetEx`] is the “extendible” type, which uses `Vec` for holding the described options,
//!    and can thus be extended with additional ones at any time. It is thus suitable for “builder”
//!    style construction, where a set is to be built dynamically at runtime.
//!  - [`OptionSet`] is designed for describing a “fixed” set of options, using a slice reference
//!    instead of `Vec`. It is primarily intended for achieving greater efficiency in designs not
//!    requiring dynamic construction.
//!
//! An example of *option set* construction, “builder” style:
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
//! An example of *option set* construction, “fixed” style, using macros:
//!
//! ```rust
//! # #[macro_use]
//! # extern crate gong;
//! use gong::options::OptionSet;
//! static OPTS: OptionSet = gong_option_set_fixed!(
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
//! You are encouraged to care about efficiency and thus use the latter model wherever possible. In
//! some cases where a small amount of dynamic addition is needed, you might choose to start with an
//! [`OptionSet`], and use [`OptionSet::to_extendible`].
//!
//! Notes:
//!
//!  - An [`OptionSet`] can be created from an [`OptionSetEx`] with
//!    [`as_fixed`][`OptionSetEx::as_fixed`]. It will hold slice references to the [`OptionSetEx`]’s
//!    `Vec` lists, with the lifetime tied to it (thus the set cannot be modified whilst the
//!    [`OptionSet`] exists).
//!  - An [`OptionSetEx`] can be created from an [`OptionSet`] with
//!    [`to_extendible`][`OptionSet::to_extendible`].
//!  - Macros are provided for constructing both as a convenience.
//!
//! ## Describe the available command arguments
//!
//! For programs designed with *[command arguments][commands_doc]*, in addition to describing a
//! “main” *option set*, a *command set* must also be described, for which [`CommandSetEx`] and
//! [`CommandSet`] types are provided, similar to the respective *option set* types.
//!
//! Note that [`Command`]s each hold an [`OptionSet`], and may hold a [`CommandSet`] of
//! *sub-commands*.
//!
//! An example of constructing a *command* based structure is not given here, but it should be
//! fairly trivial to understand how to achieve.
//!
//! ## Create the `Parser`
//!
//! ```rust
//! use gong::parser::Parser;
//! # let opts = gong::options::OptionSet::default();
//! # let cmds = gong::commands::CommandSet::default();
//! let parser = Parser::new(&opts, Some(&cmds));
//! ```
//!
//! Note that the [`Parser`] only accepts [`OptionSet`] and [`CommandSet`] types, not the extendible
//! variants, so if you have used the extendible ones, you must use the respective `as_fixed`
//! methods.
//!
//! ```rust
//! use gong::parser::Parser;
//! # let opts = gong::options::OptionSetEx::default();
//! # let cmds = gong::commands::CommandSetEx::default();
//! let opts_fixed = opts.as_fixed();
//! let cmds_fixed = cmds.as_fixed();
//! let parser = Parser::new(&opts_fixed, Some(&cmds_fixed));
//! ```
//!
//! If you want to change any parser settings, e.g. choose which *option* mode (*standard* or
//! *alternate*) is used, or whether or not abbreviated long option name matching is allowed, you
//! can control this now via the parser’s `settings` attribute.
//!
//! ### Validation
//!
//! Once a parser has been built, it should be validated before use to ensure that there are no
//! issues with the *option set* and *command set* you have described. The
//! [`is_valid`][`Parser::is_valid`] and [`validate`][`Parser::validate`] methods are provided for
//! this. It is recommended that you typically only check validity in a *debug* assert variant,
//! to allow catching mistakes in development, but otherwise avoid wasting energy in release builds
//! for parser descriptions that you know must be perfectly valid.
//!
//! Note, some basic validation is performed directly by the `add_*` methods on [`OptionSetEx`] and
//! [`CommandSetEx`], but this does not cover checking for duplicates. The *option set* and
//! *command set* structures also have their own validation checking methods, which are used
//! internally by the *parser* validation checks. There is no need to run them in addition to
//! checking the *parser*.
//!
//! **Note**: With respect to what is or is not a duplicate, only the name/`char` of the *option* or
//! *command* matters; the `expects_data` attribute of *options* and *option set* and *sub-command
//! set* of *commands* make no difference.
//!
//! # Step #2: Gather arguments to be parsed
//!
//! You also need to retrieve (or build) a set of arguments to be parsed. A simple example:
//!
//! ```rust
//! let args: Vec<String> = std::env::args()
//!                             .skip(1)        // Skip the program-name/path argument
//!                             .collect();
//! ```
//!
//! Notes:
//!
//! 1. The very first entry in an argument list is the program path/name, and often you will not be
//!    interested in it. It is usually best to just skip it. You can do so in two easy ways, with
//!    the `skip()` method when collecting, as above, or alternatively with the slice range used
//!    when parsing (i.e. `&args[1..]` instead of `&args[..]`).
//! 2. Rust’s `std` library provides two functions for obtaining arguments, `std::end::args()` and
//!    `std::env::args_os()`; see the [Unicode discussion chapter][unicode_doc] of the documentation
//!    module for information on which you should chose.
//! 3. The parser methods also accept argument lists in string reference form (`&str` and `&OsStr`
//!    respectively).
//!
//! # Step #3: Parse
//!
//! With input args gathered and “available” *options* and *commands* described, now you’re ready
//! for parsing. All you need to do is feed the argument list to the parser’s
//! [`parse`][`Parser::parse`] method and it will spit out an analysis that describes what it
//! identified.
//!
//! ```rust
//! # let opts = gong::options::OptionSet::default();
//! # let cmds = gong::commands::CommandSet::default();
//! # let parser = gong::parser::Parser::new(&opts, Some(&cmds));
//! # let args: Vec<String> = std::env::args().collect();
//! let analysis = parser.parse(&args[..]);
//! ```
//!
//! If you are taking arguments in `OsString` form, as discussed above, you should use the alternate
//! `parse_os` method here instead.
//!
//! # Step #4: Take action
//!
//! It is now up to you to take appropriate action in response to what was found.
//!
//! The [`Analysis`] object returned by the [`parse`][`Parser::parse`] method contains `error` and
//! `warn` booleans, which give a quick indication of problems. It also contains a list of items,
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
//! > **Note:** some item variants that may be returned in the [`Analysis`] struct hold string
//! > references to strings that were provided in the argument and option data provided to
//! > [`parse`][`Parser::parse`]. This is done for efficiency. Beware of this with respect to
//! > lifetimes.
//!
//! # Have a play
//!
//! The source code repository that houses this project includes a small test application for trying
//! out the library’s analysis capabilities. It has a small set of built-in example options of
//! different kinds, and when run, outputs details of them along with details of analysing any
//! provided arguments against them. Instruction on using it are provided in the `README.md` file
//! that accompanies it.
//!
//! [`Parser`]: ../../parser/struct.Parser.html
//! [`Parser::parse`]: ../../parser/struct.Parser.html#method.parse
//! [`Parser::parse_os`]: ../../parser/struct.Parser.html#method.parse_os
//! [`Parser::is_valid`]: ../../parser/struct.Parser.html#method.is_valid
//! [`Parser::validate`]: ../../parser/struct.Parser.html#method.validate
//! [`Settings`]: ../../parser/struct.Settings.html
//! [`OptionSet`]: ../../options/struct.OptionSet.html
//! [`OptionSetEx`]: ../../options/struct.OptionSetEx.html
//! [`OptionSet::to_extendible`]: ../../options/struct.OptionSet.html#method.to_extendible
//! [`OptionSetEx::as_fixed`]: ../../options/struct.OptionSetEx.html#method.as_fixed
//! [`Command`]: ../../commands/struct.Command.html
//! [`CommandSet`]: ../../commands/struct.CommandSet.html
//! [`CommandSetEx`]: ../../commands/struct.CommandSetEx.html
//! [`Analysis`]: ../../analysis/struct.Analysis.html
//! [`ItemClass`]: ../../analysis/enum.ItemClass.html
//! [`Item`]: ../../analysis/enum.Item.html
//! [`ItemW`]: ../../analysis/enum.ItemW.html
//! [`ItemE`]: ../../analysis/enum.ItemE.html
//! [commands_doc]: ../commands/index.html
//! [options_doc]: ../options/index.html
//! [unicode_doc]: ../unicode/index.html
