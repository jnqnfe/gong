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
//! (`src/main.rs` or `src/lib.rs`):
//!
//! ```rust
//! extern crate gong;
//! # fn main() {}
//! ```
//!
//! Now proceed with the following steps.
//!
//! # Step #1: Create a parser
//!
//! A parser holds: a description of the available [*options*][options_doc]; a description of the
//! available [*commands*][commands_doc] (where applicable); and settings to control parsing. It
//! also provides the methods for performing the actual parsing.
//!
//! One of the first things you need to do therefore is construct a [`Parser`] or, if your program
//! will make use of *command arguments*, a [`CmdParser`]. We will begin here by creating
//! descriptions of the data that the parser object will need to hold.
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
//! use gong::options::{OptionSetEx, OptionType};
//! let mut opts = OptionSetEx::new();
//! opts.add_pair('h', "help", OptionType::Flag)
//!     .add_long("foo", OptionType::Flag)
//!     .add_pair('V', "version", OptionType::Flag)
//!     .add_long("foobar", OptionType::Flag)
//!     .add_long("ábc", OptionType::Flag)
//!     .add_long("hah", OptionType::Data)
//!     .add_long("delay", OptionType::OptionalData)
//!     .add_short('❤', OptionType::Flag)
//!     .add_short('x', OptionType::Flag)
//!     .add_short('o', OptionType::Data)
//!     .add_shorts_from_str("jk:l"); // Multiple shorts from string, where `k` takes data
//! ```
//!
//! An example of *option set* construction, “fixed” style, using macros:
//!
//! ```rust
//! use gong::{longopt, shortopt, option_set};
//! use gong::options::OptionSet;
//! static OPTS: OptionSet = option_set!(
//!     @long [
//!         longopt!(@flag "help"),
//!         longopt!(@flag "foo"),
//!         longopt!(@flag "version"),
//!         longopt!(@flag "foobar"),
//!         longopt!(@flag "ábc"),
//!         longopt!(@data "hah"),       // This one expects a mandatory data arg
//!         longopt!(@opt_data "delay"), // This one expects an optional data arg
//!     ],
//!     @short [
//!         shortopt!(@flag 'h'),
//!         shortopt!(@flag 'V'),
//!         shortopt!(@flag '❤'),
//!         shortopt!(@flag 'x'),
//!         shortopt!(@data 'o'),
//!     ]
//! );
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
//! Also, note that you do **not** *have* to describe the entire command tree up front, as will be
//! discussed in step #3.
//!
//! An example of constructing a *command* based structure is not given here, but it should be
//! fairly trivial to understand how to achieve.
//!
//! ## Create the parser itself
//!
//! Creating a [`Parser`] (or [`CmdParser`]) requires providing an *option set* reference (and in
//! the [`CmdParser`] case, also a *command set* reference).
//!
//! ```rust
//! use gong::parser::Parser;
//! # let opts = gong::options::OptionSet::default();
//! let parser = Parser::new(&opts);
//! debug_assert!(parser.is_valid());
//! ```
//!
//! Understand that with a command-based program design, the *option set* specified in parser
//! creation is the *top-level* set of *options* (see the *[command arguments documentation][commands_doc]*
//! for more information on this).
//!
//! Note that the [`Parser`] only accepts [`OptionSet`] and [`CommandSet`] types, not the extendible
//! variants, so if you have used the extendible ones, you must use the respective `as_fixed`
//! methods.
//!
//! ```rust
//! use gong::parser::CmdParser;
//! # let opts = gong::options::OptionSetEx::default();
//! # let cmds = gong::commands::CommandSetEx::default();
//! let opts_fixed = opts.as_fixed();
//! let cmds_fixed = cmds.as_fixed();
//! let parser = CmdParser::new(&opts_fixed, &cmds_fixed);
//! debug_assert!(parser.is_valid());
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
//! *command* matters; the [`OptionType`] of *options*, and *option set* and *sub-command set* of
//! *commands*, make no difference.
//!
//! # Step #2: Gather arguments to be parsed
//!
//! You also need to retrieve (or build) a set of arguments to be parsed. A simple example, skipping
//! the program-name argument:
//!
//! ```rust
//! let args: Vec<_> = std::env::args_os().skip(1).collect();
//! ```
//!
//! An alternate solution, capturing the program-name argument into a variable:
//!
//! ```rust
//! let mut args_iter = std::env::args_os();
//! let prog_name = args_iter.next().unwrap();
//! let args: Vec<_> = args_iter.collect();
//! ```
//!
//! Notes:
//!
//! 1. The very first entry in an argument list is the program path/name, and often you will not be
//!    interested in it. It is usually best to just skip it. You can do so in two easy ways, with
//!    the `skip()` method when collecting, as above, or alternatively with the slice range used
//!    when parsing (i.e. `&args[1..]` instead of `&args[..]`).
//! 2. Rust’s `std` library provides two functions for obtaining arguments, `std::end::args()` and
//!    `std::env::args_os()`. Understand that both Windows and Unix based (Linux and Mac OSx)
//!    operating systems allow file and folder names that may not be fully valid Unicode, while the
//!    primary string types in Rust must be fully valid, hence Rust offers “OS” variants which
//!    preserve the possible OS string quirks. With the latter function, Rust simply gives us the
//!    arguments in `OsString` form. With the former, they are converted to `String` form, panicking
//!    if they contained such quirks and thus could not be converted. The latter is what you should
//!    use here; the parser handles strings in OS form, which is what you will want for taking
//!    file or folder names. For other values you will want to later convert them to `&str` form,
//!    just as you may want to convert a value from string to integer form.
//! 3. The parser methods accept argument lists in both `&OsStr` and `&str` forms.
//! 4. The argument iterators given by `std::end::args()` and `std::env::args_os()` are more costly
//!    to create than you might realise; it involves parsing the entire set of OS provided C strings
//!    in order to generate a `Vec` of `CStr`. This is done on iterator creation; individual C
//!    string arguments are **not** processed on each iteration, and the results are not cached.
//!    Thus you should avoid creating more than one such instance where ever possible.
//!
//! # Step #3: Parse
//!
//! With input arguments gathered and “available” *options* and *commands* described, now you’re
//! ready for parsing. You have two choices here for how you want to approach this, either “one at a
//! time” (iterative) style or “all in one” (collect and data mine) style.
//!
//! The former is done with the `parse_iter` method (see [`Parser::parse_iter`] or
//! [`CmdParser::parse_iter`]) which returns an iterator, allowing arguments to be parsed and
//! responded to one at a time. The latter is done with the `parse` method (see [`Parser::parse`] or
//! [`CmdParser::parse`]); internally this uses the iterator method, collecting the results into an
//! object that has methods suitable for performing “data mining”. In the case of
//! [`CmdParser::parse`] the items are further partitioned per use of command arguments.
//!
//! An example of “one at a time” (iterative) analysis:
//!
//! ```rust
//! # let opts = gong::options::OptionSet::default();
//! # let cmds = gong::commands::CommandSet::default();
//! # let parser = gong::parser::CmdParser::new(&opts, &cmds);
//! # let args: Vec<_> = std::env::args_os().collect();
//! for item in parser.parse_iter(&args[..]) {
//!     // react to it...
//! }
//! ```
//!
//! An example of “all in one” (collect and data mine) analysis:
//!
//! ```rust
//! # let opts = gong::options::OptionSet::default();
//! # let cmds = gong::commands::CommandSet::default();
//! # let parser = gong::parser::CmdParser::new(&opts, &cmds);
//! # let args: Vec<_> = std::env::args_os().collect();
//! let analysis = parser.parse(&args[..]);
//! // now react to it...
//! ```
//!
//! One benefit of the “all in one” approach is that the [`ItemSet`] type provides a collection
//! of data-mining methods for extracting information, whereas the iterative approach involves a
//! large match block. However the iterative approach has its advantages also, with less overhead,
//! being more efficient, and as discussed below, not requiring up-front construction of an entire
//! command structure.
//!
//! Items are returned in both cases in the same order as respective arguments are given in the
//! input list.
//!
//! If your program uses commands and one or more commands are used in an argument list, the
//! analysis object returned by the “all in one” approach ([`CommandAnalysis`]) partitions the items
//! given by the iterator into multiple sets, per use of commands and how items are matched against
//! different option and command sets.
//!
//! Note that if you have nested sub-commands, you do **not** *have* to describe the full structure
//! up front when creating the [`CmdParser`] if you use the iterative approach; the iterator object
//! provides methods for changing the *option set* and *command set* for subsequent iterations, thus
//! you can specify the top-level command names only, with empty *option*/*command* sets, and on
//! encountering a command, give the iterator to the relevant function handling that command, which
//! can set the right *option* and *command* sets to use from that point onwards, before continuing
//! to iterate over any remaining arguments. (See the *[command arguments documentation][commands_doc]*
//! to understand how a command should affect parsing of subsequent items in an argument list).
//!
//! # Step #4: Take action
//!
//! It is now up to you to take appropriate action in response to what was found.
//!
//! In the case of data-mining with respect to the “all in one” approach, go ahead and simply use
//! the methods available on the [`ItemSet`] object. Of course if using commands, you need to
//! handle the partitioning first, and approach data-mining upon each individual [`ItemSet`].
//!
//! If not taking a data-mining approach, you need to grasp how *items* are described in the
//! returned analysis types. It is pretty straight forward. The analysis items are of the type
//! [`ItemResult`] which is just simply a type alias of `Result<Item, ProblemItem>` where [`Item`]
//! represents a good item and [`ProblemItem`] represents a problematic one.
//!
//! Matched and unmatched long-option/command names are returned in `&str` form, whilst positionals
//! and data values are returned in `&OsStr` form. If you wish to know the argument index number for
//! a given item, or the location from which option data was taken (same argument as the option
//! itself or the next argument), the iterator has methods for retrieving that information. You can
//! also alternatively use the `indexed` method of the iterators to convert to a wrapping iterator
//! which bundles this extra data along with each item in a tuple. (Note that it is excluded from
//! the data-mining object though, since unlikely to be wanted).
//!
//! Note that the [`ItemSet`] and [`CommandAnalysis`] objects both contain a `problems` boolean
//! which gives a quick indication of problems (of course).
//!
//! As just mentioned, strings representing *positional arguments* and *option data values* are
//! given in `&OsStr` form. This is ideal for those that represent filenames/paths, and can for
//! instance be converted directly to a `std::path::Path`. For other inputs you likely will want to
//! convert to `&str` form, which you can do either with `OsStr`’s `to_str` or `to_string_lossy`
//! methods. You will also want to convert to `&str` form first if, for example, wanting to try to
//! convert to an integer.
//!
//! > **Note:** some item variants that may be returned hold string references to strings that were
//! > provided in the argument and option data used by the parse method. This is done for
//! > efficiency. Beware of this with respect to lifetimes.
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
//! [`Parser::parse_iter`]: ../../parser/struct.Parser.html#method.parse_iter
//! [`Parser::is_valid`]: ../../parser/struct.Parser.html#method.is_valid
//! [`Parser::validate`]: ../../parser/struct.Parser.html#method.validate
//! [`CmdParser`]: ../../parser/struct.CmdParser.html
//! [`CmdParser::parse`]: ../../parser/struct.CmdParser.html#method.parse
//! [`CmdParser::parse_iter`]: ../../parser/struct.CmdParser.html#method.parse_iter
//! [`Settings`]: ../../parser/struct.Settings.html
//! [`OptionType`]: ../../options/enum.OptionType.html
//! [`OptionSet`]: ../../options/struct.OptionSet.html
//! [`OptionSetEx`]: ../../options/struct.OptionSetEx.html
//! [`OptionSet::to_extendible`]: ../../options/struct.OptionSet.html#method.to_extendible
//! [`OptionSetEx::as_fixed`]: ../../options/struct.OptionSetEx.html#method.as_fixed
//! [`Command`]: ../../commands/struct.Command.html
//! [`CommandSet`]: ../../commands/struct.CommandSet.html
//! [`CommandSetEx`]: ../../commands/struct.CommandSetEx.html
//! [`CommandAnalysis`]: ../../analysis/struct.CommandAnalysis.html
//! [`ItemSet`]: ../../analysis/struct.ItemSet.html
//! [`ItemResult`]: ../../analysis/type.ItemResult.html
//! [`Item`]: ../../analysis/enum.Item.html
//! [`ProblemItem`]: ../../analysis/enum.ProblemItem.html
//! [options_doc]: ../ch3_options/index.html
//! [commands_doc]: ../ch4_commands/index.html
//! [unicode_doc]: ../ch5_unicode/index.html
