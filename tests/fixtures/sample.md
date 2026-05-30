# Heading One

An edit.

## Heading Two

### Heading Three

#### Heading Four

Normal paragraph text with **bold content**, *italic content*, and `inline code`. Some sentences
are long enough that they will soft-wrap inside a narrow editing column, which exercises the
renderer's line-wrapping logic and continuation indentation.

**Bold at the start of a line** and a line ending in *italic*.

Mixed: **bold and *nested italic* inside bold** — pulldown-cmark handles this correctly.

What about *italic and **nested bold***?

---

## Inline Elements

A [link to example](https://example.com) in the middle of a sentence.

A bare link with a long URL: [Rust documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html).

`inline code` at the start, middle `of` a sentence, and at the end `too`.

What happens when I type a long line? A long line? A line that goes on for a long long time? 

~~Strikethrough.~~

***Bold and italic combined with triple asterisks.***

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
- A really really really really really really really really really really long...
  - a nested really, really, `really`, really, really, really, really long...

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
- [x] What about a really, really, really, really, really, really, really, `really` long to-do item?
    - [ ] what about nesting a to-do item? A really, really, really, really long to-do item? Nice!

Test entry

---

## Fenced Code Blocks

```rust
// Count vowels in a string slice and return a summary.
fn count_vowels(s: &str) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}

fn main() {
    let greeting = "Hello, world!";
    let n = count_vowels(greeting);
    println!("Vowels: {n}");  // 3

    let values: Vec<i32> = vec![1, 2, 3, 4, 5];
    let total: i32 = values.iter().sum();
    let doubled: Vec<i32> = values.iter().map(|&x| x * 2).collect();
    assert!(total > 0 && doubled.len() == values.len());
}
```

```python
# Compute factorials up to n using recursion.
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

results = [factorial(i) for i in range(1, 6)]
total = sum(results)
print(f"Results: {results}")   # [1, 2, 6, 24, 120]
print(f"Sum: {total}")         # 153
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
