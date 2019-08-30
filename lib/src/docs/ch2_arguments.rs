// Copyright 2019 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Arguments overview
//!
//! This is an overview of the different types of command line arguments that can be supplied to a
//! program, and which this argument parsing library directly understands and handles. The purpose
//! of this documentation chapter is to provide overview and understanding of basic argument types
//! and terminology, for those needing such introduction.
//!
//! The basic set of argument types consists of the following:
//!
//! * *Positional* arguments
//! * *Option* arguments, both of “short” form and “long” form
//! * *Command* arguments
//! * The “*early terminator*” argument
//!
//! # Positional arguments
//!
//! *Positional* arguments are simple, *generic* arguments, basically any argument which is not
//! interpreted as an *option* argument, a *command* argument, or an *early terminator*. The term
//! “positional” relates to the fact that the position (index number) in the argument list is the
//! only thing available to determine the association between the argument and what it is to be used
//! for (i.e. the first positional is for X, the second for Y).
//!
//! # Option arguments
//!
//! *Option* arguments are arguments which make use of a fixed identifier that associates them with
//! a particular purpose, and thus allows more flexibility compared to pure *positional* arguments,
//! for instance in terms of the order arguments are given.
//!
//! They come in multiple forms, with two key aspects of classification; one being the form of the
//! identifier and the other being whether or not they come with a data value.
//!
//! In terms of the identifier, *options* come in both *short option* form which uses a single
//! `char`, and *long option* form which uses a “name”. Both identifiers are given after a
//! particular prefix that explicitly marks an argument as being an *option*. Commonly this is a
//! double-dash for *long options* (e.g. `--help`) and a single dash for *short options* (e.g.
//! `-h`), however some program designs only provide options in *long option* form and use a single
//! dash prefix for them (e.g. `-help`). Both models are supported by this parser.
//!
//! In terms of data values, there are three variations:
//!
//!  - *Flag* type options (sometimes known as the *switch* type) do not come with a data value;
//!    their presence alone within a set of input arguments means something to the program.
//!    Typically their use will toggle the default state of some boolean feature.
//!  - *Data-taking* type options require a single accompanying value. The benefit of such options
//!    over positionals for providing data input arguments to a program, as mentioned, is the
//!    flexibility that options offer, in terms of the order they are given for instance.
//!  - *Mixed* type options (as we call them here) are those that are a mixture of the two previous
//!    forms. The single data value is optional. There is a limitation however, values can only be
//!    provided within the same argument (something discussed in the dedicated *options* chapter
//!    linked below. This restriction is necessary to remove abiguity in parsing.
//!
//! Note that when it come to *short options*, more than one can be given within a single argument,
//! as a sequence of `chars` following the *short option* prefix, for example `-abc`. We refer to
//! this as a *short option set*.
//!
//! More details on *option* support can be found within the [dedicated *options* chapter][options].
//!
//! Note that this library in some places refers to arguments that are not *option* arguments as
//! *non-option* arguments.
//!
//! # Early terminator
//!
//! An *early terminator* is an argument which consists entirely of two dashes only (`--`). It is
//! a special argument used to control interpretation of arguments. More specifically, it is used to
//! request that all remaining arguments simply be assumed to be *positionals*, thus that no attempt
//! be made to interpret them as *options*, nor anything else. (I.e. it requests early termination
//! of argument interpretation). This provides users with a means of preventing arguments that look
//! like *option* arguments from being parsed as such, when needing to supply them as *positionals*.
//!
//! This is useful for instance if a program passes along some or all *positionals* to something
//! else, and the user wants some *option* arguments to be passed along.
//!
//! An example is demonstrated below.
//!
//! # Command arguments
//!
//! *Command* arguments are arguments that consist simply of a name, to which special meaning is
//! given by an application. They are used by some programs that offer a lot of functionality to
//! given a means of selecting which action to perform. (Typically there will be a set of commands
//! from which the user may make a choice).
//!
//! Unlike *options* they have nothing special about them to clearly distinguish them from
//! *positionals*, but there are rules that come into play regarding where they can be placed within
//! an argument list. They also affect argument parsing through the fact that once a command
//! argument has been given, all subsequent arguments in the argument list are parsed within the
//! context of that command (a command has its own set of available options and possibly a set of
//! sub-commands, for instance).
//!
//! Cargo and Git are two examples of programs you should likely be familiar with that use *command*
//! arguments.
//!
//!  - `Cargo`’s include: `build`, `test`, and `run`
//!  - `Git`’s include: `clone`, `branch`, `add`, and `push`
//!
//! More details on *command* support can be found within the [dedicated *commands*
//! chapter][commands].
//!
//! # Example
//!
//! The following is an example set of arguments given in an invocation of the `cargo` program,
//! which as a Rust programmer you should be familiar with. This program makes use of *command*
//! arguments, handles both *positionals* and *options*, and respects the *early terminator*, so is
//! perfect for demonstration purposes here.
//!
//! ```text
//! cargo run --release -- --foo --bar
//! ```
//!
//! The first argument (`run`) is a *command* argument and determines `cargo`’s “mode”. As you
//! should know, in “run” mode `cargo` *runs* the binary program of a Cargo project. When it does so
//! it passes along all *positional* arguments as arguments of the invocation of your program.
//!
//! So as for the remaining arguments, the `--release` argument is consumed by Cargo as a *long
//! option*; the `--` argument is an *early terminator*, forcing all remaining arguments to be
//! interpreted as *positionals*, and thus the `--foo` and `--bar` arguments, being *positionals*
//! are thus passed along to the program to be run by Cargo. Thus effectively this is the same as
//! directly running the project’s binary as (after compiling in release mode of course):
//!
//! ```text
//! <my-prog> --foo --bar
//! ```
//!
//! Without the *early terminator* `cargo` would have seen `--foo` and `--bar` to be *options* and
//! thus tried to consume them for itself (resulting in unrecognised *option* errors).
//!
//! [overview]: ../ch1_overview/index.html
//! [options]: ../ch2_options/index.html
//! [commands]: ../ch4_commands/index.html
//! [unicode]: ../ch5_unicode/index.html
