# <unreleased>

 * Improved efficiency of `is_valid` for option sets
 * Fixed incorrect order of license filenames mentioned in header blocks

# 1.3.0 (October 29th, 2018)

**Behaviour change:** This release contains an insignificant behaviour change with respect to
option set validation issues - details are no longer output on `stderr`.

 * Added `validate` method to `OptionSet` and `OptionSetEx` as an alternative to `is_valid`,
   returning details of any flaws.
 * Changed the `is_valid` method on `OptionSet` and `OptionSetEx` to no longer output details of
   problems to `stderr`, preferring details to now be obtained via the new `validate` method. The
   `is_valid` method was originally designed with the idea that it should realistically only ever
   return `false` in a debug build during development, and as such, unusually it was allowed to
   output descriptions of problems encountered to `stderr`. That decision has now been revised.
 * Added methods to `OptionSetEx` that add ready-made `LongOption`s/`ShortOption`s
 * Improved internal organisation of the `OptionSet`/`OptionSetEx` validation code
 * Improved organisation of the test suite’s `options` file

# 1.2.1 (October 28th, 2018)

 * Updated usage docs per `OptionSetEx`/`OptionSet` pair introduction
 * Improved macro set construction testing
 * Extended the option set type equality testing to try a not-equal example
 * Cleaned up bits of the “preparation” usage documentation
 * Updated crate description in toml file, overlooked in v1.1

# 1.2.0 (October 27th, 2018)

 * Refactored and optimised the analysis engine
 * Renamed `Options` to `OptionSetEx` and introduced companion `OptionSet`. While the `OptionSetEx`
   type uses `Vec` to hold lists of options (the `Ex` in the new name refers to “extendible”), the
   new `OptionSet` holds slice references, and thus can be used for situations where dynamic
   “builder” style construction is not necessary, and allows creation of static option sets, which
   improves efficiency in many cases. Use of the old `Options` type name is now deprecated.
 * Added the `gong_option_set_fixed` macro. While the existing `gong_option_set` macro produces
   an `OptionSetEx`, this new macro produces an `OptionSet`.
 * Added a `process` method to `Options` (now `OptionSetEx`) as a cleaner alternative to calling the
   `process` function directly (also available on `OptionSet`), and deprecated direct use of the
   `process` function.
 * Updated the test suite to use a static `OptionSet` for the base test set, for greater efficiency

# 1.1.2 (October 26th, 2018)

 * **BUGFIX:** Fixed a bug with abbreviated match ambiguity
 * Added test to check processing function’s expanded support of both `&[String]` and `&[&str]`
   argument list types.
 * Switched to `&str` based argument lists for most tests now for greater efficiency

# 1.1.1 (October 20th, 2018)

 * Fixed some broken links in documentation
 * Minor tweaks not worth documenting

# 1.1.0 (October 19th, 2018)

 * Split lib code into multiple modules. Everything is still available at the top level for now for
   compatibility. It is recommended to access things from the sub-modules from now on though, as
   this will be required in a future version.
 * Added macros as an alternative means of constructing an “available options” set. This makes
   building an option set without the `add_*` methods easier (crafting the raw struct this way can
   be a more efficient option).
 * Improved the flexibility of the`process` function’s “available” args data param. Instead of
   explicitly taking `&[String]`, it now takes `&[T]` where `T: AsRef<str>`, thus supporting both
   `&[String]` and `&[&str]` inputs.
 * Renamed `Results` to `Analysis`. A type alias of `Results` is provided, marked as deprecated, for
   compatibility purposes, and to encourage updating to the new name.
 * Improved crate documentation:
    - Significantly re-edited much of it
    - Split it up into `docs` sub-modules
 * Improved the test suite:
    - Expanded the suite with an even broader set of tests.
    - Moved almost all of the tests out into the `tests` directory such that they will be run
      outside of the crate as a “user” of the public API, and split up the single file containing
      tests into multiple files.
    - Replaced the internal test macro that defined the common base set of test options used by most
      tests with a function, which should be more efficient than injecting the option construction
      code into each test function. This function also now uses the new macro construction model
      rather than the `add_*` methods, which avoids the validation they perform, though we still
      (currently) have inefficiency of pushing items onto the `Vec`, as we have no static option set
      capability yet.
    - Tidied up the code significantly.
 * Implemented `Default` for `Options`
 * Enabled testing for more doc examples
 * Re-ordered copyright & module doc placement

# 1.0.3 (October 17th, 2018)

 * **BUGFIX:** Fixed a bug with correctly analysing “in-same-argument” short option data values
 * Minor code simplification
 * Fixed a doc typo

# 1.0.2 (October 9th, 2018)

 * Added homepage and repo links to toml file
 * Bundled the changelog file into the published package
 * Bundled the license files into the published package

# 1.0.1 (February 1st, 2018)

 * Fixed toml file missing author email address

# 1.0 (January 24th, 2018)

 * Original release
