use std::cell::RefCell;
use std::rc::Rc;

use grift::Lisp;
use ratzilla::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratzilla::ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratzilla::ratatui::style::{Color, Modifier, Style, Stylize};
use ratzilla::ratatui::text::{Line, Span, Text};
use ratzilla::ratatui::widgets::{Block, BorderType, List, ListItem, Paragraph, Tabs, Wrap};
use ratzilla::ratatui::Frame;
use ratzilla::widgets::Hyperlink;
use ratzilla::DomBackend;
use ratzilla::WebRenderer;

use tachyonfx::fx::{self};
use tachyonfx::{Duration, Effect, EffectRenderer, EffectTimer, Interpolation, Motion, SimpleRng};

const BANNER: &str = r#"
 ██████╗  ██████╗ ██╗     ██████╗    ███████╗██╗██╗    ██╗   ██╗███████╗██████╗
██╔════╝ ██╔═══██╗██║     ██╔══██╗   ██╔════╝██║██║    ██║   ██║██╔════╝██╔══██╗
██║  ███╗██║   ██║██║     ██║  ██║   ███████╗██║██║    ██║   ██║█████╗  ██████╔╝
██║   ██║██║   ██║██║     ██║  ██║   ╚════██║██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗
╚██████╔╝╚██████╔╝███████╗██████╔╝██╗███████║██║███████╗╚████╔╝ ███████╗██║  ██║
 ╚═════╝  ╚═════╝ ╚══════╝╚═════╝ ╚═╝╚══════╝╚═╝╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝

                       ██████╗ ██████╗ ██████╗ ██████╗ ███████╗██████╗
                      ██╔════╝██╔═══██╗██╔══██╗██╔══██╗██╔════╝██╔══██╗
                      ██║     ██║   ██║██████╔╝██████╔╝█████╗  ██████╔╝
                      ██║     ██║   ██║██╔═══╝ ██╔═══╝ ██╔══╝  ██╔══██╗
                      ╚██████╗╚██████╔╝██║     ██║     ███████╗██║  ██║
                       ╚═════╝ ╚═════╝ ╚═╝     ╚═╝     ╚══════╝╚═╝  ╚═╝
"#;

const DESCRIPTION: &str = "\
>_ Personal website of gold.silver.copper\n\
\n\
Software developer • Rust enthusiast • Language designer\n\
\n\
Creator of Grift — a minimalistic Lisp implementing vau calculus.\n\
Features: no_std, no_alloc, arena-allocated, tail-call optimized,\n\
mark-and-sweep GC, and zero unsafe code.\n\
\n\
This terminal-themed site is built with Ratzilla + TachyonFX + WebAssembly.";

const LINKS: &[(&str, &str)] = &[
    (
        "GitHub (gold-silver-copper)",
        "https://github.com/gold-silver-copper",
    ),
    ("GitHub (grift)", "https://github.com/skyfskyf/grift"),
    (
        "GitHub (grift-site)",
        "https://github.com/skyfskyf/grift-site",
    ),
    ("Ratzilla", "https://github.com/ratatui/ratzilla"),
    ("Ratatui", "https://github.com/ratatui/ratatui"),
    ("TachyonFX", "https://github.com/ratatui/tachyonfx"),
];

const DOC_BASICS: &str = "\
Grift Basics\n\
────────────\n\
\n\
Grift is a Kernel-style Lisp with first-class operatives (fexprs).\n\
All values live in a fixed-size arena with const-generic capacity.\n\
\n\
Atoms:\n\
  42          ; number\n\
  #t #f       ; booleans\n\
  hello       ; symbol\n\
  \"hello\"     ; string\n\
  ()          ; nil / empty list\n\
  #inert      ; inert value (side-effect returns)\n\
  #ignore     ; ignore (parameter matching)\n\
\n\
Arithmetic:\n\
  (+ 1 2)           => 3\n\
  (* 6 7)           => 42\n\
  (- 10 3)          => 7\n\
  (/ 20 4)          => 5\n\
  (mod 10 3)        => 1\n\
\n\
Comparison:\n\
  (=? 1 1)          => #t\n\
  (<? 1 2)          => #t\n\
  (>? 2 1)          => #t\n\
  (<=? 1 1)         => #t\n\
  (>=? 2 1)         => #t";

