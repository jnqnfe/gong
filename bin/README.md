gong-playground
===============

A test program for the `gong` "next-gen" getopt replacement for Rust programs.

The point of this test program is simply to provide a means of having an interactive play with the
underlying library's analysis capabilities.

## Usage

To use this test program, firstly ensure that you have the Rust compiler and Cargo installed, then
obtain a copy of the code from the source repository (either clone it or download and extract an
archive); and finally, in a terminal, navigate to the `/bin` sub-directory. You are now ready to
play.

You can either play with the program through `cargo run`, or if you prefer, through building the
program with `cargo build` and then running the program more directly. I will stick to the former
here.

Cargo supports passing command line arguments to your program when you use `cargo run`, but needs to
distinguish between arguments meant for itself and those meant for the to-be-run program. Any
argument that does not look like an option (does not start with a dash) are non-options, and Cargo
passes these on. An argument of exactly two dashes only is special - an *early terminator* - which
Cargo responds to by treating all subsequent arguments as non-options. You can thus use the early
terminator to get Cargo to pass options to the test program.

So, type the following, then add on the end any arguments to give to the test program (after a
space): `cargo run -- `

An example: `cargo run -- --foo --bar abc -- def --ghi`

In this example, the test program will be run with: `--foo --bar abc -- def --ghi`

(Only the first *early terminator* is comsumed by Cargo, subsequent ones are themselves treated as
non-options and thus are passed on).

## Features

The test program has the following features that can be enabled/disabled when compiling:

 - `alt_mode` enables use of `gong`'s "alt mode".
 - `no_abbreviations` disables `gong`'s abbreviated matching feature.
 - `keep_prog_name` avoids skipping over the first ("program name") argument when outputting
    analysis.
 - `color` enables formatted (color/bold/etc) analysis output

To use these features, use Cargo's `features` option. You can enable a single feature, such as
`alt_mode` as `--features alt_mode` or `--features=alt_mode`. You can enable multiple features by
enclosing them in quotes, such as `--features "alt_mode no_abbreviations"` or
`--features="alt_mode no_abbreviations"`. You can enable all features simply with `--all-features`.
You can disable default-enabled features with `--no-default-features`.

Remember, you must supply this Cargo option before supplying the *early terminator* which separates
Cargo's options from those to supply to the to-be-run program.

An example: `cargo run --features alt_mode -- -foo`

This runs the program (re-compiling if necessary) with the `alt_mode` feature, passing in the
arguments: `-foo`.

Have fun!
