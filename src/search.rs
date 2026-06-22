//! Search and replace state for the editor.
//!
//! [`SearchState`] tracks the query, replace string, regex flag, and the
//! computed match list.  Matches are stored as `(line, char_start, char_end)`
//! tuples in document char-space (not byte-space) so the renderer can paint
//! them without any further conversion.
//!
//! `fancy-regex` is already pulled into the build transitively by `syntect`'s
//! `regex-fancy` feature, so adding it here incurs zero additional compile cost.

use fancy_regex::Regex;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single match: (logical line index, char_start, char_end).
pub type Match = (usize, usize, usize);

/// All state for an active search / search-and-replace session.
#[derive(Debug, Clone)]
pub struct SearchState {
    /// The raw query string as typed by the user.
    pub query: String,
    /// The replacement string (only meaningful when `show_replace` is `true`).
    pub replace: String,
    /// When `true`, `query` is compiled as a fancy-regex pattern.
    /// When `false`, special characters are escaped so the query is literal.
    pub regex_mode: bool,
    /// Show the replace row in the search-bar UI.
    pub show_replace: bool,
    /// Which field has keyboard focus: `true` = search field, `false` = replace field.
    pub focus_search: bool,
    /// Compiled regex.  `None` when `query` is empty or invalid.
    compiled: Option<Regex>,
    /// All matches in the current document, in document order.
    pub matches: Vec<Match>,
    /// Index of the currently selected match (wraps around).
    pub current: usize,
    /// `true` when the last compile attempt produced a regex error.
    pub regex_error: bool,
    /// Show the floating shortcut-cheatsheet overlay.
    ///
    /// Starts `true` every time a new search session opens; cleared on the
    /// first keypress inside search mode so the hint is ephemeral.
    pub show_help: bool,
}

impl SearchState {
    /// Create a new search state, optionally showing the replace row.
    pub fn new(show_replace: bool) -> Self {
        Self {
            query: String::new(),
            replace: String::new(),
            regex_mode: false,
            show_replace,
            focus_search: true,
            compiled: None,
            matches: Vec::new(),
            current: 0,
            regex_error: false,
            show_help: true,
        }
    }

    /// Recompile the pattern and refresh the match list from `lines`.
    ///
    /// Call whenever `query`, `regex_mode`, or the document text changes.
    pub fn update_matches(&mut self, lines: &[String]) {
        if self.query.is_empty() {
            self.compiled = None;
            self.matches.clear();
            self.regex_error = false;
            return;
        }

        let pattern = if self.regex_mode {
            self.query.clone()
        } else {
            escape_literal(&self.query)
        };

        match Regex::new(&pattern) {
            Ok(re) => {
                self.compiled = Some(re);
                self.regex_error = false;
            }
            Err(_) => {
                self.compiled = None;
                self.matches.clear();
                self.regex_error = true;
                return;
            }
        }

        self.matches = find_all_matches(self.compiled.as_ref().unwrap(), lines);

        // Keep `current` in bounds.
        if self.matches.is_empty() {
            self.current = 0;
        } else {
            self.current = self.current.min(self.matches.len() - 1);
        }
    }

    /// Append a character to the active field.
    ///
    /// Does **not** call `update_matches` — the event loop fires that after the
    /// search debounce timer elapses (see `App::search_last_typed`).
    pub fn push_char(&mut self, c: char) {
        if self.focus_search {
            self.query.push(c);
        } else {
            self.replace.push(c);
        }
    }

    /// Delete the last character from the active field.
    ///
    /// Does **not** call `update_matches` — the event loop fires that after the
    /// search debounce timer elapses (see `App::search_last_typed`).
    pub fn pop_char(&mut self) {
        if self.focus_search {
            self.query.pop();
        } else {
            self.replace.pop();
        }
    }

