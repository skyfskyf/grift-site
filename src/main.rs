use std::cell::RefCell;
use std::rc::Rc;

use grift::Lisp;
use ratzilla::event::{KeyCode, KeyEvent};
use ratzilla::ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratzilla::ratatui::style::{Color, Modifier, Style, Stylize};
use ratzilla::ratatui::text::{Line, Span, Text};
use ratzilla::ratatui::widgets::{Block, BorderType, Clear, List, ListItem, Paragraph, Tabs, Wrap};
use ratzilla::ratatui::{Frame, Terminal};
use ratzilla::widgets::Hyperlink;
use ratzilla::{DomBackend, WebRenderer};

const BANNER: &str = r#"
   ██████╗ ██████╗ ██╗███████╗████████╗
  ██╔════╝ ██╔══██╗██║██╔════╝╚══██╔══╝
  ██║  ███╗██████╔╝██║█████╗     ██║   
  ██║   ██║██╔══██╗██║██╔══╝     ██║   
  ╚██████╔╝██║  ██║██║██║        ██║   
   ╚═════╝ ╚═╝  ╚═╝╚═╝╚═╝        ╚═╝   
"#;

const DESCRIPTION: &str = "\
>_ Grift is a minimalistic Lisp interpreter implementing vau calculus.\n\
\n\
Features: no_std, no_alloc, arena-allocated, tail-call optimized,\n\
mark-and-sweep GC, and zero unsafe code.\n\
\n\
Built for embedded systems and the web with Ratzilla + WebAssembly.";

const LINKS: &[(&str, &str)] = &[
    ("GitHub (grift)", "https://github.com/skyfskyf/grift"),
    (
        "GitHub (grift-site)",
        "https://github.com/skyfskyf/grift-site",
    ),
    ("Ratzilla", "https://github.com/ratatui/ratzilla"),
    ("Ratatui", "https://github.com/ratatui/ratatui"),
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
        "Welcome to Grift",
        "2025-01-15",
        "Grift is a minimalistic Lisp interpreter built in Rust.\n\
         It implements Kernel-style vau calculus with first-class\n\
         operatives that subsume both functions and macros.\n\
         \n\
         Key design goals:\n\
         - Zero unsafe code (#![forbid(unsafe_code)])\n\
         - No heap allocation (arena-only memory)\n\
         - Runs on bare-metal embedded systems\n\
         - Compiles to WebAssembly",
    ),
    (
        "Arena Allocation in Grift",
        "2025-02-01",
        "All values in Grift live in a fixed-size arena with\n\
         const-generic capacity. This means no Vec, String, or\n\
         Box - just a flat array of slots.\n\
         \n\
         The arena supports mark-and-sweep garbage collection\n\
         triggered automatically at 75% occupancy, with explicit\n\
         collection via (gc-collect).",
    ),
    (
        "Vau Calculus Explained",
        "2025-03-10",
        "Unlike traditional Lisps, Grift uses vau calculus where\n\
         operatives receive their arguments unevaluated along with\n\
         the caller's environment. This makes operatives strictly\n\
         more powerful than macros - they can choose whether and\n\
         when to evaluate each argument.\n\
         \n\
         ($vau (x) env-param body) creates an operative that\n\
         captures the formal parameter tree, environment parameter,\n\
         and body expression as a closure.",
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
    lisp: Lisp<50000>,
    // Docs state
    doc_page: usize,
    // Blog state
    blog_index: usize,
}

impl App {
    fn new() -> Self {
        let lisp: Lisp<50000> = Lisp::new();
        Self {
            page: Page::Home,
            repl_input: String::new(),
            repl_cursor: 0,
            repl_history: Vec::new(),
            repl_scroll: 0,
            lisp,
            doc_page: 0,
            blog_index: 0,
        }
    }

    fn handle_event(&mut self, key: KeyEvent) {
        match self.page {
            Page::Home => self.handle_home_event(key),
            Page::Repl => self.handle_repl_event(key),
            Page::Docs => self.handle_docs_event(key),
            Page::Blog => self.handle_blog_event(key),
            Page::Links => self.handle_links_event(key),
        }
    }

    fn handle_global_nav(&mut self, key: &KeyEvent) -> bool {
        match key.code {
            KeyCode::Tab => {
                let idx = self.page.index();
                self.page = Page::ALL[(idx + 1) % Page::ALL.len()];
                true
            }
            KeyCode::Char('1') if self.page != Page::Repl => {
                self.page = Page::Home;
                true
            }
            KeyCode::Char('2') if self.page != Page::Repl => {
                self.page = Page::Repl;
                true
            }
            KeyCode::Char('3') if self.page != Page::Repl => {
                self.page = Page::Docs;
                true
            }
            KeyCode::Char('4') if self.page != Page::Repl => {
                self.page = Page::Blog;
                true
            }
            KeyCode::Char('5') if self.page != Page::Repl => {
                self.page = Page::Links;
                true
            }
            _ => false,
        }
    }

