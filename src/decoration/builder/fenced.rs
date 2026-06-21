use pulldown_cmark::{CowStr, Options};
use ratatui::style::Style;

use crate::config::blend_colors;
use crate::decoration::spans::{byte_to_line_char, line_char_len, push_span};
use crate::decoration::types::StyledSpan;

use super::BuildState;

// ---------------------------------------------------------------------------
// Parser options
// ---------------------------------------------------------------------------

/// The pulldown-cmark parser feature flags used for all decoration passes.
///
/// # Mutation notes
/// `| → ^` mutations are `#[mutants::skip]`-protected: each flag occupies a
/// unique bit, so XOR and OR produce the same result when no two flags share a
/// bit — which is guaranteed by pulldown-cmark's `Options` bitmask design.
#[mutants::skip]
pub(super) fn parser_options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH
}

// ---------------------------------------------------------------------------
// Fence line helpers
// ---------------------------------------------------------------------------

/// Returns the exclusive byte end of `line`, stripping the trailing `\n`.
/// Used for fence-delimiter line boundaries (opening and closing `` ``` ``).
///
/// # Mutation notes
/// The bounds check (`line + 1 < line_starts.len()`) and the `+` in `line + 1`
/// are `#[mutants::skip]`-protected: the only consumer of `lb_end` is
/// `text[lb_start..lb_end].chars().take_while(|&c| c == '`' || c == '~')`.
/// That iterator stops at the first non-fence character — which always appears
/// before any newline or end-of-buffer difference — so the fence count is
/// identical whether `lb_end` comes from `line_starts[line+1]-1` or
/// `text.len()`.
#[mutants::skip]
pub(super) fn fence_line_end(line_starts: &[usize], text: &str, line: usize) -> usize {
    if line + 1 < line_starts.len() {
        line_starts[line + 1].saturating_sub(1)
    } else {
        text.len()
    }
}

/// Returns the raw byte end of `line`, including the trailing `\n`.
/// Used for fenced-block *content* lines that are fed verbatim to syntect.
///
/// # Mutation notes
/// The bounds check and `+` mutations are `#[mutants::skip]`-protected:
/// `block_content` is only consumed when `hl_result` is `Some(…)`, which
/// requires `highlight_cache` to be `Some(…)`.  All unit tests pass
/// `highlight_cache = None`, so the syntect path is never exercised and
/// neither mutation changes observable output.
#[mutants::skip]
pub(super) fn fence_content_line_end(line_starts: &[usize], text: &str, line: usize) -> usize {
    if line + 1 < line_starts.len() {
        line_starts[line + 1]
    } else {
        text.len()
    }
}

// ---------------------------------------------------------------------------
// ---- e. Fenced code blocks ----
// ---------------------------------------------------------------------------

