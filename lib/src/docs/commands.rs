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
//! See the [arguments overview][arguments] for an introductory explanation of what *command*
//! arguments are. The following gives additional details of how they work (at least with respect to
//! this parsing library).
//!
//! *Command arguments* have a command *name*, an associated *option set* (which may be empty), and
//! a set of *sub-commands* (which also may be empty). Each *sub-command* has its own *name*,
//! *option set* and *sub-command set*. There is no fixed limit to the size and depth of the tree of
//! *commands* and nested *sub-commands* that can be built for a program.
//!
//! When parsing a given set of arguments, initially arguments are parsed against the “main”
//! (“top-level”) options in the *option set* given directly to the parser. If a *non-option* is
//! encountered, and it is the first *non-option*, it is compared to the “main” (“top-level”) set of
//! *commands*, if there are any. If there are no commands, then it would simply be considered to be
//! a *positional*, but if there are commands, then a match will be looked for, and it will either
//! thus be recognised as a known command, or otherwise reported as an unknown command. In the case
//! of a known command, all subsequent arguments from that point forward will be parsed against the
//! *option set* and *sub-command set* of that command. If a command has no sub-command set, then
//! all *non-options* will be taken to be *positionals*. In the case of an unknown command, parsing
//! of remaining arguments will continue with the same option/command sets, but naturally the
//! results will be incorrect if it genuinely was intended as a command (you are free to reinterpret
//! the unknown command argument to be a positional instead if applicable to your program, in which
//! case ignore that). Upon encountering an unknown command, no further attempt will be made to
//! interpret *non-options* as *commands* in subsequent arguments, they will be taken to be
//! *positionals*.
//!
//! Note, *command name* matching, like *option* matching, is case-sensitive. Similarly, abbreviated
//! matching is supported, though is disabled by default (you can enable it with a parser setting).
//!
//! Visually, the usage model for a program with *command arguments* looks like this:
//!
//! > <prog-name> [main-options] [command [cmd-options] [sub-cmd [sub-cmd-opts] [...]]]
//!
//! Per the above description, *positional* arguments cannot come before *command* arguments, they
//! must always only appear after the last command, mixed with any remaining *option* arguments. Any
//! attempt to use a *positional* before a command would result in it being interpretted as an
//! unknown command. (Unless of course the positional happens to unintentially match a command).
//!
//! Thus, where a *command set* (or *sub-command set*) is active, i.e. one or more commands are
//! available for use, but you also accept one or more *positionals*, understand that the first
//! *positional* will come out as an unknown command item. You would have to reinterpret this as a
//! *positional* (by retrieiving a copy of the original argument, via the given index). Also to be
//! considered in such a situation is the issue of the unknown command interpretation with respect
//! to the stop-on-problem parser setting.
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
//! # thus is an unknown command. The `build` argument then ends up being simply
//! # interpreted as a positional.
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
//! [arguments]: ../arguments/index.html
//! [option support]: ../options/index.html