    fn handle_home_event(&mut self, key: KeyEvent) {
        self.handle_global_nav(&key);
    }

    fn handle_repl_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                let idx = self.page.index();
                self.page = Page::ALL[(idx + 1) % Page::ALL.len()];
            }
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
                    // Auto-scroll to bottom
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
        if self.handle_global_nav(&key) {
            return;
        }
        match key.code {
            KeyCode::Left => {
                self.doc_page = self.doc_page.saturating_sub(1);
            }
            KeyCode::Right => {
                if self.doc_page < 2 {
                    self.doc_page += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_blog_event(&mut self, key: KeyEvent) {
        if self.handle_global_nav(&key) {
            return;
        }
        match key.code {
            KeyCode::Left | KeyCode::Up => {
                self.blog_index = self.blog_index.saturating_sub(1);
            }
            KeyCode::Right | KeyCode::Down => {
                if self.blog_index < BLOG_ENTRIES.len() - 1 {
                    self.blog_index += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_links_event(&mut self, key: KeyEvent) {
        self.handle_global_nav(&key);
    }

    fn byte_index(&self) -> usize {
        self.repl_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.repl_cursor)
            .unwrap_or(self.repl_input.len())
    }

    fn draw(&self, frame: &mut Frame) {
        // Clear background
        frame.render_widget(Clear, frame.area());

        let main_area = frame.area();

        // Layout: tabs at top, content below
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
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let titles: Vec<Line> = Page::ALL
            .iter()
            .enumerate()
            .map(|(i, p)| {
                Line::from(vec![
                    Span::styled(
                        format!("{} ", i + 1),
                        Style::default().fg(Color::Rgb(100, 200, 100)),
                    ),
                    Span::raw(p.title()),
                ])
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Rgb(80, 80, 80))
                    .title(" Grift ")
                    .title_style(Style::default().fg(Color::Rgb(100, 200, 100)).bold()),
            )
            .select(self.page.index())
            .style(Style::default().fg(Color::Rgb(180, 180, 180)))
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
            .border_style(Color::Rgb(80, 80, 80))
            .title_bottom(
                Line::from("│ built with Ratzilla │")
                    .alignment(Alignment::Right)
                    .style(Style::default().fg(Color::Rgb(80, 80, 80))),
            )
            .style(Style::default().bg(Color::Rgb(10, 14, 20)));

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
                    .border_style(Color::Rgb(60, 60, 60))
                    .title(" About ".bold().fg(Color::Rgb(100, 200, 100))),
            );
        frame.render_widget(desc, desc_area);

        // Navigation help
        let nav_text = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                "  Navigate: ".fg(Color::Rgb(120, 120, 120)),
                "Tab".fg(Color::Rgb(100, 200, 100)).bold(),
                " or ".fg(Color::Rgb(120, 120, 120)),
                "1-5".fg(Color::Rgb(100, 200, 100)).bold(),
                " to switch pages".fg(Color::Rgb(120, 120, 120)),
            ]),
            Line::from(vec![
                "  Try the ".fg(Color::Rgb(120, 120, 120)),
                "REPL".fg(Color::Rgb(100, 200, 100)).bold(),
                " (press ".fg(Color::Rgb(120, 120, 120)),
                "2".fg(Color::Rgb(100, 200, 100)).bold(),
                ") to evaluate Grift expressions interactively"
                    .fg(Color::Rgb(120, 120, 120)),
            ]),
        ]);
        frame.render_widget(
            Paragraph::new(nav_text).block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Rgb(60, 60, 60))
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
                Line::from("│ Tab: switch page │ ↑↓: scroll │ Enter: eval │")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Rgb(80, 80, 80))),
            )
            .style(Style::default().bg(Color::Rgb(10, 14, 20)));

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
                    Style::default()
                        .fg(Color::Rgb(100, 200, 100))
                        .bold(),
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

        // Calculate scroll
        let visible_height = history_area.height as usize;
        let total_lines = history_lines.len();
        let max_scroll = total_lines.saturating_sub(visible_height);
        let scroll = self.repl_scroll.min(max_scroll);

        let history = Paragraph::new(Text::from(history_lines))
            .scroll((scroll as u16, 0))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Color::Rgb(60, 60, 60))
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
        let cursor_x = input_area.x + 1 + 7 + self.repl_cursor as u16; // 1 border + "grift> "
        let cursor_y = input_area.y + 1;
        if cursor_x < input_area.right() - 1 {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }

    fn render_docs(&self, frame: &mut Frame, area: Rect) {
        let doc_content = match self.doc_page {
            0 => DOC_BASICS,
            1 => DOC_FORMS,
            2 => DOC_ADVANCED,
            _ => DOC_BASICS,
        };

        let doc_titles = ["Basics", "Forms", "Advanced"];

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(80, 80, 80))
            .title(
                format!(" Documentation: {} ", doc_titles[self.doc_page])
                    .bold()
                    .fg(Color::Rgb(100, 200, 100)),
            )
            .title_bottom(
                Line::from(format!(
                    "│ ◄ ► : navigate ({}/{}) │",
                    self.doc_page + 1,
                    doc_titles.len()
                ))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Rgb(80, 80, 80))),
            )
            .style(Style::default().bg(Color::Rgb(10, 14, 20)));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Syntax highlight doc content
        let lines: Vec<Line> = doc_content
            .lines()
            .map(|line| {
                if line.starts_with("  (") || line.starts_with("    (") {
                    // Code lines
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
                    // Indented non-code lines (atom descriptions, etc.)
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
                    // Separator lines
                    Line::styled(line, Style::default().fg(Color::Rgb(100, 200, 100)))
                } else if !line.trim().is_empty() {
                    // Section headers
                    Line::styled(
                        line,
                        Style::default()
                            .fg(Color::Rgb(100, 200, 100))
                            .bold(),
                    )
                } else {
                    Line::from("")
                }
            })
            .collect();

        let doc = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
        frame.render_widget(doc, inner);
    }

    fn render_blog(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(80, 80, 80))
            .title(" Blog ".bold().fg(Color::Rgb(100, 200, 100)))
            .title_bottom(
                Line::from("│ ↑↓ or ◄►: navigate posts │")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Rgb(80, 80, 80))),
            )
            .style(Style::default().bg(Color::Rgb(10, 14, 20)));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let [list_area, content_area] =
            Layout::horizontal([Constraint::Length(30), Constraint::Min(1)]).areas(inner);

        // Blog list
        let items: Vec<ListItem> = BLOG_ENTRIES
            .iter()
            .enumerate()
            .map(|(i, (title, date, _))| {
                let style = if i == self.blog_index {
                    Style::default()
                        .fg(Color::Rgb(100, 200, 100))
                        .bold()
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

        let list = List::new(items).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Color::Rgb(60, 60, 60))
                .title(" Posts ".fg(Color::Rgb(100, 200, 100))),
        );
        frame.render_widget(list, list_area);

        // Blog content
        if let Some((title, date, content)) = BLOG_ENTRIES.get(self.blog_index) {
            let mut lines = vec![
                Line::styled(
                    *title,
                    Style::default()
                        .fg(Color::Rgb(100, 200, 100))
                        .bold(),
                ),
                Line::styled(*date, Style::default().fg(Color::Rgb(80, 80, 80))),
                Line::styled(
                    "─".repeat(content_area.width.saturating_sub(4) as usize),
                    Style::default().fg(Color::Rgb(60, 60, 60)),
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
                        .border_style(Color::Rgb(60, 60, 60)),
                );
            frame.render_widget(blog, content_area);
        }
    }

    fn render_links(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(80, 80, 80))
            .title(" Links ".bold().fg(Color::Rgb(100, 200, 100)))
            .style(Style::default().bg(Color::Rgb(10, 14, 20)));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let [links_area, info_area] =
            Layout::vertical([Constraint::Length(LINKS.len() as u16 + 2), Constraint::Min(1)])
                .areas(inner);

        // Links with hyperlinks
        let links_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Color::Rgb(60, 60, 60))
            .title(" Repositories ".fg(Color::Rgb(100, 200, 100)));

        let links_inner = links_block.inner(links_area);
        frame.render_widget(links_block, links_area);

        for (i, (_, url)) in LINKS.iter().enumerate() {
            let link = Hyperlink::new(*url);
            let y_offset = i as i32;
            let link_area = Rect::new(links_inner.x, links_inner.y + y_offset as u16, links_inner.width, 1);
            if link_area.y < links_inner.bottom() {
                frame.render_widget(link, link_area);
            }
        }

        // Info section
        let info_text = Text::from(vec![
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
                "  Ratatui ".fg(Color::Rgb(100, 200, 100)).bold(),
                "– A Rust library for building TUI applications"
                    .fg(Color::Rgb(150, 150, 150)),
            ]),
            Line::from(""),
            Line::from(
                "  This website is entirely rendered as a terminal UI in your browser."
                    .fg(Color::Rgb(100, 100, 100)),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(info_text)
                .wrap(Wrap { trim: false })
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .border_style(Color::Rgb(60, 60, 60))
                        .title(" Info ".fg(Color::Rgb(100, 200, 100))),
                ),
            info_area,
        );
    }
}

fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let backend = DomBackend::new().expect("failed to create DOM backend");
    let terminal = Terminal::new(backend)?;

    let app = Rc::new(RefCell::new(App::new()));

    terminal.on_key_event({
        let app = app.clone();
        move |key_event| {
            app.borrow_mut().handle_event(key_event);
        }
    });

    terminal.draw_web({
        let app = app.clone();
        move |frame| {
            app.borrow().draw(frame);
        }
    });

    Ok(())
}
