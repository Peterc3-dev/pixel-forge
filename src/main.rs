mod canvas;
mod color;

use std::io;
use std::time::Duration;

use canvas::Canvas;
use color::{default_palette, PixelColor};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

const GREEN: Color = Color::Rgb(0, 255, 0);
const DIM_GREEN: Color = Color::Rgb(0, 100, 0);
const CHECKER_A: Color = Color::Rgb(30, 30, 30);
const CHECKER_B: Color = Color::Rgb(50, 50, 50);

#[derive(PartialEq, Clone, Copy)]
enum Tool {
    Pencil,
    Eraser,
    Fill,
    Line,
    Rect,
}

impl Tool {
    fn name(&self) -> &str {
        match self {
            Tool::Pencil => "Pencil",
            Tool::Eraser => "Eraser",
            Tool::Fill => "Fill",
            Tool::Line => "Line",
            Tool::Rect => "Rect",
        }
    }
}

struct App {
    canvas: Canvas,
    cursor_x: usize,
    cursor_y: usize,
    palette: Vec<PixelColor>,
    color_idx: usize,
    tool: Tool,
    undo_stack: Vec<Vec<(usize, usize, Option<PixelColor>)>>,
}

impl App {
    fn new(w: usize, h: usize) -> Self {
        Self {
            canvas: Canvas::new(w, h),
            cursor_x: w / 2,
            cursor_y: h / 2,
            palette: default_palette(),
            color_idx: 10, // bright green
            tool: Tool::Pencil,
            undo_stack: Vec::new(),
        }
    }

    fn current_color(&self) -> PixelColor {
        self.palette[self.color_idx]
    }

    fn apply_tool(&mut self) {
        match self.tool {
            Tool::Pencil => {
                let old = self.canvas.get(self.cursor_x, self.cursor_y);
                self.canvas.set(self.cursor_x, self.cursor_y, Some(self.current_color()));
                self.undo_stack.push(vec![(self.cursor_x, self.cursor_y, old)]);
            }
            Tool::Eraser => {
                let old = self.canvas.get(self.cursor_x, self.cursor_y);
                self.canvas.set(self.cursor_x, self.cursor_y, None);
                self.undo_stack.push(vec![(self.cursor_x, self.cursor_y, old)]);
            }
            Tool::Fill => {
                let changes = self.canvas.flood_fill(
                    self.cursor_x,
                    self.cursor_y,
                    Some(self.current_color()),
                );
                if !changes.is_empty() {
                    self.undo_stack.push(changes);
                }
            }
            Tool::Line | Tool::Rect => {
                let old = self.canvas.get(self.cursor_x, self.cursor_y);
                self.canvas.set(self.cursor_x, self.cursor_y, Some(self.current_color()));
                self.undo_stack.push(vec![(self.cursor_x, self.cursor_y, old)]);
            }
        }
    }

    fn undo(&mut self) {
        if let Some(changes) = self.undo_stack.pop() {
            for (x, y, old) in changes {
                self.canvas.set(x, y, old);
            }
        }
    }
}

