// Copyright 2018 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: “Command” argument support
//!
//! This crate has built-in understanding of *command arguments*, thus simplifying argument parsing
//! for applications that use them.
//!
//! *Command arguments* are *non-option* arguments (see [option support]), which are given special
//! meaning by an application, used to select its execution mode (e.g. specify an “action” that it
//! should perform). They also affect argument parsing by changing the set of *options* that
//! subsequent arguments are matched against, to a potentially different set, i.e. an *option set*
//! specific to the given *command*.
//!
//! `Cargo` and `Git` are two examples of programs that use *command arguments*.
//!
//!  - `Cargo`’s include: `build`, `test`, and `run`
//!  - `Git`’s include: `clone`, `branch`, `add`, and `push`
//!
//! *Command arguments* have a command *name*, an associated *option set* (which may be empty), and
//! a set of *sub-commands* (which also may be empty). Each *sub-command* has its own *name*,
//! *option set* and *sub-command set*. There is no fixed limit to the size and depth of the tree of
//! *commands* and nested *sub-commands* that can be built for a program.
//!
//! When parsing a given set of arguments, initially arguments are parsed against the “main”
//! (“top-level”) options in the *option set* given directly to the parser. If a *non-option* is
//! encountered, and it is the first *non-option*, it is compared to the “main” (“top-level”) set of
//! *commands*, if there are any. If it does not match any *command* in the set, it is considered to
//! be a *positional* argument, but if it does, then it is promoted to *command*, and all subsequent
//! arguments will be parsed against the *option set* and *sub-command set* of that command. If the
//! first *non-option* following that command matches one of its *sub-commands*, then it is promoted
//! to a *command* in exactly the same way.
//!
//! Note, *command name* matching, like *option* matching, is case-sensitive. However, unlike
//! *option* matching, abbreviated matching is not (currently) supported for *command names*.
//!
//! Visually, the usage model for a program with *command arguments* looks like this:
//!
//! > <prog-name> [main-options] [command [cmd-options] [sub-cmd [sub-cmd-opts] [...]]]
//!
//! Per the above description, *positional* arguments cannot come before *command* arguments, they
//! must always only appear after the last command, mixed with any remaining *option* arguments. Any
//! attempt to use a *positional* before a command would result in the command being interpretted as
//! a *positional*.
//!
//! ## Example
//!
//! Take an example program designed with the following *option*/*command* structure:
//!
//! ```text
//! --help              # Top-level option
//! --verbose           # Top-level option
//! build               # Top-level command
//!     --help          # `build` command option
//!     --action        # `build` command option
//!     foo             # `build` command sub-command (with no further sub-commands)
//!         --print     # `build::foo` command option
//!     bar             # `build` command sub-command (with no options, nor further sub-commands)
//! test                # Top-level command (with no options, nor further sub-commands)
//! ```
//!
//! Then here is how some example interactions with this program would be interpreted:
//!
//! ```text
//! # This asks for the main program help output.
//! <prog-name> --help
//!
//! # This uses the main `verbose` option, the `build` command, and the `build`
//! # command’s `action` option.
//! <prog-name> --verbose build --action
//!
//! # Here, `blah` is a non-option that does not match a recognised command, and
//! # thus is a positional. Since `build` is not the first non-option, it too is
//! # simply interpreted as a positional.
//! <prog-name> blah build
//!
//! # Here the `build` command is used, followed by its `help` option, which
//! # requests help output specific to the `build` command.
//! <prog-name> build --help
//!
//! # Here, the `build` command is used, followed by use of its `action` option
//! # and then its `foo` sub-command. `blah` is a non-option, but since `foo`
//! # has no futher sub-commands, it is simply considered to be a *positional*.
//! <prog-name> build --action foo blah
//!
//! # Here, the `build` command is used, followed by its `foo` sub-command. This
//! # is then followed by a long option `help`, but the `build::foo` sub-command
//! # has no such option, so it will come out as unrecognised.
//! <prog-name> build foo --help
//! ```
//!
//! [option support]: ../options/index.html
