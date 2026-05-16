//! Strip terminal-escape-bearing control chars from strings before they
//! reach the host terminal.
//!
//! ratatui's `CrosstermBackend` writes adjacent cells via
//! `crossterm::Print(symbol)` without intervening `MoveTo` for runs of
//! same-style cells, so a contiguous run of bytes like
//! `\x1b]52;c;<base64>\x07` in pad content would be concatenated into a
//! single terminal write and interpreted as an OSC 52 "set clipboard"
//! sequence (silent clipboard hijack), or a CSI 2J "clear screen", or a
//! window-title spoof, etc. ratatui itself does not filter control bytes.
//!
//! A malicious co-author on a shared pad could insert these bytes into
//! the pad content and weaponize anyone else's terminal. This helper
//! runs at *render* time only — the rope keeps the original bytes so
//! the OT state stays in lockstep with the server, but anything written
//! into a ratatui `Span` / `Line` / `Paragraph` goes through here first.
//!
//! Allowlist: `\n` (line break) and `\t` (tab). Everything else with
//! `char::is_control() == true` is dropped. That covers:
//!   - C0 controls 0x00-0x1F except \n / \t (incl. ESC 0x1B, BEL 0x07)
//!   - DEL 0x7F
//!   - C1 controls U+0080..U+009F
//!   - other Unicode "control" code points (e.g. U+2028 LINE SEPARATOR)
//!
//! Saved-to-disk files still contain the raw rope bytes — if a user
//! later `cat`s a pad-shared file at a naked shell, the same escape
//! could fire there. That's out of scope for this helper; documented in
//! the README.

/// Strip control chars (preserves `\n` and `\t`) from `s`.
///
/// Returns the input verbatim when nothing needs stripping — the common
/// case for pad content typed by humans — so the allocator is only
/// touched when an attacker (or an unusual paste) actually included
/// control bytes.
pub fn for_terminal(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.chars().any(needs_stripping) {
        return std::borrow::Cow::Borrowed(s);
    }
    std::borrow::Cow::Owned(s.chars().filter(|c| !needs_stripping(*c)).collect())
}

fn needs_stripping(c: char) -> bool {
    if c == '\n' || c == '\t' {
        return false;
    }
    if c.is_control() {
        return true;
    }
    // U+2028 (LINE SEPARATOR) and U+2029 (PARAGRAPH SEPARATOR) are
    // General_Category=Zl/Zp, not Cc, so `is_control()` is false — but
    // they smuggle a logical line break past softwrap and some
    // terminals treat them as cursor-affecting. Drop them too.
    matches!(c, '\u{2028}' | '\u{2029}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_plain_text_through_borrowed() {
        let s = "hello world\nsecond line\ttabbed";
        let out = for_terminal(s);
        assert!(matches!(out, std::borrow::Cow::Borrowed(_)));
        assert_eq!(out, s);
    }

    #[test]
    fn strips_esc() {
        let out = for_terminal("a\x1b[2Jb");
        assert_eq!(out, "a[2Jb");
    }

    #[test]
    fn strips_osc52_clipboard_hijack() {
        // The dangerous one: \x1b]52;c;<b64>\x07 silently overwrites the
        // host clipboard. Both the leading ESC and the terminating BEL
        // must go; otherwise the terminal could still parse a partial
        // sequence.
        let out = for_terminal("\x1b]52;c;Zm9v\x07payload");
        // After strip: leading ESC gone, BEL gone, the inner ASCII
        // remains as harmless text.
        assert_eq!(out, "]52;c;Zm9vpayload");
    }

    #[test]
    fn strips_osc8_hyperlink() {
        let out = for_terminal("\x1b]8;;evil://x\x07click me\x1b]8;;\x07");
        assert_eq!(out, "]8;;evil://xclick me]8;;");
    }

    #[test]
    fn strips_del_and_c1() {
        let out = for_terminal("a\x7fb\u{0085}c\u{009b}d");
        assert_eq!(out, "abcd");
    }

    #[test]
    fn preserves_newlines_and_tabs() {
        let out = for_terminal("col1\tcol2\nrow2\tval");
        assert_eq!(out, "col1\tcol2\nrow2\tval");
    }

    #[test]
    fn strips_unicode_line_separator() {
        // U+2028 LINE SEPARATOR is GC=Zl (not Cc), so is_control() is
        // false — we explicitly drop it to keep softwrap row counts
        // honest.
        let out = for_terminal("foo\u{2028}bar\u{2029}baz");
        assert_eq!(out, "foobarbaz");
    }

    #[test]
    fn empty_in_empty_out() {
        let out = for_terminal("");
        assert_eq!(out, "");
    }
}
