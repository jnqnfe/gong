// Copyright 2017 Lyndon Brown
//
// This file is part of the `gong` command-line argument parsing library.
//
// Licensed under the MIT license or the Apache license (version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

//! Documentation: Unicode & UTF-8 support
//!
//! This is a discussion of text encoding with respect to use of this library.
//!
//! Note that no understanding of Unicode or UTF-8 is actually necessary to use it.
//!
//! # Options and commands
//!
//! > **Note**: Available program *option* and *command* names/`char`s are always expected to be
//! > valid UTF-8.
//!
//! This library has been carefully built to correctly parse UTF-8 encoded strings when parsing
//! arguments. You are thus free to use a wide range of Unicode characters for both [*long options*
//! and *short options*][options], as well as [*commands*][commands].
//!
//! In C programming, it may have been tempting to limit these things to ASCII only, avoiding the
//! richness of Unicode and UTF-8 due to the complexities of handling it. In Rust however, Unicode
//! and UTF-8 are native, and use of them is made pretty trivial. It thus comes naturally that we
//! can feel much more free to make use of the expanded “character” space when specifying
//! “available” *options* and *commands* for our programs.
//!
//! An example consequence of the freedom available to you here is that you could pick something
//! like `💖` (the “sparkling heart” `char`) to be a *short option* for your program, if you
//! really wanted to, without any worries as to the fact that this takes up four bytes in a UTF-8
//! string.
//!
//! Note that just as the ASCII character set included things other than “text characters” (i.e. the
//! “control characters”), Unicode includes not just these but also a wide range of other *code
//! points* that also are not what we traditionally think of as “normal text characters”; there are
//! a whole bunch of “combinators” for instance, which are drawn on top of the previous character,
//! and “selector modifiers”, for instance to change the appearance of an “emoji character”.
//!
//! There is a point of caution to be made with respect to different yet visually identical strings.
//! Note that in some cases it is possible to create a visible “grapheme” (what you might call a
//! “character”) in multiple ways using Unicode. For instance `ö` may look identical to `ö` (they
//! *should* look identical, assuming no bugs in text-rendering ability on your machine), but I
//! assure you that they are in fact different. The first instance here uses a single “character”
//! (*code point*), `U+F6`. The second instance is actually two “characters” (*code points*), a
//! standard Latin `o` (`U+6F`), followed by the special “umlauts” (diaeresis) *combining character*
//! (`U+0308`). Any text-rendering software that understands Unicode knows to draw the *combining
//! character* over the previous character, and thus both should look identical visually. Some
//! software will even go to the trouble of treating them identically (e.g. with respect to deletion
//! and highlighting, your word processor or even text-editor, might treat both as if they *were*
//! single items). As far as parsing arguments with this library is concerned, `--föö` and `--föö`
//! are completely different *options*. No attempt at treating these as the same is made as doing
//! so would introduce a whole heap of complexity and inefficiency. Caution in choice of
//! “characters” used for *options*/*commands* is advised, particularly with respect to ease of
//! users typing a valid program *option*/*command* (“combining characters” would be best avoided to
//! avoid user confusion).
//!
//! With respect to *short options*, a `char` in Rust represents a single Unicode *scalar value*;
//! This library, for the sake of simplicity, allows only a single `char` to represent each *short
//! option*. Thus a `char` paired with one or more special combinator/selector `char`s cannot be
//! used together.
//!
//! To keep things simple and efficient, this library only restricts one or two characters from
//! being used in *long option* names and *command* names, or as *short option* characters, where
//! use of those characters would cause problems correctly parsing arguments. You are thus left
//! largely free to use pretty much whatever “characters” you like, though applying some common
//! sense to your choices is advised.
//!
//! Note that per the above, whilst `-ö` (using `U+F6`) results in a single matched/unmatched *short
//! option*, `-ö` (using `U+6F` followed by the `U+0308` combinator) will result in two. As another
//! example, `❤` is the “black heart” character, and `❤️` is it along with the `U+FE0F` “variant
//! #16 - emoji” selector `char`; with the selector, `--❤️` is a single matched/unmatched *long
//! option*, while `-❤️` is a pair of matched/unmatched *short options*, one for the “black heart”
//! `char` and one for the selector `char`.
//!
//! [commands]: ../commands/index.html
//! [options]: ../options/index.html
