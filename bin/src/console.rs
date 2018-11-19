//! Console/terminal text control

// Copyright (c) 2017 Lyndon Brown
//
// Licensed under the MIT license or the Apache license (Version 2.0), at your option. You may not
// copy, modify, or distribute this file except in compliance with said license. You can find copies
// of these licenses either in the LICENSE-MIT and LICENSE-APACHE files, or alternatively at
// <http://opensource.org/licenses/MIT> and <http://www.apache.org/licenses/LICENSE-2.0>
// respectively.

#![allow(dead_code)] // Really this file should be a lib

use libc;

/* Console/terminal text control
 *
 * Not available on Windows (apparently), and should only be used otherwise when stdout is actually
 * a tty.
 *
 * Control sequence pattern is "\u{1B}" (escape char) and "m", with one or more particular numbers
 * separated by ";".
 *
 * Note, when resetting text back to "normal", prefer RESET ("\u{1B}[0m") rather than setting color
 * to black (or white). */

// Control
pub static RESET:   &str = "\u{1B}[0m";
pub static BOLD: &str = "\u{1B}[1m";
pub static DIM: &str = "\u{1B}[2m";
pub static ITALIC: &str = "\u{1B}[3m";
pub static UNDERLINE: &str = "\u{1B}[4m";
pub static REVERSE: &str = "\u{1B}[7m";
pub static INVISIBLE: &str = "\u{1B}[8m";
pub static STRIKE: &str = "\u{1B}[9m"; // strike-through

// Color
pub static BLACK:   &str = "\u{1B}[30m";
pub static RED:     &str = "\u{1B}[31m";
pub static GREEN:   &str = "\u{1B}[32m";
pub static YELLOW:  &str = "\u{1B}[33m";
pub static BLUE:    &str = "\u{1B}[34m";
pub static MAGENTA: &str = "\u{1B}[35m";
pub static CYAN:    &str = "\u{1B}[36m";
pub static WHITE:   &str = "\u{1B}[37m";

pub static BLACK_2:   &str = "\u{1B}[90m";
pub static RED_2:     &str = "\u{1B}[91m";
pub static GREEN_2:   &str = "\u{1B}[92m";
pub static YELLOW_2:  &str = "\u{1B}[93m";
pub static BLUE_2:    &str = "\u{1B}[94m";
pub static MAGENTA_2: &str = "\u{1B}[95m";
pub static CYAN_2:    &str = "\u{1B}[96m";
pub static WHITE_2:   &str = "\u{1B}[97m";

// Bold + color
pub static BLACK_B:   &str = "\u{1B}[30;1m";
pub static RED_B:     &str = "\u{1B}[31;1m";
pub static GREEN_B:   &str = "\u{1B}[32;1m";
pub static YELLOW_B:  &str = "\u{1B}[33;1m";
pub static BLUE_B:    &str = "\u{1B}[34;1m";
pub static MAGENTA_B: &str = "\u{1B}[35;1m";
pub static CYAN_B:    &str = "\u{1B}[36;1m";
pub static WHITE_B:   &str = "\u{1B}[37;1m";

// Text background color highlighting
pub static H_BLACK:   &str = "\u{1B}[40m";
pub static H_RED:     &str = "\u{1B}[41m";
pub static H_GREEN:   &str = "\u{1B}[42m";
pub static H_YELLOW:  &str = "\u{1B}[43m";
pub static H_BLUE:    &str = "\u{1B}[44m";
pub static H_MAGENTA: &str = "\u{1B}[45m";
pub static H_CYAN:    &str = "\u{1B}[46m";
pub static H_WHITE:   &str = "\u{1B}[47m";

pub static H_BLACK_2:   &str = "\u{1B}[100m";
pub static H_RED_2:     &str = "\u{1B}[101m";
pub static H_GREEN_2:   &str = "\u{1B}[102m";
pub static H_YELLOW_2:  &str = "\u{1B}[103m";
pub static H_BLUE_2:    &str = "\u{1B}[104m";
pub static H_MAGENTA_2: &str = "\u{1B}[105m";
pub static H_CYAN_2:    &str = "\u{1B}[106m";
pub static H_WHITE_2:   &str = "\u{1B}[107m";

#[cfg(windows)]
pub fn color_supported() -> bool {
    false
}
#[cfg(not(windows))]
pub fn color_supported() -> bool {
    if unsafe { libc::isatty(libc::STDOUT_FILENO) } == 0 {
        return false;
    }
    true
}
