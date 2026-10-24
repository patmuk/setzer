+++
title = "Hello World"
hello = "Hello"
world = "World"
+++

Everything above the closing `+++` is front matter: each key becomes a field of
`Content`, so `render` reads `c.title`, `c.hello` and `c.world`.

Everything below it is the **body**: ordinary Markdown that arrives as `c.body`
and renders as HTML wherever you place it with `(c.body)`.

## What the body can hold

- paragraphs, *emphasis* and **strong** text
- headings, lists and `inline code`
- links, such as [the Setzer repository](https://github.com/patmuk/setzer)

Links without a scheme point to pages of this site and are checked when you
build, so a link to a page that does not exist will not compile.