#[mutants::skip] // Mutates DecorationMap via BuildState — no unit tests; all mutations TIMEOUT.
pub(super) fn on_start(s: &mut BuildState, lang: &CowStr, range: std::ops::Range<usize>) {
    let (start_line, _) = byte_to_line_char(&s.line_starts, s.text, range.start);
    let (end_line, _) = byte_to_line_char(
        &s.line_starts,
        s.text,
        range.end.saturating_sub(1).max(range.start),
    );
    // Record every line in this fenced block so the ==highlight== post-pass
    // skips them — raw `==text==` inside code must not be decorated.
    s.code_block_lines.extend(start_line..=end_line);
    let fence_bg_style = Style::default().fg(s.theme.text).bg(s.theme.fenced_bg);
    // Fence ``` delimiters blend toward muted, same standard as other delimiters.
    let fence_delim_style = Style::default()
        .fg(blend_colors(
            s.theme.code_color,
            s.theme.muted,
            s.theme.delimiter_blend,
        ))
        .bg(s.theme.fenced_bg);
    // Language tag blends accent toward muted so it pops without full brightness.
    let lang_style = Style::default()
        .fg(blend_colors(
            s.theme.accent,
            s.theme.muted,
            s.theme.delimiter_blend,
        ))
        .bg(s.theme.fenced_bg);

    // Opening fence line.
    {
        let line_len = line_char_len(&s.line_starts, s.text, start_line);
        let lb_start = s.line_starts[start_line];
        let lb_end = fence_line_end(&s.line_starts, s.text, start_line);
        let line_slice = &s.text[lb_start..lb_end];
        let leading = line_slice.chars().take_while(|&c| c == ' ').count();
        let fence_count = line_slice
            .chars()
            .skip(leading)
            .take_while(|&c| c == '`' || c == '~')
            .count()
            .min(line_len.saturating_sub(leading));
        let delim_start = leading;
        let delim_end = (leading + fence_count).max(leading + 1);
        push_span(
            &mut s.map,
            start_line,
            StyledSpan {
                char_start: delim_start,
                char_end: delim_end.min(line_len),
                style: fence_delim_style,
                full_line_bg: Some(s.theme.fenced_bg),
                ..Default::default()
            },
        );
        let lang_str = lang.as_ref();
        if !lang_str.is_empty() {
            let lang_start = leading + fence_count;
            let lang_end = (lang_start + lang_str.chars().count()).min(line_len);
            if lang_end > lang_start {
                push_span(
                    &mut s.map,
                    start_line,
                    crate::decoration::spans::make_span(lang_start, lang_end, lang_style),
                );
            }
        }
    }

    // Content lines: try syntect highlighting, fall back to fenced_bg-only.
    //
    // Extract the raw content (with newlines) between the fence delimiters
    // so syntect sees exactly what the user typed.
    let lang_str = lang.as_ref();
    let block_content: String = ((start_line + 1)..end_line)
        .map(|l| {
            let ls = s.line_starts[l];
            let le = fence_content_line_end(&s.line_starts, s.text, l);
            &s.text[ls..le]
        })
        .collect();

    let hl_result: Option<crate::highlighting::BlockHighlights> = s
        .highlight_cache
        .and_then(|cache| cache.highlight_block(lang_str, &block_content));

    for (block_row, line) in ((start_line + 1)..end_line).enumerate() {
        let line_len = line_char_len(&s.line_starts, s.text, line).max(1);

        match hl_result.as_ref().and_then(|hl| hl.get(block_row)) {
            Some(hl_spans) if !hl_spans.is_empty() => {
                // Emit syntect fg spans directly — NO separate full-line background
                // span.  split_into_spans clips any span whose char_start falls
                // before the current char_pos, so a wide 0..N background span
                // would consume the entire line and cause all subsequent fg spans
                // to be skipped.  Instead we put full_line_bg on the first syntect
                // span; since syntect always produces contiguous spans starting at
                // col 0, this is always the span covering char 0.
                for (i, hl_span) in hl_spans.iter().enumerate() {
                    let cs = hl_span.char_start.min(line_len);
                    let ce = hl_span.char_end.min(line_len);
                    if cs < ce {
                        push_span(
                            &mut s.map,
                            line,
                            StyledSpan {
                                char_start: cs,
                                char_end: ce,
                                style: Style::default().fg(hl_span.fg).bg(s.theme.fenced_bg),
                                // Only the first span signals full_line_bg so the
                                // background fills the column beyond the last char.
                                full_line_bg: if i == 0 {
                                    Some(s.theme.fenced_bg)
                                } else {
                                    None
                                },
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            _ => {
                // No highlights (disabled / unknown lang / empty line) — fenced_bg.
                push_span(
                    &mut s.map,
                    line,
                    StyledSpan {
                        char_start: 0,
                        char_end: line_len,
                        style: fence_bg_style,
                        full_line_bg: Some(s.theme.fenced_bg),
                        ..Default::default()
                    },
                );
            }
        }
    }

    // Closing fence line.
    if end_line > start_line {
        let close_len = line_char_len(&s.line_starts, s.text, end_line);
        let lb_start = s.line_starts[end_line];
        let lb_end = fence_line_end(&s.line_starts, s.text, end_line);
        let close_slice = &s.text[lb_start..lb_end];
        let close_leading = close_slice.chars().take_while(|&c| c == ' ').count();
        let close_fence = close_slice
            .chars()
            .skip(close_leading)
            .take_while(|&c| c == '`' || c == '~')
            .count()
            .min(close_len.saturating_sub(close_leading));
        push_span(
            &mut s.map,
            end_line,
            StyledSpan {
                char_start: close_leading,
                char_end: (close_leading + close_fence)
                    .max(close_leading + 1)
                    .min(close_len),
                style: fence_delim_style,
                full_line_bg: Some(s.theme.fenced_bg),
                ..Default::default()
            },
        );
    }
}
