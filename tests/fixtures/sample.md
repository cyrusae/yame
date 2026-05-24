# Heading One

## Heading Two

### Heading Three

#### Heading Four

Normal paragraph text with **bold content**, *italic content*, and `inline code`. Some sentences
are long enough that they will soft-wrap inside a narrow editing column, which exercises the
renderer's line-wrapping logic and continuation indentation.

**Bold at the start of a line** and a line ending in *italic*.

Mixed: **bold and *nested italic* inside bold** — pulldown-cmark handles this correctly.

---

## Inline Elements

A [link to example](https://example.com) in the middle of a sentence.

A bare link with a long URL: [Rust documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html).

`inline code` at the start, middle `of` a sentence, and at the end `too`.

---

## Blockquotes

> A single-line blockquote.

> A blockquote that is long enough to soft-wrap inside the editing column, which means the
> renderer must correctly indent continuation lines to align with the text start after the
> left-edge indicator character.

> Nested structure:
>
> Second paragraph inside the same blockquote block.

---

## Lists

Unordered:

- First item
- Second item with **bold** inside
- Third item with `inline code`
  - Nested item one
  - Nested item two

Ordered:

1. First ordered item
2. Second ordered item
3. Third ordered item
   1. Nested ordered item

---

## Task Lists

- [ ] Unchecked todo item
- [x] Checked and completed todo item
- [ ] Another unchecked item with **bold** content
- [x] Another completed item

---

## Fenced Code Blocks

```rust
fn blend(fg: (u8, u8, u8), bg: (u8, u8, u8), ratio: f32) -> (u8, u8, u8) {
    let r = (fg.0 as f32 * ratio + bg.0 as f32 * (1.0 - ratio)) as u8;
    let g = (fg.1 as f32 * ratio + bg.1 as f32 * (1.0 - ratio)) as u8;
    let b = (fg.2 as f32 * ratio + bg.2 as f32 * (1.0 - ratio)) as u8;
    (r, g, b)
}
```

```python
def word_count(text: str) -> int:
    return len(text.split())
```

A fenced block with no language tag:

```
plain text block
no syntax highlighting
just fenced_bg tint
```

---

## Tables

| Element       | Style                        | Notes               |
| ------------- | ---------------------------- | ------------------- |
| Heading       | Bold + accent color          | Full-line bg tint   |
| Bold          | Bold weight                  | Includes delimiters |
| Italic        | Italic + emphasis color      | With fallback       |
| Inline code   | Code color + bg tint         | Backticks included  |
| Link          | Underline + accent / muted   | Split at `](`       |

---

## Multi-byte Characters

Café, naïve, résumé — accented Latin characters.

Japanese: 日本語のテキスト (Japanese text) — multi-byte UTF-8 sequences in a paragraph.

Emoji at end of line: 🦀

A heading with multi-byte content:

### Ünïcödé Héàding

Code block containing multi-byte characters:

```
# comment with emoji 🎉
value = "café"
```

Link with non-ASCII text: [日本語](https://example.co.jp).

---

## Edge Cases

Empty blockquote line below:

>

A paragraph immediately after a heading with no blank line between them is unusual but valid Markdown.
#### Tight Heading
Followed immediately by text.

Very long unbroken word that cannot be split at a space boundary and must be hard-wrapped by the renderer: superlongwordwithnospacesthatexceedsthecolumnwidthandmustbeforcebroken.

Asterisks that are *not* emphasis because they are unmatched: 2 * 3 = 6.

Backtick that is `not closed — just a literal character after this point.