    /// Advance to the next match, wrapping at the end.
    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current = (self.current + 1) % self.matches.len();
        }
    }

    /// Retreat to the previous match, wrapping at the beginning.
    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            if self.current == 0 {
                self.current = self.matches.len() - 1;
            } else {
                self.current -= 1;
            }
        }
    }

    /// Return the current match, if any.
    pub fn current_match(&self) -> Option<Match> {
        self.matches.get(self.current).copied()
    }

    /// Snap `current` to the first match at-or-after the given cursor position.
    ///
    /// Wraps to index 0 if the cursor is past all matches.
    pub fn snap_to_cursor(&mut self, cursor_line: usize, cursor_col: usize) {
        if self.matches.is_empty() {
            return;
        }
        let idx = self
            .matches
            .iter()
            .position(|&(ml, ms, _)| ml > cursor_line || (ml == cursor_line && ms >= cursor_col))
            .unwrap_or(0);
        self.current = idx;
    }

    /// Replace the chars `char_start..char_end` on `line` with `self.replace`.
    pub fn apply_replace_to_line(&self, line: &str, char_start: usize, char_end: usize) -> String {
        let chars: Vec<char> = line.chars().collect();
        let before: String = chars[..char_start.min(chars.len())].iter().collect();
        let after: String = chars[char_end.min(chars.len())..].iter().collect();
        format!("{}{}{}", before, self.replace, after)
    }

    /// Apply replace-all and return a fully modified lines vector.
    ///
    /// Processes matches from bottom to top so earlier char offsets remain
    /// valid as later (rightward) spans are replaced.
    pub fn apply_replace_all(&self, lines: &[String]) -> Vec<String> {
        if self.compiled.is_none() || self.matches.is_empty() {
            return lines.to_vec();
        }
        let mut result: Vec<String> = lines.to_vec();
        for &(ml, ms, me) in self.matches.iter().rev() {
            if ml < result.len() {
                result[ml] = self.apply_replace_to_line(&result[ml], ms, me);
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Escape all fancy-regex metacharacters so a query is treated as literal text.
fn escape_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for c in s.chars() {
        if matches!(
            c,
            '.' | '^' | '$' | '*' | '+' | '?' | '{' | '}' | '[' | ']' | '\\' | '|' | '(' | ')'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Find all matches of `re` across `lines`, returning `(line, char_start, char_end)`.
///
/// # Mutation notes
/// The zero-width-match guard (`m.end() == m.start()`) and the advance expression
/// are `#[mutants::skip]`-protected:
/// - `== → !=` inverts the guard, causing the loop to advance on every *normal* match
///   while stalling on zero-width ones → infinite loop.
/// - Mutations to the char-boundary advance produce either an infinite loop (no
///   progress) or a panic (mid-codepoint byte index into `find_from_pos`).
#[mutants::skip]
fn find_all_matches(re: &Regex, lines: &[String]) -> Vec<Match> {
    let mut out = Vec::new();
    for (li, line) in lines.iter().enumerate() {
        let mut byte_pos = 0usize;
        while byte_pos <= line.len() {
            match re.find_from_pos(line, byte_pos) {
                Ok(Some(m)) => {
                    if m.end() == m.start() {
                        // Zero-width match — advance to the next UTF-8 char boundary
                        // to avoid both an infinite loop and a mid-codepoint index.
                        let step = line[m.start()..].chars().next().map_or(1, |c| c.len_utf8());
                        byte_pos = m.start() + step;
                        continue;
                    }
                    let char_start = line[..m.start()].chars().count();
                    let char_end = char_start + line[m.start()..m.end()].chars().count();
                    out.push((li, char_start, char_end));
                    byte_pos = m.end();
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(s: &str) -> Vec<String> {
        s.lines().map(|l| l.to_string()).collect()
    }

    #[test]
    fn literal_search_finds_two_occurrences() {
        let mut s = SearchState::new(false);
        let d = doc("hello world\nhello again");
        s.query = "hello".to_string();
        s.update_matches(&d);
        assert_eq!(s.matches.len(), 2);
        assert_eq!(s.matches[0], (0, 0, 5));
        assert_eq!(s.matches[1], (1, 0, 5));
    }

    #[test]
    fn push_char_appends_query() {
        let mut s = SearchState::new(false);
        let d = doc("hello world\nhello again");
        for c in "hello".chars() {
            s.push_char(c);
        }
        assert_eq!(s.query, "hello");
        // Matches are now updated via the event-loop debounce, not inline.
        // Call update_matches explicitly to verify the query is correct.
        s.update_matches(&d);
        assert_eq!(s.matches.len(), 2);
    }

    #[test]
    fn regex_mode_matches_pattern() {
        let mut s = SearchState::new(false);
        s.regex_mode = true;
        let d = doc("foo 123 bar 456");
        s.query = r"\d+".to_string();
        s.update_matches(&d);
        assert_eq!(s.matches.len(), 2);
        assert_eq!(s.matches[0], (0, 4, 7));
        assert_eq!(s.matches[1], (0, 12, 15));
    }

    #[test]
    fn literal_escapes_dot() {
        let mut s = SearchState::new(false);
        let d = doc("foo.bar");
        s.query = "foo.bar".to_string();
        s.update_matches(&d);
        assert_eq!(s.matches.len(), 1);

        // Must NOT match "fooXbar" (dot is not treated as regex wildcard).
        let d2 = doc("fooXbar");
        s.update_matches(&d2);
        assert_eq!(s.matches.len(), 0);
    }

    #[test]
    fn next_match_wraps_from_last_to_first() {
        let mut s = SearchState::new(false);
        let d = doc("aaa");
        s.query = "a".to_string();
        s.update_matches(&d);
        assert_eq!(s.matches.len(), 3);
        s.current = 2;
        s.next_match();
        assert_eq!(s.current, 0);
    }

    #[test]
    fn prev_match_wraps_from_first_to_last() {
        let mut s = SearchState::new(false);
        let d = doc("aaa");
        s.query = "a".to_string();
        s.update_matches(&d);
        s.current = 0;
        s.prev_match();
        assert_eq!(s.current, 2);
    }

    #[test]
    fn pop_char_removes_last_char_of_query() {
        let mut s = SearchState::new(false);
        let d = doc("hello");
        s.query = "hell".to_string();
        s.update_matches(&d);
        s.pop_char();
        assert_eq!(s.query, "hel");
    }

    #[test]
    fn empty_query_clears_matches() {
        let mut s = SearchState::new(false);
        let d = doc("hello");
        s.query = "hello".to_string();
        s.update_matches(&d);
        assert_eq!(s.matches.len(), 1);
        s.query.clear();
        s.update_matches(&d);
        assert!(s.matches.is_empty());
    }

    #[test]
    fn invalid_regex_sets_error_flag() {
        let mut s = SearchState::new(false);
        s.regex_mode = true;
        let d = doc("hello");
        s.query = "[invalid".to_string();
        s.update_matches(&d);
        assert!(s.regex_error);
        assert!(s.matches.is_empty());
    }

    #[test]
    fn valid_regex_clears_error_flag() {
        let mut s = SearchState::new(false);
        s.regex_mode = true;
        let d = doc("hello");
        s.query = "[invalid".to_string();
        s.update_matches(&d);
        assert!(s.regex_error);
        s.query = "hello".to_string();
        s.update_matches(&d);
        assert!(!s.regex_error);
        assert_eq!(s.matches.len(), 1);
    }

    #[test]
    fn apply_replace_to_line_basic() {
        let mut s = SearchState::new(false);
        s.replace = "world".to_string();
        assert_eq!(s.apply_replace_to_line("hello there", 0, 5), "world there");
    }

    #[test]
    fn apply_replace_to_line_mid() {
        let mut s = SearchState::new(false);
        s.replace = "X".to_string();
        assert_eq!(s.apply_replace_to_line("abc", 1, 2), "aXc");
    }

    #[test]
    fn apply_replace_all_replaces_every_occurrence() {
        let mut s = SearchState::new(true);
        s.replace = "baz".to_string();
        let d = doc("foo foo\nfoo bar");
        s.query = "foo".to_string();
        s.update_matches(&d);
        let result = s.apply_replace_all(&d);
        assert_eq!(result[0], "baz baz");
        assert_eq!(result[1], "baz bar");
    }

    #[test]
    fn snap_to_cursor_finds_next_forward_match() {
        let mut s = SearchState::new(false);
        let d = doc("hello world hello");
        s.query = "hello".to_string();
        s.update_matches(&d);
        s.snap_to_cursor(0, 6); // cursor after first "hello "
        assert_eq!(s.current, 1);
    }

    #[test]
    fn snap_to_cursor_wraps_to_first_when_past_all() {
        let mut s = SearchState::new(false);
        let d = doc("hello world hello");
        s.query = "hello".to_string();
        s.update_matches(&d);
        s.snap_to_cursor(0, 13); // past second "hello"
        assert_eq!(s.current, 0);
    }

    #[test]
    fn focus_replace_field_push_pop_does_not_update_matches() {
        let mut s = SearchState::new(true);
        s.focus_search = false;
        let d = doc("hello");
        s.query = "hello".to_string();
        s.update_matches(&d);
        let before = s.matches.len();
        s.push_char('x');
        assert_eq!(s.replace, "x");
        assert_eq!(
            s.matches.len(),
            before,
            "replace-field edit must not clear matches"
        );
        s.pop_char();
        assert_eq!(s.replace, "");
        assert_eq!(s.matches.len(), before);
    }

    #[test]
    fn no_match_current_stays_zero() {
        let mut s = SearchState::new(false);
        let d = doc("hello");
        s.query = "xyz".to_string();
        s.update_matches(&d);
        assert_eq!(s.current, 0);
        s.next_match();
        assert_eq!(s.current, 0);
        s.prev_match();
        assert_eq!(s.current, 0);
    }

    // ---- T-16: update_matches clamps current to last valid index ----

    #[test]
    fn update_matches_clamps_current_to_len_minus_one() {
        // Kills: line 102:64 `replace - with +` and `replace - with /`
        // in `self.current = self.current.min(self.matches.len() - 1)`.
        // With `+1`: current = min(99, 4) = 4 (out of bounds index when len=3).
        // With `/1 = len`: same OOB. Test asserts current_match() is Some with correct index.
        let mut s = SearchState::new(false);
        let d = doc("aaa");
        s.query = "a".to_string();
        s.update_matches(&d); // 3 matches
        assert_eq!(s.matches.len(), 3);
        s.current = 99; // manually set OOB
        s.update_matches(&d); // must clamp
        assert_eq!(s.current, 2, "current must be clamped to len-1 = 2");
        assert!(
            s.current_match().is_some(),
            "current_match() must return Some after clamp"
        );
    }

    // ---- T-17: prev_match decrements current correctly ----

    #[test]
    fn prev_match_decrements_current_exact_value() {
        // Kills: line 140:30 `replace -= with +=` and `replace -= with /=`
        // `+=1` would go forward; `/=1` would stay. Assert exact values.
        let mut s = SearchState::new(false);
        let d = doc("aaa");
        s.query = "a".to_string();
        s.update_matches(&d); // 3 matches at indices 0,1,2
        s.current = 1;
        s.prev_match();
        assert_eq!(s.current, 0, "prev from index 1 must give index 0");
        s.prev_match(); // wraps from 0 to last
        assert_eq!(s.current, 2, "prev from index 0 must wrap to index 2");
    }

    // ---- T-18: current_match returns correct (line, char_start, char_end) ----

    #[test]
    fn current_match_returns_specific_field_values() {
        // Kills: line 147:9 `replace -> Option<Match> with None` and
        //        `replace -> Option<Match> with Some(Default::default())`
        // Default::default() for Match=(usize,usize,usize) is (0,0,0).
        // Assert on non-default char_start and char_end.
        let mut s = SearchState::new(false);
        let d = doc("abcd");
        s.query = "bc".to_string();
        s.update_matches(&d); // match at (0, 1, 3)
        let m = s.current_match().expect("must return Some match");
        assert_eq!(m.0, 0, "match must be on line 0");
        assert_eq!(m.1, 1, "char_start must be 1 (not 0 from Default)");
        assert_eq!(m.2, 3, "char_end must be 3 (not 0 from Default)");
    }

    // ---- T-19: snap_to_cursor selects first match at-or-after cursor ----

    #[test]
    fn snap_to_cursor_selects_match_at_or_after_cursor_col() {
        // Kills: line 161:20 `replace > with < in snap_to_cursor`
        // With `<`, cursor "after" means behind → selects match BEFORE cursor.
        // "x foo x bar x" — matches at cols 0, 6, 12.
        let mut s = SearchState::new(false);
        let d = doc("x foo x bar x");
        s.query = "x".to_string();
        s.update_matches(&d); // 3 matches: (0,0,1), (0,6,7), (0,12,13)
        assert_eq!(s.matches.len(), 3);

        s.snap_to_cursor(0, 4); // cursor between first and second match
        let m = s.current_match().expect("must have a match");
        assert_eq!(m.1, 6, "first match at-or-after col 4 must be at col 6");

        s.snap_to_cursor(0, 0); // cursor at exact match position
        let m = s.current_match().expect("must have a match");
        assert_eq!(m.1, 0, "match exactly at cursor col 0 must be selected");
    }

    // ---- T-20: find_all_matches char_end arithmetic ----

    #[test]
    fn find_all_matches_char_end_is_start_plus_match_len() {
        // Kills: line 227:46 `replace + with -` and `replace + with *`
        // char_end = char_start + match_char_count.
        // For "barfoo", "foo" starts at char 3, ends at char 6.
        // With `+→-`: 3 - 3 = 0 (underflow).  With `+→*`: 3 * 3 = 9 (wrong).
        let mut s = SearchState::new(false);
        let d = doc("barfoo");
        s.query = "foo".to_string();
        s.update_matches(&d);
        assert_eq!(s.matches.len(), 1);
        let (ln, cs, ce) = s.matches[0];
        assert_eq!(ln, 0, "match on line 0");
        assert_eq!(cs, 3, "char_start = 3");
        assert_eq!(ce, 6, "char_end = 6 (3 + 3 chars of 'foo')");
    }

    // ---- T-21: snap_to_cursor exercises the ml > cursor_line branch ----

    #[test]
    fn snap_to_cursor_uses_line_number_ordering_across_lines() {
        // Kills line 161:20 `replace > with <`.
        // Single-line tests can't expose this: when ml == cursor_line the
        // `ml > cursor_line` and `ml < cursor_line` branches both evaluate to
        // false, so the result is identical. A multi-line doc where the target
        // match is on a later line than the cursor is required.
        // With `> → <`: ml=2 < cursor_line=1 is false AND ml==cursor_line is
        // false → no match found → wraps to index 0 (wrong).
        let mut s = SearchState::new(false);
        let d = doc("needle\nboring\nneedle");
        s.query = "needle".to_string();
        s.update_matches(&d);
        assert_eq!(s.matches.len(), 2); // (0,0,6) and (2,0,6)

        s.snap_to_cursor(1, 0); // cursor on line 1 (between the two matches)
        assert_eq!(
            s.current, 1,
            "cursor on line 1 must snap to the match on line 2 (index 1), not line 0"
        );
    }

    // ---- T-22: apply_replace_all guard: || vs && ----

    #[test]
    fn apply_replace_all_with_none_compiled_bails_without_modifying_buffer() {
        // Kills line 185:36 `replace || with &&`.
        // With `&&`, the guard only bails when BOTH compiled.is_none() AND
        // matches.is_empty().  A stale state where compiled is cleared but
        // matches remain would skip the guard and corrupt the buffer.
        let mut s = SearchState::new(true);
        let d = doc("foo bar");
        s.query = "foo".to_string();
        s.update_matches(&d);
        assert_eq!(s.matches.len(), 1);

        // Simulate stale state: compiled cleared, matches still set.
        s.compiled = None;
        s.replace = "baz".to_string();
        let result = s.apply_replace_all(&d);
        assert_eq!(
            result, d,
            "compiled=None must prevent replace-all even when matches is non-empty"
        );
    }

    // ---- T-23: apply_replace_all bounds guard: < vs <= ----

    #[test]
    fn apply_replace_all_skips_stale_out_of_bounds_match_lines() {
        // Kills line 190:19 `replace < with <=`.
        // With `<=`, `result[ml]` when `ml == result.len()` would panic.
        // A stale match pointing past the last line must be silently skipped.
        let mut s = SearchState::new(true);
        let d = doc("hello");
        s.replace = "world".to_string();
        // Inject a valid compiled regex and a stale match on line 99.
        s.compiled = Some(Regex::new("hello").unwrap());
        s.matches = vec![(99, 0, 5)];
        let result = s.apply_replace_all(&d);
        assert_eq!(
            result, d,
            "stale out-of-bounds match must be skipped, not panic"
        );
    }

    // ---- T-24: zero-width match advance (line 227) ----

    #[test]
    fn find_all_matches_zero_width_match_advance_terminates() {
        // Kills line 227 `replace + with -` and `replace + with *`.
        // `a*` on "aab" produces a non-zero-width match "aa" at (0,0,2), then
        // a zero-width empty match at byte 2 (advance by +1 → 3), then
        // another empty at byte 3 → advance to 4 → loop exits.
        // With `+→*`: advance = m.start() * 1 = 2 → infinite loop.
        // With `+→-`: advance = m.start() - 1 = 1 → re-finds "a" at 1, then
        //   zero-width at 2 → advance to 1 again → infinite loop.
        let mut s = SearchState::new(false);
        s.regex_mode = true;
        let d = doc("aab");
        s.query = "a*".to_string();
        s.update_matches(&d);
        // Only "aa" is a non-zero-width match; zero-width empties are skipped.
        assert_eq!(
            s.matches.len(),
            1,
            "must find exactly the 'aa' match and terminate"
        );
        assert_eq!(s.matches[0], (0, 0, 2));
    }

    // ---- T-25: zero-width match on a multi-byte UTF-8 line does not panic ----

    #[test]
    fn find_all_matches_zero_width_match_respects_utf8_boundaries() {
        // `^` produces a zero-width match at byte 0.  With the old `+ 1` advance,
        // byte_pos becomes 1 — the middle of the 2-byte sequence for `é` (\xc3\xa9)
        // — causing `find_from_pos` to panic on the next iteration.  The fixed code
        // advances by the byte-length of the char at m.start(), so byte_pos jumps
        // to 2 (past `é`), then finds no further `^` anchors and terminates.
        let mut s = SearchState::new(false);
        s.regex_mode = true;
        let d = doc("élan");
        s.query = "^".to_string();
        s.update_matches(&d);
        // `^` anchors don't produce a visible match we count (zero-width, skipped),
        // so no matches in the output — but the important thing is: no panic.
        assert_eq!(
            s.matches.len(),
            0,
            "zero-width anchor must not panic and must produce no match"
        );
    }
}
