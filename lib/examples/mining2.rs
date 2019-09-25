// Copyright 2019 Lyndon Brown
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Data-mining example #2 for the `gong` library
//!
//! This example demonstrates use of:
//!
//!  - Data-mining style
//!  - A very simple **static** option set (for maximum efficiency)
//!  - A fixed number of required positionals
//!  - A fixed string help text (for maximum efficiency)

extern crate gong;

use gong::{option_set, optpair};
use gong::analysis::ProblemItem;
use gong::arguments::Args;
use gong::options::{OptionSet, OptionPair};
use gong::parser::{Parser, OptionsMode};
use gong::positionals::Policy as PositionalsPolicy;

// Define our option constants
//
// This is a good idea for pairs of long and short options in particular. We can both generate the
// specific long and short option structs for the static option set via the `const` function methods
// it offers for doing so, and we can also generate the correct corresponding `FindOption` object
// for data-mining. All whilst also only declaring the type of option just once.
const HELP_OPT: OptionPair = optpair!(@flag 'h', "help");
const VERSION_OPT: OptionPair = optpair!(@flag 'V', "version");

// Our options
//
// These are divided up into two lists, one for long options and one for short, for efficiency.
static OPTIONS: OptionSet = option_set!(
    @long [
        HELP_OPT.as_long(),
        VERSION_OPT.as_long(),
    ],
    @short [
        HELP_OPT.as_short(),
        VERSION_OPT.as_short(),
    ]
);

// A static help text string, for efficiency - no need to waste resources generating one dynamically
static HELP_TEXT: &str = "\
Simple example for the `gong` argument parser.

USAGE: <prog> [OPTIONS]
       <prog> [OPTIONS] <input> <output>

OPTIONS:
    -h, --help      Outputs this usage info.
    -V, --version   Outputs the version number of this test app.";

// Our program entry point
fn main() {
    // Get our arguments
    let args = Args::new();

    // Setup our parser
    let mut parser = Parser::new(&OPTIONS);
    parser.set_positionals_policy(PositionalsPolicy::Fixed(2));
    parser.settings().set_mode(OptionsMode::Standard)
                     .set_allow_opt_abbreviations(true)
                     .set_stop_on_problem(true);
    debug_assert!(parser.is_valid());

    // Parse our arguments, collecting into a data-mining suitable object
    let analysis = parser.parse(&args);

    // Since if help or version info is requested we actually need to ignore the minimum aspect of
    // our positionals policy, we need to find a way to work around it. Here we will record the
    // number of missing positionals and only output the corresponding error if such an info
    // request has not been made!
    let mut missing_positionals = 0;

    // Handle the first problem, if there is one, and exit
    //
    // We will handle the first problem only, not all of them, since some problems can cause
    // subsequent arguments to be parsed incorrectly. Note that having set the `set_stop_on_problem`
    // setting to `true` above, there will only be one problem captured anyway.
    if let Some(problem) = analysis.get_first_problem() {
        let mut bypass = false;
        match problem {
            ProblemItem::UnexpectedPositional(arg) => {
                eprintln!("Error: Unexpected argument {:?}", arg);
            },
            ProblemItem::MissingPositionals(num) => {
                // See above for why we must do this
                // Also, note that this problem only ever occurs as the last item!
                bypass = true;
                missing_positionals = *num;
            },
            ProblemItem::UnknownLong(opt, _) => {
                eprintln!("Error: Unknown option `{}`", opt.to_string_lossy());
            },
            ProblemItem::UnknownShort(opt) => {
                eprintln!("Error: Unknown short option `{}`", opt);
            },
            ProblemItem::AmbiguousLong(opt) => {
                eprintln!("Error: Ambiguous abbreviated option name `{}`", opt.to_string_lossy());
            },
            ProblemItem::LongWithUnexpectedData(opt, _) => {
                eprintln!("Error: Option `{}` does not take data, but some was provided", opt);
            },
            // There are no more problems applicable to this example!
            ProblemItem::MissingOptionData(_)  |   // No data taking options!
            ProblemItem::AmbiguousCmd(_)       |   // No commands here!
            ProblemItem::UnknownCommand(_, _) => { // No commands here!
                unreachable!();
            },
        }
        if !bypass {
            return;
        }
    }

    // Handle help and version info requests
    //
    // Note, here help takes precedence over version such that even if the flag for version was used
    // first, help is output not version (i.e. in both `--help --version` and `--version --help`
    // help is output not version). If we cared about respecting order, i.e. reacting to which ever
    // came first or which ever came last, then the `get_last_used` method would help with the
    // latter, but nothing exists for the former; you would have to scan the set of items directly.
    if analysis.option_used(HELP_OPT.into()) {
        println!("{}", HELP_TEXT);
        return;
    }
    if analysis.option_used(VERSION_OPT.into()) {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Per above, if an info request has not been made, respect the positionals policy and now
    // output an error if the minimum bound was not met.
    if missing_positionals > 0 {
        eprintln!("Error: Missing {} argument(s). Use `--help` to see usage info.", missing_positionals);
        return;
    }

    // Get our positionals
    //
    // Since we know that we must have the right number, per policy enforcement, we can safely
    // just get by positional index (**not** argument index) and unwrap.
    let input_file = analysis.get_positional(0).unwrap();
    let output_file = analysis.get_positional(1).unwrap();

    // The main functionality of our app here would be to actually do something with the input and
    // output files, of course, but we will just output their names here for the purpose of the
    // example.
    println!("Input: {:?}", input_file);
    println!("Output: {:?}", output_file);
}
