Overview
========

[<img alt="travis.com" src="https://img.shields.io/travis/com/jnqnfe/gong?style=for-the-badge" height="24">](https://travis-ci.com/jnqnfe/gong)
<img alt="license" src="https://img.shields.io/crates/l/gong.svg?style=for-the-badge" height="24">

This repository contains `gong`, a lightweight, flexible and simple-to-use library provided to
assist in parsing command line arguments in Rust based programs.

You can help fund my work through one of the following platforms: [patreon][patreon],
[liberapay][liberapay], or [buy-me-a-coffee][buymeacoffee].

[patreon]: https://www.patreon.com/jnqnfe
[liberapay]: https://liberapay.com/jnqnfe/
[buymeacoffee]: https://www.buymeacoffee.com/jnqnfe

Copyright & Licensing
=====================

All files in this source code repository, except as noted below, are licensed under the MIT license
or the Apache license, Version 2.0, at your option. You can find copies of these licenses either in
the `LICENSE-MIT` and `LICENSE-APACHE` files, or alternatively [here][1] and [here][2] respectively.

[1]: http://opensource.org/licenses/MIT
[2]: http://www.apache.org/licenses/LICENSE-2.0

The `bin` directory (currently) contains only a tiny test program that was quickly thrown together
for interactively playing with the library. This program is technically covered by the same licenses
as the library, however there is extremely little to it, and really it can be considered fair-use to
re-use pieces of it for the purposes of writing code that uses the library without worrying about
such stuff. (It can be considered an example of basic usage).

The logo image files are derivatives of the Rust programming language icon. I apply no specific
image-oriented license upon them (I am not familiar with such licenses). As a substitute, subject to
any constraints of licensing of the Rust logo image, I freely permit use on a common-sense fair-use
basis. Feel free to make your own such derived logos, I make no claim upon it being an original
idea.

Source Code Contents
====================

 - lib/          - The library itself
 - bin/          - An interactive “playground” program for testing out the library’s functionality
