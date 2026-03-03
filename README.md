# grift-site

Personal showcase website, blog, and documentation for the
[Grift](https://github.com/skyfskyf/grift) programming language. Features an
embedded REPL powered by [bevy_ratatui](https://github.com/ratatui/bevy_ratatui)
compiled to WebAssembly.

## Features

- **Interactive REPL** — A browser-based Grift interpreter with a terminal UI
  rendered via bevy_ratatui + WASM (with JavaScript fallback)
- **Language Documentation** — Getting started guide, language reference,
  formal specification, architecture, and contributor guide
- **Blog** — Articles about Grift, Lisp, vau calculus, and embedded systems
- **Project Showcase** — Library cards for Grift and bevy_ratatui

## Development

```bash
npm install
npm run dev
```

## Build

```bash
npm run build
```

The site is built with [Astro](https://astro.build) and
[Starlight](https://starlight.astro.build) and deployed to GitHub Pages.

## Project Structure

```
src/
├── content/docs/           Markdown/MDX content pages
│   ├── index.mdx           Home page (splash)
│   ├── blog.mdx            Blog index
│   ├── projects.mdx        Project showcase
│   ├── demo.mdx            REPL demo page
│   ├── blog/               Blog posts
│   ├── language/            Language documentation
│   │   ├── getting-started.md
│   │   ├── reference.md
│   │   └── spec.md
│   └── internals/          Internal documentation
│       ├── architecture.md
│       └── contributor-guide.md
├── components/
│   └── GriftRepl.astro     WASM REPL component with JS fallback
├── styles/
│   └── custom.css          Theme customizations
public/
├── favicon.svg
└── wasm/                   WASM REPL binaries (built separately)
```

## WASM REPL

The main demo is a bevy_ratatui powered Grift REPL compiled to WASM. To build
the WASM module from the [grift](https://github.com/skyfskyf/grift) repository:

```bash
# In the grift repo
wasm-pack build --target web crates/grift-repl

# Copy output to this site
cp pkg/* ../grift-site/public/wasm/
```

When the WASM module is not available, the demo page falls back to a JavaScript
implementation that supports basic Grift expression evaluation.

## License

MIT