const DOC_FORMS: &str = "\
Special Forms & Definitions\n\
───────────────────────────\n\
\n\
Define variables:\n\
  (define! x 42)\n\
  x                 => 42\n\
\n\
Lambda (applicative):\n\
  (define! double (lambda (x) (* x 2)))\n\
  (double 21)       => 42\n\
\n\
Conditionals:\n\
  (if #t 1 2)       => 1\n\
  (if #f 1 2)       => 2\n\
  (cond (#f 1) (#t 2))  => 2\n\
\n\
Lists:\n\
  (list 1 2 3)      => (1 2 3)\n\
  (cons 1 (list 2)) => (1 2)\n\
  (car (list 1 2))  => 1\n\
  (cdr (list 1 2))  => (2)\n\
\n\
Operatives (vau / fexprs):\n\
  ($vau (x) e x)    ; raw operative\n\
  (wrap ($vau (x) #ignore x)) ; applicative\n\
\n\
Let bindings:\n\
  (let ((x 1) (y 2)) (+ x y)) => 3\n\
\n\
Sequencing:\n\
  (begin (define! a 1) (+ a 2)) => 3";

const DOC_ADVANCED: &str = "\
Advanced Features\n\
─────────────────\n\
\n\
String operations:\n\
  (string-length \"hello\")     => 5\n\
  (string-append \"hi\" \" \" \"there\") => \"hi there\"\n\
\n\
Higher-order functions:\n\
  (map (lambda (x) (* x x)) (list 1 2 3))\n\
    => (1 4 9)\n\
  (filter (lambda (x) (>? x 2)) (list 1 2 3 4))\n\
    => (3 4)\n\
  (reduce + 0 (list 1 2 3))\n\
    => 6\n\
\n\
Recursion (tail-call optimized):\n\
  (define! fact\n\
    (lambda (n)\n\
      (if (=? n 0) 1\n\
        (* n (fact (- n 1))))))\n\
  (fact 10)           => 3628800\n\
\n\
Boolean logic:\n\
  (and? #t #f)        => #f\n\
  (or? #t #f)         => #t\n\
  (not? #t)           => #f\n\
\n\
Type checking:\n\
  (number? 42)        => #t\n\
  (string? \"hi\")      => #t\n\
  (pair? (list 1))    => #t\n\
  (null? ())          => #t\n\
  (boolean? #t)       => #t";

const BLOG_ENTRIES: &[(&str, &str, &str)] = &[
    (
        "Welcome to gold.silver.copper",
        "2025-01-15",
        "Hi! I'm gold.silver.copper — a software developer passionate\n\
         about programming languages, systems programming, and Rust.\n\
         \n\
         This site serves as my personal blog, project showcase, and\n\
         an interactive demo of Grift, my Lisp interpreter.\n\
         \n\
         Everything you see here is rendered as a terminal UI in your\n\
         browser using Ratzilla + TachyonFX + WebAssembly.",
    ),
    (
        "Building Grift: A Minimalistic Lisp",
        "2025-02-01",
        "Grift implements Kernel-style vau calculus with first-class\n\
         operatives that subsume both functions and macros.\n\
         \n\
         Key design goals:\n\
         - Zero unsafe code (#![forbid(unsafe_code)])\n\
         - No heap allocation (arena-only memory)\n\
         - Runs on bare-metal embedded systems\n\
         - Compiles to WebAssembly\n\
         \n\
         All values live in a fixed-size arena with const-generic\n\
         capacity and mark-and-sweep garbage collection.",
    ),
    (
        "Vau Calculus Explained",
        "2025-03-10",
        "Unlike traditional Lisps, Grift uses vau calculus where\n\
         operatives receive their arguments unevaluated along with\n\
         the caller's environment. This makes operatives strictly\n\
         more powerful than macros — they can choose whether and\n\
         when to evaluate each argument.\n\
         \n\
         ($vau (x) env-param body) creates an operative that\n\
         captures the formal parameter tree, environment parameter,\n\
         and body expression as a closure.",
    ),
    (
        "Terminal UIs in the Browser",
        "2025-04-20",
        "This website is built entirely with Ratzilla, which brings\n\
         Ratatui's terminal UI framework to the browser via WASM.\n\
         \n\
         TachyonFX adds shader-like visual effects — the background\n\
         animation, page transitions, and link click effects are all\n\
         powered by tachyonfx running in WebAssembly.\n\
         \n\
         No JavaScript framework. No DOM manipulation. Just Rust\n\
         rendering a terminal buffer to a canvas element.",
    ),
];

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Home,
    Repl,
    Docs,
    Blog,
    Links,
}

