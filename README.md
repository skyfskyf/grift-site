# Grift Site

A terminal-themed personal showcase website, blog, and interactive REPL for the [Grift](https://github.com/skyfskyf/grift) programming language.

Built entirely with [Ratzilla](https://github.com/ratatui/ratzilla) — terminal UI rendered in the browser via Rust + WebAssembly.

## Features

- **Interactive REPL** — Evaluate Grift expressions directly in the browser
- **Documentation** — Embedded language reference for Grift (basics, forms, advanced)
- **Blog** — Technical articles about the Grift interpreter
- **Links** — Project repositories and related resources
- **Terminal aesthetic** — Fully rendered as a TUI in the browser

## Building

```bash
# Install trunk (WASM bundler)
cargo install trunk

# Add the WASM target
rustup target add wasm32-unknown-unknown

# Serve locally
trunk serve

# Build for production
trunk build --release
```

## Technology

- [Grift](https://github.com/skyfskyf/grift) — A minimalistic Lisp implementing vau calculus (`no_std`, `no_alloc`)
- [Ratzilla](https://github.com/ratatui/ratzilla) — Terminal-themed web apps with Rust + WASM
- [Ratatui](https://github.com/ratatui/ratatui) — Rust TUI framework

## Navigation

| Key | Action |
|-----|--------|
| `Tab` | Cycle through pages |
| `1-5` | Jump to specific page |
| `Enter` | Evaluate REPL input |
| `←/→` | Navigate docs / move cursor |
| `↑/↓` | Scroll REPL history / navigate blog |
