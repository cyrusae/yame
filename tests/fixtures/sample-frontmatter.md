---
title: Sample Document
author: Cyrus
date: 2026-06-06
tags: markdown, yame, test
draft: false
---

# Heading one

This paragraph follows the frontmatter block. The key/value pairs above
should render with **accent-coloured keys**, muted separators, and body-text
values — all on the subtle heading background tint.

## TOML variant

You can swap the delimiters to `+++` to test TOML frontmatter instead.
Open `sample-frontmatter-toml.md` for the live TOML version.

> The delimiters (`---`) on line 1 and line 7 should be styled as
> frontmatter, **not** as a horizontal rule.

- Normal list item
- Another item
- [ ] Todo item
- [x] Done item

---

Horizontal rule above should **not** be styled as frontmatter — it appears
mid-document, not at the top.

```r
h <- "ello world"
```