impl Page {
    const ALL: [Page; 5] = [Page::Home, Page::Repl, Page::Docs, Page::Blog, Page::Links];

    fn title(self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Repl => "REPL",
            Page::Docs => "Docs",
            Page::Blog => "Blog",
            Page::Links => "Links",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|&p| p == self).unwrap_or(0)
    }
}

struct App {
    page: Page,
    // REPL state
    repl_input: String,
    repl_cursor: usize,
    repl_history: Vec<(String, String)>,
    repl_scroll: usize,
    lisp: Box<Lisp<2000>>,
    // Docs state
    doc_page: usize,
    // Blog state
    blog_index: usize,
    // TachyonFX
    transition_effect: Option<Effect>,
    bg_tick: u64,
    rng: SimpleRng,
    last_frame: web_time::Instant,
    // Clickable area tracking
    tab_area: Rect,
    link_areas: Vec<Rect>,
    blog_item_areas: Vec<Rect>,
    doc_nav_prev: Rect,
    doc_nav_next: Rect,
}

impl App {
    fn new() -> Self {
        let lisp: Box<Lisp<2000>> = Box::new(Lisp::new());
        Self {
            page: Page::Home,
            repl_input: String::new(),
            repl_cursor: 0,
            repl_history: Vec::new(),
            repl_scroll: 0,
            lisp,
            doc_page: 0,
            blog_index: 0,
            transition_effect: None,
            bg_tick: 0,
            rng: SimpleRng::default(),
            last_frame: web_time::Instant::now(),
            tab_area: Rect::default(),
            link_areas: Vec::new(),
            blog_item_areas: Vec::new(),
            doc_nav_prev: Rect::default(),
            doc_nav_next: Rect::default(),
        }
    }

    fn trigger_transition(&mut self) {
        let variant = self.rng.gen() % 6;
        let effect = match variant {
            0 => fx::fade_from(
                Color::Black,
                Color::Black,
                EffectTimer::from_ms(400, Interpolation::CubicOut),
            ),
            1 => fx::sweep_in(
                Motion::LeftToRight,
                10,
                3,
                Color::Black,
                EffectTimer::from_ms(500, Interpolation::QuadOut),
            ),
            2 => fx::sweep_in(
                Motion::UpToDown,
                8,
                2,
                Color::Black,
                EffectTimer::from_ms(500, Interpolation::QuadOut),
            ),
            3 => fx::coalesce(EffectTimer::from_ms(400, Interpolation::SineOut)),
            4 => fx::slide_in(
                Motion::RightToLeft,
                8,
                3,
                Color::Black,
                EffectTimer::from_ms(500, Interpolation::CubicOut),
            ),
            _ => fx::fade_from(
                Color::Rgb(0, 40, 0),
                Color::Black,
                EffectTimer::from_ms(350, Interpolation::Linear),
            ),
        };
        self.transition_effect = Some(effect);
    }