fn main() -> io::Result<()> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(32, 24);
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(34), Constraint::Length(20)])
                .split(f.area());

            draw_canvas(f, chunks[0], app);
            draw_sidebar(f, chunks[1], app);
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        app.cursor_x = app.cursor_x.saturating_sub(1);
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if app.cursor_x + 1 < app.canvas.width {
                            app.cursor_x += 1;
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.cursor_y = app.cursor_y.saturating_sub(1);
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        if app.cursor_y + 1 < app.canvas.height {
                            app.cursor_y += 1;
                        }
                    }
                    KeyCode::Char(' ') | KeyCode::Enter => app.apply_tool(),
                    KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => app.undo(),
                    KeyCode::Char('u') => app.undo(),
                    KeyCode::Char('p') => app.tool = Tool::Pencil,
                    KeyCode::Char('e') => app.tool = Tool::Eraser,
                    KeyCode::Char('f') => app.tool = Tool::Fill,
                    KeyCode::Char('[') => {
                        if app.color_idx > 0 {
                            app.color_idx -= 1;
                        }
                    }
                    KeyCode::Char(']') => {
                        if app.color_idx + 1 < app.palette.len() {
                            app.color_idx += 1;
                        }
                    }
                    KeyCode::Char('c') => {
                        app.canvas = Canvas::new(app.canvas.width, app.canvas.height);
                        app.undo_stack.clear();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn draw_canvas(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM_GREEN))
        .title(Span::styled(
            format!("pixel-forge {}x{}", app.canvas.width, app.canvas.height),
            Style::default().fg(GREEN),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Each pixel = 2 chars wide ("██"), using half-block for 2 rows per terminal row
    let mut lines = Vec::new();
    let canvas_rows = (app.canvas.height + 1) / 2;
    for row in 0..canvas_rows {
        if row >= inner.height as usize {
            break;
        }
        let y_top = row * 2;
        let y_bot = row * 2 + 1;
        let mut spans = Vec::new();
        for x in 0..app.canvas.width {
            if x * 2 >= inner.width as usize {
                break;
            }
            let is_cursor_top = x == app.cursor_x && y_top == app.cursor_y;
            let is_cursor_bot = y_bot < app.canvas.height && x == app.cursor_x && y_bot == app.cursor_y;

            let top_px = app.canvas.get(x, y_top);
            let bot_px = if y_bot < app.canvas.height {
                app.canvas.get(x, y_bot)
            } else {
                None
            };

            let checker = if (x + y_top) % 2 == 0 { CHECKER_A } else { CHECKER_B };
            let checker_bot = if (x + y_bot) % 2 == 0 { CHECKER_A } else { CHECKER_B };

            let fg_color = top_px.map(|c| c.to_ratatui()).unwrap_or(checker);
            let bg_color = bot_px.map(|c| c.to_ratatui()).unwrap_or(checker_bot);

            if is_cursor_top || is_cursor_bot {
                spans.push(Span::styled("▀▀", Style::default().fg(GREEN).bg(GREEN)));
            } else {
                spans.push(Span::styled("▀▀", Style::default().fg(fg_color).bg(bg_color)));
            }
        }
        lines.push(Line::from(spans));
    }
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_sidebar(f: &mut ratatui::Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(12),
            Constraint::Min(3),
        ])
        .split(area);

    // Tool info
    let tool_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM_GREEN))
        .title(Span::styled("Tool", Style::default().fg(GREEN)));
    let c = app.current_color();
    let tool_text = vec![
        Line::from(Span::styled(
            app.tool.name(),
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("Color: ", Style::default().fg(DIM_GREEN)),
            Span::styled("██", Style::default().fg(Color::Rgb(c.r, c.g, c.b))),
        ]),
        Line::from(Span::styled(
            format!("({},{}) ", app.cursor_x, app.cursor_y),
            Style::default().fg(DIM_GREEN),
        )),
    ];
    f.render_widget(Paragraph::new(tool_text).block(tool_block), chunks[0]);

    // Palette
    let pal_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM_GREEN))
        .title(Span::styled("Palette", Style::default().fg(GREEN)));
    let inner = pal_block.inner(chunks[1]);
    f.render_widget(pal_block, chunks[1]);

    let cols = (inner.width / 2) as usize;
    if cols > 0 {
        let mut pal_lines = Vec::new();
        for row_start in (0..app.palette.len()).step_by(cols) {
            let mut spans = Vec::new();
            for i in row_start..(row_start + cols).min(app.palette.len()) {
                let pc = &app.palette[i];
                let style = if i == app.color_idx {
                    Style::default().fg(Color::Black).bg(Color::Rgb(pc.r, pc.g, pc.b))
                } else {
                    Style::default().fg(Color::Rgb(pc.r, pc.g, pc.b))
                };
                spans.push(Span::styled("██", style));
            }
            pal_lines.push(Line::from(spans));
        }
        f.render_widget(Paragraph::new(pal_lines), inner);
    }

    // Keys
    let keys_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM_GREEN))
        .title(Span::styled("Keys", Style::default().fg(GREEN)));
    let keys = vec![
        Line::from(Span::styled("hjkl  move", Style::default().fg(DIM_GREEN))),
        Line::from(Span::styled("Space draw", Style::default().fg(DIM_GREEN))),
        Line::from(Span::styled("p/e/f tool", Style::default().fg(DIM_GREEN))),
        Line::from(Span::styled("[/]   color", Style::default().fg(DIM_GREEN))),
        Line::from(Span::styled("u     undo", Style::default().fg(DIM_GREEN))),
        Line::from(Span::styled("c     clear", Style::default().fg(DIM_GREEN))),
        Line::from(Span::styled("q     quit", Style::default().fg(DIM_GREEN))),
    ];
    f.render_widget(Paragraph::new(keys).block(keys_block), chunks[2]);
}
