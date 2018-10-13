# <unreleased>

 * Added macros as an alternative means of constructing an "available options" set. This makes
   building an option set without the `add_*` methods easier (crafting the raw struct this way can
   be a more efficient option).
 * Improved the flexibility of the`process` function's "available" args data param. Instead of
   explicitly taking `&[String]`, it now takes `&[T]` where `T: AsRef<str>`, thus supporting both
   `&[String]` and `&[&str]` inputs.
 * Improved crate documentation:
    - Significantly re-edited much of it
    - Split it up into `docs` sub-modules
 * Improved the test suite:
    - Moved almost all of the tests out into the `tests` directory such that they will be run
      outside of the crate as a "user" of the public API, and split apart into multiple files.
    - Replaced the internal test macro that defined the common base set of test options used by most
      tests with a function, which should be more efficient than injecting the option construction
      code into each test function. This function also now uses the new macro construction model
      rather than the `add_*` methods, which avoids the validation they perform, though we still
      (currently) have inefficiency of pushing items onto the `Vec`, as we have no static option set
      capability yet.
 * Implemented `Default` for `Options`
 * Split lib code into multiple files (internal change only, the mods are hidden)
 * Enabled testing for more doc examples
 * Re-ordered copyright & module doc placement

# 1.0.3 (October 17th, 2018)

 * **BUGFIX:** Fixed a bug with correctly analysing "in-same-argument" short option data values.
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