    fn switch_page(&mut self, page: Page) {
        if self.page != page {
            self.page = page;
            self.trigger_transition();
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match self.page {
            Page::Repl => self.handle_repl_event(key),
            Page::Docs => self.handle_docs_event(key),
            Page::Blog => self.handle_blog_event(key),
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) {
        if event.event == MouseEventKind::Pressed && event.button == MouseButton::Left {
            // Convert pixel coordinates to terminal grid coordinates
            // Ratzilla uses ~10px per column, ~20px per row for DomBackend
            let col = (event.x / 10) as u16;
            let row = (event.y / 20) as u16;

            // Check tab clicks
            if row >= self.tab_area.y && row < self.tab_area.bottom() {
                let tab_width = if self.tab_area.width > 0 {
                    self.tab_area.width / Page::ALL.len() as u16
                } else {
                    0
                };
                if tab_width > 0 && col >= self.tab_area.x && col < self.tab_area.right() {
                    let rel_x = col - self.tab_area.x;
                    let tab_idx = (rel_x / tab_width) as usize;
                    if tab_idx < Page::ALL.len() {
                        self.switch_page(Page::ALL[tab_idx]);
                        return;
                    }
                }
            }

            // Check link clicks on Links page
            if self.page == Page::Links {
                for (i, area) in self.link_areas.iter().enumerate() {
                    if col >= area.x
                        && col < area.right()
                        && row >= area.y
                        && row < area.bottom()
                    {
                        if let Some((_, url)) = LINKS.get(i) {
                            self.trigger_transition();
                            open_url(url);
                            return;
                        }
                    }
                }
            }

            // Check blog item clicks
            if self.page == Page::Blog {
                for (i, area) in self.blog_item_areas.iter().enumerate() {
                    if col >= area.x
                        && col < area.right()
                        && row >= area.y
                        && row < area.bottom()
                        && i < BLOG_ENTRIES.len()
                        && self.blog_index != i
                    {
                        self.blog_index = i;
                        self.trigger_transition();
                        return;
                    }
                }
            }

            // Check doc navigation buttons
            if self.page == Page::Docs {
                if col >= self.doc_nav_prev.x
                    && col < self.doc_nav_prev.right()
                    && row >= self.doc_nav_prev.y
                    && row < self.doc_nav_prev.bottom()
                    && self.doc_page > 0
                {
                    self.doc_page -= 1;
                    self.trigger_transition();
                }
                if col >= self.doc_nav_next.x
                    && col < self.doc_nav_next.right()
                    && row >= self.doc_nav_next.y
                    && row < self.doc_nav_next.bottom()
                    && self.doc_page < 2
                {
                    self.doc_page += 1;
                    self.trigger_transition();
                }
            }
        }
    }

    fn handle_repl_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                if !self.repl_input.is_empty() {
                    let input = self.repl_input.clone();
                    let result = match self.lisp.eval_to_index(&input) {
                        Ok(idx) => {
                            let mut buf = String::new();
                            match self.lisp.write_value(idx, &mut buf) {
                                Ok(()) => buf,
                                Err(_) => "<format error>".to_string(),
                            }
                        }
                        Err(e) => format!("Error: {e:?}"),
                    };
                    self.repl_history.push((input, result));
                    self.repl_input.clear();
                    self.repl_cursor = 0;
                    let total = self.repl_history.len() * 2;
                    self.repl_scroll = total;
                }
            }
            KeyCode::Char(c) => {
                let byte_idx = self.byte_index();
                self.repl_input.insert(byte_idx, c);
                self.repl_cursor += 1;
            }
            KeyCode::Backspace => {
                if self.repl_cursor > 0 {
                    let current = self.repl_cursor;
                    let before: String = self.repl_input.chars().take(current - 1).collect();
                    let after: String = self.repl_input.chars().skip(current).collect();
                    self.repl_input = before + &after;
                    self.repl_cursor -= 1;
                }
            }
            KeyCode::Left => {
                self.repl_cursor = self.repl_cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                let max = self.repl_input.chars().count();
                if self.repl_cursor < max {
                    self.repl_cursor += 1;
                }
            }
            KeyCode::Up => {
                self.repl_scroll = self.repl_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.repl_scroll += 1;
            }
            _ => {}
        }
    }

    fn handle_docs_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left => {
                if self.doc_page > 0 {
                    self.doc_page -= 1;
                    self.trigger_transition();
                }
            }
            KeyCode::Right => {
                if self.doc_page < 2 {
                    self.doc_page += 1;
                    self.trigger_transition();
                }
            }
            _ => {}
        }
    }

    fn handle_blog_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Up => {
                if self.blog_index > 0 {
                    self.blog_index -= 1;
                    self.trigger_transition();
                }
            }
            KeyCode::Right | KeyCode::Down => {
                if self.blog_index < BLOG_ENTRIES.len() - 1 {
                    self.blog_index += 1;
                    self.trigger_transition();
                }
            }
            _ => {}
        }
    }

    fn byte_index(&self) -> usize {
        self.repl_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.repl_cursor)
            .unwrap_or(self.repl_input.len())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let now = web_time::Instant::now();
        let elapsed_std = now - self.last_frame;
        self.last_frame = now;
        let elapsed = Duration::from_millis(elapsed_std.as_millis() as u32);

        self.bg_tick = self.bg_tick.wrapping_add(1);

        // Render background animation
        self.render_background(frame);

        let main_area = frame.area();

        let [tab_area, content_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(main_area);

        self.render_tabs(frame, tab_area);

        match self.page {
            Page::Home => self.render_home(frame, content_area),
            Page::Repl => self.render_repl(frame, content_area),
            Page::Docs => self.render_docs(frame, content_area),
            Page::Blog => self.render_blog(frame, content_area),
            Page::Links => self.render_links(frame, content_area),
        }

        // Process transition effects
        if let Some(ref mut effect) = self.transition_effect {
            if effect.running() {
                frame.render_effect(effect, content_area, elapsed);
            }
        }
        if self
            .transition_effect
            .as_ref()
            .is_some_and(|e| !e.running())
        {
            self.transition_effect = None;
        }
    }

    fn render_background(&self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        let tick = self.bg_tick;

        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let pos = Position::new(x, y);
                if let Some(cell) = buf.cell_mut(pos) {
                    // Procedural calming wave pattern
                    let fx = x as f64 * 0.15;
                    let fy = y as f64 * 0.3;
                    let ft = tick as f64 * 0.02;

                    let wave1 = ((fx + ft).sin() * 0.5 + 0.5) * 0.4;
                    let wave2 = ((fy * 0.7 + ft * 1.3).cos() * 0.5 + 0.5) * 0.3;
                    let wave3 = ((fx * 0.5 + fy * 0.5 + ft * 0.7).sin() * 0.5 + 0.5) * 0.3;

                    let intensity = wave1 + wave2 + wave3;

                    let r = (5.0 + intensity * 12.0) as u8;
                    let g = (10.0 + intensity * 25.0) as u8;
                    let b = (15.0 + intensity * 18.0) as u8;

                    cell.set_bg(Color::Rgb(r, g, b));
                    cell.set_fg(Color::Rgb(
                        (r as u16 + 15).min(255) as u8,
                        (g as u16 + 25).min(255) as u8,
                        (b as u16 + 15).min(255) as u8,
                    ));
                }
            }
        }
    }

    fn render_tabs(&mut self, frame: &mut Frame, area: Rect) {
        self.tab_area = area;

        let titles: Vec<Line> = Page::ALL
            .iter()
            .map(|p| {
                Line::from(vec![
                    Span::styled(" ", Style::default()),
                    Span::styled(
                        p.title(),
                        Style::default().fg(Color::Rgb(180, 220, 180)),
                    ),
                    Span::styled(" ", Style::default()),
                ])
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Rgb(60, 80, 60))
                    .title(" gold.silver.copper ")
                    .title_style(Style::default().fg(Color::Rgb(100, 200, 100)).bold()),
            )
            .select(self.page.index())
            .style(Style::default().fg(Color::Rgb(140, 140, 140)))
            .highlight_style(
                Style::default()
                    .fg(Color::Rgb(100, 200, 100))
                    .bold()
                    .add_modifier(Modifier::UNDERLINED),
            )
            .divider(" │ ");

        frame.render_widget(tabs, area);
    }

    fn render_home(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(60, 80, 60))
            .title_bottom(
                Line::from("│ click tabs to navigate │")
                    .alignment(Alignment::Right)
                    .style(Style::default().fg(Color::Rgb(60, 80, 60))),
            );

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let banner_height = BANNER.lines().count() as u16;
        let desc_lines = DESCRIPTION.lines().count() as u16;

        let [banner_area, desc_area, nav_area] = Layout::vertical([
            Constraint::Length(banner_height),
            Constraint::Length(desc_lines + 2),
            Constraint::Min(3),
        ])
        .areas(inner);

        // Banner
        let banner = Paragraph::new(BANNER)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(100, 200, 100)).bold());
        frame.render_widget(banner, banner_area);

        // Description
        let desc = Paragraph::new(DESCRIPTION)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Rgb(180, 180, 180)))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Rgb(50, 70, 50))
                    .title(" About ".bold().fg(Color::Rgb(100, 200, 100))),
            );
        frame.render_widget(desc, desc_area);

        // Navigation help
        let nav_text = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                "  Click the ".fg(Color::Rgb(120, 120, 120)),
                "tabs above".fg(Color::Rgb(100, 200, 100)).bold(),
                " to navigate between pages".fg(Color::Rgb(120, 120, 120)),
            ]),
            Line::from(vec![
                "  Try the ".fg(Color::Rgb(120, 120, 120)),
                "REPL".fg(Color::Rgb(100, 200, 100)).bold(),
                " to evaluate Grift expressions interactively"
                    .fg(Color::Rgb(120, 120, 120)),
            ]),
            Line::from(vec![
                "  All ".fg(Color::Rgb(120, 120, 120)),
                "links".fg(Color::Rgb(100, 200, 100)).bold(),
                " and ".fg(Color::Rgb(120, 120, 120)),
                "buttons".fg(Color::Rgb(100, 200, 100)).bold(),
                " are clickable".fg(Color::Rgb(120, 120, 120)),
            ]),
        ]);
        frame.render_widget(
            Paragraph::new(nav_text).block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Rgb(50, 70, 50))
                    .title(" Navigation ".bold().fg(Color::Rgb(100, 200, 100))),
            ),
            nav_area,
        );
    }

    fn render_repl(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(100, 200, 100))
            .title(" Grift REPL ".bold().fg(Color::Rgb(100, 200, 100)))
            .title_bottom(
                Line::from("│ Type expressions and press Enter to evaluate │")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Rgb(60, 80, 60))),
            );

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let [history_area, input_area] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(inner);

        // History
        let mut history_lines: Vec<Line> = Vec::new();
        for (input, output) in &self.repl_history {
            history_lines.push(Line::from(vec![
                Span::styled(
                    "grift> ",
                    Style::default().fg(Color::Rgb(100, 200, 100)).bold(),
                ),
                Span::styled(input.as_str(), Style::default().fg(Color::Rgb(200, 200, 200))),
            ]));
            history_lines.push(Line::from(vec![Span::styled(
                format!("  => {output}"),
                Style::default().fg(Color::Rgb(180, 180, 100)),
            )]));
        }

        if history_lines.is_empty() {
            history_lines.push(Line::from(
                "  Welcome to the Grift REPL! Type expressions and press Enter."
                    .fg(Color::Rgb(120, 120, 120)),
            ));
            history_lines.push(Line::from(
                "  Try: (+ 1 2), (list 1 2 3), (define! x 42)"
                    .fg(Color::Rgb(120, 120, 120)),
            ));
        }

        let visible_height = history_area.height as usize;
        let total_lines = history_lines.len();
        let max_scroll = total_lines.saturating_sub(visible_height);
        let scroll = self.repl_scroll.min(max_scroll);

        let history = Paragraph::new(Text::from(history_lines))
            .scroll((scroll as u16, 0))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Rgb(50, 70, 50))
                    .title(" Output ".fg(Color::Rgb(180, 180, 100))),
            );
        frame.render_widget(history, history_area);

        // Input
        let input_display = format!("grift> {}", self.repl_input);
        let input = Paragraph::new(input_display.as_str())
            .style(Style::default().fg(Color::Rgb(200, 200, 200)))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Rgb(100, 200, 100))
                    .title(" Input ".bold().fg(Color::Rgb(100, 200, 100))),
            );
        frame.render_widget(input, input_area);

        // Cursor position
        let cursor_x = input_area.x + 1 + 7 + self.repl_cursor as u16;
        let cursor_y = input_area.y + 1;
        if cursor_x < input_area.right() - 1 {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }

    fn render_docs(&mut self, frame: &mut Frame, area: Rect) {
        let doc_content = match self.doc_page {
            0 => DOC_BASICS,
            1 => DOC_FORMS,
            2 => DOC_ADVANCED,
            _ => DOC_BASICS,
        };

        let doc_titles = ["Basics", "Forms", "Advanced"];

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(60, 80, 60))
            .title(
                format!(" Documentation: {} ", doc_titles[self.doc_page])
                    .bold()
                    .fg(Color::Rgb(100, 200, 100)),
            );

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split inner into doc content and navigation buttons
        let [doc_area, nav_bar] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(inner);

        // Render doc content with syntax highlighting
        let lines: Vec<Line> = doc_content
            .lines()
            .map(|line| {
                if line.starts_with("  (") || line.starts_with("    (") {
                    if let Some((code, comment)) = line.split_once(';') {
                        if let Some((expr, result)) = code.split_once("=>") {
                            Line::from(vec![
                                Span::styled(expr, Style::default().fg(Color::Rgb(200, 200, 200))),
                                Span::styled("=>", Style::default().fg(Color::Rgb(80, 80, 80))),
                                Span::styled(
                                    result,
                                    Style::default().fg(Color::Rgb(180, 180, 100)),
                                ),
                                Span::styled(
                                    format!(";{comment}"),
                                    Style::default().fg(Color::Rgb(100, 100, 100)),
                                ),
                            ])
                        } else {
                            Line::from(vec![
                                Span::styled(code, Style::default().fg(Color::Rgb(200, 200, 200))),
                                Span::styled(
                                    format!(";{comment}"),
                                    Style::default().fg(Color::Rgb(100, 100, 100)),
                                ),
                            ])
                        }
                    } else if let Some((expr, result)) = line.split_once("=>") {
                        Line::from(vec![
                            Span::styled(expr, Style::default().fg(Color::Rgb(200, 200, 200))),
                            Span::styled("=>", Style::default().fg(Color::Rgb(80, 80, 80))),
                            Span::styled(result, Style::default().fg(Color::Rgb(180, 180, 100))),
                        ])
                    } else {
                        Line::styled(line, Style::default().fg(Color::Rgb(200, 200, 200)))
                    }
                } else if line.starts_with("  ") && !line.trim().is_empty() {
                    if let Some((code, comment)) = line.split_once(';') {
                        Line::from(vec![
                            Span::styled(code, Style::default().fg(Color::Rgb(200, 200, 200))),
                            Span::styled(
                                format!(";{comment}"),
                                Style::default().fg(Color::Rgb(100, 100, 100)),
                            ),
                        ])
                    } else if let Some((expr, result)) = line.split_once("=>") {
                        Line::from(vec![
                            Span::styled(expr, Style::default().fg(Color::Rgb(200, 200, 200))),
                            Span::styled("=>", Style::default().fg(Color::Rgb(80, 80, 80))),
                            Span::styled(result, Style::default().fg(Color::Rgb(180, 180, 100))),
                        ])
                    } else {
                        Line::styled(line, Style::default().fg(Color::Rgb(200, 200, 200)))
                    }
                } else if line.contains('─') {
                    Line::styled(line, Style::default().fg(Color::Rgb(100, 200, 100)))
                } else if !line.trim().is_empty() {
                    Line::styled(
                        line,
                        Style::default().fg(Color::Rgb(100, 200, 100)).bold(),
                    )
                } else {
                    Line::from("")
                }
            })
            .collect();

        let doc = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
        frame.render_widget(doc, doc_area);

        // Navigation buttons
        let [prev_area, info_area, next_area] = Layout::horizontal([
            Constraint::Length(12),
            Constraint::Min(1),
            Constraint::Length(12),
        ])
        .areas(nav_bar);

        self.doc_nav_prev = prev_area;
        self.doc_nav_next = next_area;

        let prev_style = if self.doc_page > 0 {
            Style::default().fg(Color::Rgb(100, 200, 100)).bold()
        } else {
            Style::default().fg(Color::Rgb(60, 60, 60))
        };
        let next_style = if self.doc_page < 2 {
            Style::default().fg(Color::Rgb(100, 200, 100)).bold()
        } else {
            Style::default().fg(Color::Rgb(60, 60, 60))
        };

        frame.render_widget(
            Paragraph::new(" [◄ Prev]").style(prev_style).block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Rgb(50, 70, 50)),
            ),
            prev_area,
        );
        frame.render_widget(
            Paragraph::new(format!(" Page {}/{} ", self.doc_page + 1, doc_titles.len()))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Rgb(100, 100, 100)))
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .border_style(Color::Rgb(50, 70, 50)),
                ),
            info_area,
        );
        frame.render_widget(
            Paragraph::new(" [Next ►]").style(next_style).block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Rgb(50, 70, 50)),
            ),
            next_area,
        );
    }

    fn render_blog(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(60, 80, 60))
            .title(" Blog ".bold().fg(Color::Rgb(100, 200, 100)))
            .title_bottom(
                Line::from("│ click a post to read │")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Rgb(60, 80, 60))),
            );

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let [list_area, content_area] =
            Layout::horizontal([Constraint::Length(35), Constraint::Min(1)]).areas(inner);

        // Blog list - track clickable areas
        self.blog_item_areas.clear();
        let items: Vec<ListItem> = BLOG_ENTRIES
            .iter()
            .enumerate()
            .map(|(i, (title, date, _))| {
                let style = if i == self.blog_index {
                    Style::default().fg(Color::Rgb(100, 200, 100)).bold()
                } else {
                    Style::default().fg(Color::Rgb(150, 150, 150))
                };
                let marker = if i == self.blog_index { "▶ " } else { "  " };
                ListItem::new(vec![
                    Line::from(format!("{marker}{title}")).style(style),
                    Line::from(format!("  {date}"))
                        .style(Style::default().fg(Color::Rgb(80, 80, 80))),
                ])
            })
            .collect();

        let list_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(50, 70, 50))
            .title(" Posts ".fg(Color::Rgb(100, 200, 100)));

        let list_inner = list_block.inner(list_area);
        // Each blog item is 2 lines tall
        for i in 0..BLOG_ENTRIES.len() {
            let item_y = list_inner.y + (i as u16 * 2);
            if item_y + 2 <= list_inner.bottom() {
                self.blog_item_areas.push(Rect::new(
                    list_inner.x,
                    item_y,
                    list_inner.width,
                    2,
                ));
            }
        }

        let list = List::new(items).block(list_block);
        frame.render_widget(list, list_area);

        // Blog content
        if let Some((title, date, content)) = BLOG_ENTRIES.get(self.blog_index) {
            let mut lines = vec![
                Line::styled(
                    *title,
                    Style::default().fg(Color::Rgb(100, 200, 100)).bold(),
                ),
                Line::styled(*date, Style::default().fg(Color::Rgb(80, 80, 80))),
                Line::styled(
                    "─".repeat(content_area.width.saturating_sub(4) as usize),
                    Style::default().fg(Color::Rgb(50, 70, 50)),
                ),
                Line::from(""),
            ];
            for line in content.lines() {
                lines.push(Line::styled(
                    line,
                    Style::default().fg(Color::Rgb(180, 180, 180)),
                ));
            }

            let blog = Paragraph::new(Text::from(lines))
                .wrap(Wrap { trim: false })
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .border_style(Color::Rgb(50, 70, 50)),
                );
            frame.render_widget(blog, content_area);
        }
    }

    fn render_links(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(60, 80, 60))
            .title(" Links ".bold().fg(Color::Rgb(100, 200, 100)))
            .title_bottom(
                Line::from("│ click a link to open │")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Rgb(60, 80, 60))),
            );

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let [links_area, info_area] =
            Layout::vertical([Constraint::Length(LINKS.len() as u16 + 2), Constraint::Min(1)])
                .areas(inner);

        // Links with hyperlinks
        let links_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(50, 70, 50))
            .title(" Repositories & Resources ".fg(Color::Rgb(100, 200, 100)));

        let links_inner = links_block.inner(links_area);
        frame.render_widget(links_block, links_area);

        self.link_areas.clear();
        for (i, (_label, url)) in LINKS.iter().enumerate() {
            let link_area = Rect::new(links_inner.x, links_inner.y + i as u16, links_inner.width, 1);
            if link_area.y < links_inner.bottom() {
                // Render hyperlink (makes it clickable via ratzilla)
                let link = Hyperlink::new(*url);
                frame.render_widget(link, link_area);
                self.link_areas.push(link_area);
            }
        }

        // Info section
        let info_text = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                "  gold.silver.copper ".fg(Color::Rgb(100, 200, 100)).bold(),
                "— Software developer & language designer"
                    .fg(Color::Rgb(150, 150, 150)),
            ]),
            Line::from(""),
            Line::from(vec![
                "  Grift ".fg(Color::Rgb(100, 200, 100)).bold(),
                "– A minimalistic Lisp implementing vau calculus"
                    .fg(Color::Rgb(150, 150, 150)),
            ]),
            Line::from(vec![
                "  Ratzilla ".fg(Color::Rgb(100, 200, 100)).bold(),
                "– Terminal-themed web apps with Rust + WASM"
                    .fg(Color::Rgb(150, 150, 150)),
            ]),
            Line::from(vec![
                "  TachyonFX ".fg(Color::Rgb(100, 200, 100)).bold(),
                "– Shader-like effects for terminal UIs"
                    .fg(Color::Rgb(150, 150, 150)),
            ]),
            Line::from(""),
            Line::from(
                "  This website is entirely rendered as a terminal UI in your browser."
                    .fg(Color::Rgb(100, 100, 100)),
            ),
            Line::from(
                "  Powered by Ratzilla + TachyonFX + WebAssembly."
                    .fg(Color::Rgb(100, 100, 100)),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(info_text)
                .wrap(Wrap { trim: false })
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .border_style(Color::Rgb(50, 70, 50))
                        .title(" Info ".fg(Color::Rgb(100, 200, 100))),
                ),
            info_area,
        );
    }
}

fn open_url(url: &str) {
    let _ = ratzilla::utils::open_url(url, true);
}

fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let backend = DomBackend::new().expect("failed to create DOM backend");
    let terminal = ratzilla::ratatui::Terminal::new(backend)?;

    let app = Rc::new(RefCell::new(App::new()));

    terminal.on_key_event({
        let app = app.clone();
        move |key_event| {
            app.borrow_mut().handle_key_event(key_event);
        }
    });

    terminal.on_mouse_event({
        let app = app.clone();
        move |mouse_event| {
            app.borrow_mut().handle_mouse_event(mouse_event);
        }
    });

    terminal.draw_web({
        let app = app.clone();
        move |frame| {
            app.borrow_mut().draw(frame);
        }
    });

    Ok(())
}
