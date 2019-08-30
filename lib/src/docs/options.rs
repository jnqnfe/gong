// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Option support
//!
//! This crate has been designed around standard *option* conventions.
//!
//! Basic feature support is on par with the C `getopt_long` function. (See the [overview] section
//! for mention of the small differences).
//!
//! # Option styles
//!
//! Firstly, if you don’t know what the terms *long* and *short* mean when referring to command line
//! *options*, these come from the fact that *short options* are identified and signalled with
//! single *characters* only (e.g. `h` traditionally is used to request help output and `V` for
//! version number output), while *long options* use a *name* (e.g. `help` or `version`,
//! respectively).
//!
//! There are two different common *option* styles this library can parse, which we refer to in this
//! library as the *option* parsing “mode”:
//!
//! - Standard mode (default) supports traditional *long* and *short* *options*, where *long
//!   options* use a “double-dash” (`--`) prefix (e.g. `--help`), and *short option sets* use a
//!   single dash (e.g. `-h`).
//! - Alternate mode supports *long options* only (no *short options*), with a single dash prefix
//!   (i.e. `-help` rather than `--help`). Some people simply prefer this style, and support for it
//!   was both trivial to add and involves very little overhead.
//!
//! > **Note:** *Short options* can still be added to an *option set* when using this mode, it will
//! > still pass as valid; they will simply be ignored when performing matching.
//!
//! Which style an argument list will be parsed with must be specified in the parser settings, at
//! some point prior to parsing.
//!
//! Note, from this point on, the below discussion is written towards *standard* mode conventions,
//! though everything applies equally to *alternate* mode, only with a few obvious details adjusted
//! and stuff relating to *short options* ignored, as applicable.
//!
//! # Basic parsing model
//!
//! The fundamental argument parsing logic follows this model:
//!
//!  - An argument either **not** starting with a dash (`-`), or consisting only of a single dash,
//!    is not an *option*, it is a *non-option* argument. A *non-option* argument is either a
//!    *positional* argument, or possibly a *[command argument][commands]*.
//!  - An argument of exactly two dashes (`--`) only is called an *early terminator*. This has
//!    special meaning, as described below.
//!  - An argument starting with two dashes (`--`) followed by additional characters is a *long
//!    option*. The portion after the double-dash prefix is the *long option name* (or possibly the
//!    name combined with an “in-same-argument” *data value*, as discussed below).
//!  - An argument starting with a single dash (`-`) followed by additional (non-dash) characters is
//!    a *short option set*, where each character (`char`) after the single-dash prefix represents a
//!    *short option* (except with respect to “in-same-argument” *data values*, as discussed below).
//!    (More than one *short option* can be specified in a single argument). Note that a dash (`-`)
//!    itself is not permitted to be a valid program *short option* (it would be misinterpreted in
//!    some cases). Note also that interpretation of what consists of a “character” may surprise you
//!    as actually being a complicated matter, as per the dedicated [unicode related
//!    discussion][unicode].
//!
//! Naturally, for *alternate* mode *option* style, ignore the discussion of *short options*, and
//! remember that only a single dash is used as the prefix for *long options*.
//!
//! Parsing of each argument may alter how one or more subsequent arguments are interpreted,
//! deviating from the above. Specifically this applies to the *early terminator* and where an
//! *option* is recognised as an *option* that takes a *data value*, as discussed below.
//!
//! Note, *option* matching is case-sensitive.
//!
//! # Data values
//!
//! *Long* and *short* *options* can be configured as either “flag” style (used to signal a
//! condition) or as “data taking” style, accepting an accompanying single *data value*.
//! *Data values* for both *long* and *short* *options* can either be supplied within the same
//! argument as the *option* itself (“in-same-argument”), or as the next argument, in which case
//! that will thus be consumed as the *option*’s *data value* and otherwise ignored.
//!
//!  - For *long options*, “in-next-argument” style looks like `--foo bar`, while “in-same-argument”
//!    style uses an equals (`=`) character between the *option name* and *value* components, e.g.
//!    `--foo=bar`. Note that a program’s “available” *long options* are forbidden from containing
//!    an equals (`=`) character in their name as this would otherwise introduce significant
//!    problems.
//!
//!    When parsing a *long option* argument, if the argument contains one or more equals (`=`)
//!    characters then it is considered to have an “in-same-argument” *data value* (since names are
//!    not permitted to contain them), and is split into two components upon the first. The left
//!    hand portion (without the double-dash prefix) is taken as the name, and the right as the
//!    “in-same-argument” *data value* (e.g. `--foo=bar` → name: “foo”, value: “bar”), with the
//!    equals (`=`) separator being discarded. This naturally occurs **before** checking for a
//!    matching “available” program *option*.
//!
//!     - If the name component does not match any “available” *long option*, then it is reported as
//!       unknown, with any “in-same-argument” *data value* component ignored.
//!     - If a match is found which **does not** take a *data value*, then if an “in-same-argument”
//!       *data value* component was supplied, its presence is reported as unexpected, otherwise all
//!       is good.
//!     - If a match is found that **does** take a *data value*, then if an “in-same-argument” *data
//!       value* component was present, this is consumed as such, otherwise the next argument is
//!       consumed. If an “in-same-argument” *data value* component was present, but the actual
//!       value is missing (e.g. as in `--foo=`), this does not matter, the *data value* is accepted
//!       as being an empty string (it does not consume the next argument). If in an
//!       “in-next-argument” situation, the next argument is an empty string (e.g. as in
//!       `--foo ""`), the *data value* is accepted as an empty string. If in an “in-next-argument”
//!       situation there is no next argument, then the *data value* is reported as missing.
//!
//!  - For *short options*, “in-next-argument” style looks like `-o arg`, while “in-same-argument”
//!    style looks like `-oarg`.
//!
//!    When a *short option set* is encountered (remember, more than one *short option* can be
//!    grouped together in the same argument), the characters are gone through in sequence, looking
//!    for a matching program *short option* for each.
//!
//!     - If no match is found then it is reported as unknown.
//!     - If a match is found that **does not** take a *data value*, then the match is simply
//!       reported.
//!     - If a match is found that **does** take a *data value*, then one needs to be found. If this
//!       character is **not** the last in the set, then the remaining portion of the argument is
//!       consumed as this *option*’s *data value* (e.g. if `o` is such an *option* then in `-oarg`,
//!       `arg` is it’s *data value*). If it is the last character in the set, then the next
//!       argument is consumed (e.g. `-o arg` → `arg`). If it is the last in the set and there is no
//!       next argument, then the *data value* is reported as missing.
//!
//!    Naturally when multiple *short options* are grouped together in the same argument, only the
//!    last in that group can be one that takes a *data value*, and users must be careful when
//!    constructing such groups.
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
//! An example use case is given below, using the `cargo` program, which as a Rust programmer you
//! should be familiar with. Note that the first argument, `run`, is a *non-option* which `cargo`
//! recognises and consumes as a *[command argument][commands]*, and determines `cargo`’s “mode”.
//! As you should know, in “run” mode, `cargo` *runs* the binary program of the `Cargo` project in
//! the *current working directory*, and does so passing along all *positional* arguments as input
//! arguments. So in the below example command line, `run` has already just been explained; the
//! `--release` argument is consumed by `cargo` as a *long option*; the `--` argument is an *early
//! terminator*, and the `--foo` and `--bar` arguments are thus considered *positionals*. Thus,
//! `cargo` here in “run” mode passes along `--foo` and `--bar` to the project program it runs. This
//! is equivalent to running `<my-prog> --foo --bar` directly. Without the *early terminator*,
//! `cargo` would have seen `--foo` and `--bar` to be *options* and thus tried to consume them for
//! itself (resulting in unrecognised *option* errors).
//!
//! ```text
//! cargo run --release -- --foo --bar
//! ```
//!
//! # Abbreviated long option name matching
//!
//! Abbreviated *long option* name matching is a feature whereby an abbreviated form of a *long
//! option’s* name can be used and matched successfully to the *option*, so long as the abbreviation
//! uniquely matches a single *long option*.
//!
//! This is supported and is optional. It is enabled by default, but can be opted out of when
//! parsing, if not desired, through a parser setting.
//!
//! As an example, take the input arguments from the following command line:
//!
//! ```text
//! <progname> --f --fo --foo --foob --fooba --foobar
//! ```
//!
//! If the feature is enabled, and `foo` and `foobar` are available *long options*, then:
//!
//!  - `--foo` and `--foobar` are exact matches for the available `foo` and `foobar` *options*
//!    respectively.
//!  - `--f` and `--fo` are invalid as being ambiguous (and reported as such by the parser).
//!  - `--foob` and `--fooba` both uniquely match `foobar` and so are valid.
//!
//! Note that an exact match always takes precedence.
//!
//! [overview]: ../overview/index.html
//! [commands]: ../commands/index.html
//! [unicode]: ../unicode/index.html
