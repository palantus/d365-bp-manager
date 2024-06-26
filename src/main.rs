#![allow(clippy::enum_glob_use, clippy::wildcard_imports)]

use std::{cmp, error::Error, io};

use config::{read_config, Config};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use style::palette::tailwind;

mod read;
use read::{read_diagnostics, Diagnostic};
mod write;
use write::write_diagnostics;
mod config;

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];
const INFO_TEXT_NORMAL: &str =
    "(Enter) enter justification | (q) quit | (↑) move up | (↓) move down | (w) write file | (Esc) switch model";
const INFO_TEXT_JUSTIFICATION: &str = "(Enter) go back";
const INFO_TEXT_ERROR: &str = "(Esc) quit";
const INFO_TEXT_MODELSELECT: &str = "(Enter) select | (Esc) quit";

const ITEM_HEIGHT: usize = 4;

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

impl Diagnostic {
    const fn ref_array(&self) -> [&String; 3] {
        [&self.Moniker, &self.Severity, &self.Path]
    }

    fn info(&self) -> String {
        format!("Message: {}  --  Path: {}", &self.Message, &self.Path)
    }
}

enum InputMode {
    Normal,
    Justification,
    ModelSelect,
    Error,
}

struct App {
    state: TableState,
    items: Vec<Diagnostic>,
    scroll_state: ScrollbarState,
    colors: TableColors,
    color_index: usize,
    mode: InputMode,
    model: String,
    config: Config,
    error_message: String,
}

impl App {
    fn new(data_vec: Vec<Diagnostic>, config: Config, model: String) -> Self {
        Self {
            state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new((cmp::max(data_vec.len(), 1) - 1) * ITEM_HEIGHT),
            colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            items: data_vec,
            mode: InputMode::ModelSelect,
            model,
            config,
            error_message: String::new(),
        }
    }

    pub fn next(&mut self, count: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous(&mut self, count: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % PALETTES.len();
    }

    pub fn previous_color(&mut self) {
        let count = PALETTES.len();
        self.color_index = (self.color_index + count - 1) % count;
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    pub fn get_selected(&self) -> Option<&Diagnostic> {
        if self.items.len() < 1 {
            return None;
        }
        match self.state.selected() {
            Some(idx) => {
                if idx >= self.items.len() {
                    return None;
                }
                Some(&self.items[idx])
            }
            None => None,
        }
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut Diagnostic> {
        if self.items.len() < 1 {
            return None;
        }
        match self.state.selected() {
            Some(idx) => Some(self.items.get_mut(idx).unwrap()),
            None => None,
        }
    }

    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    fn write_file(&self) -> Result<(), String> {
        write_diagnostics(&self.items, &self.config, &self.model)
    }

    pub fn set_error(&mut self, error_message: String) {
        self.set_mode(InputMode::Error);
        self.error_message = error_message;
    }

    pub fn set_model(&mut self, model: String) {
        self.model = model;
        match read_diagnostics(&self.config, &self.model) {
            Ok(data) => {
                self.items = data
                    .into_iter()
                    .filter(|d| d.Severity != "Informational")
                    .collect();
                self.state.select(Some(0))
            }
            Err(e) => self.set_error(e),
        }
    }

    pub fn get_selected_model(&self) -> Option<&String> {
        if self.config.models.len() < 1 {
            return None;
        }
        Some(&self.config.models[self.state.selected().unwrap()])
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = match read_config() {
        Ok(config) => App::new(vec![], config, String::new()),
        Err(e) => {
            let mut app = App::new(vec![], Config::default(), String::new());
            app.set_error(e);
            app
        }
    };
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                use KeyCode::*;
                match app.mode {
                    InputMode::Normal => match key.code {
                        Char('q') => return Ok(()),
                        Char('j') | Down => app.next(app.items.len()),
                        Char('k') | Up => app.previous(app.items.len()),
                        Char('l') | Right => app.next_color(),
                        Char('h') | Left => app.previous_color(),
                        Char('w') => match app.write_file() {
                            Err(e) => app.set_error(e),
                            _ => {}
                        },
                        Enter => app.set_mode(InputMode::Justification),
                        Char('m') | Esc => app.set_mode(InputMode::ModelSelect),
                        _ => {}
                    },

                    InputMode::Justification => match key.code {
                        Esc | Enter => app.set_mode(InputMode::Normal),
                        Char(c) => match app.get_selected_mut() {
                            Some(s) => s.Justification.push(c),
                            None => {}
                        },
                        Backspace => match app.get_selected_mut() {
                            Some(s) => {
                                s.Justification.pop();
                            }
                            None => {}
                        },
                        _ => {}
                    },
                    InputMode::Error => match key.code {
                        Char('q') | Esc => return Ok(()),
                        _ => {}
                    },
                    InputMode::ModelSelect => match key.code {
                        Char('q') | Esc => return Ok(()),
                        Char('j') | Down => app.next(app.config.models.len()),
                        Char('k') | Up => app.previous(app.config.models.len()),
                        Char('l') | Right => app.next_color(),
                        Char('h') | Left => app.previous_color(),
                        Enter => match app.get_selected_model() {
                            Some(m) => {
                                app.set_model(m.clone());
                                app.set_mode(InputMode::Normal)
                            }
                            None => (),
                        },
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    app.set_colors();
    match app.mode {
        InputMode::Error => {
            let layout = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(f.size());

            render_error(f, app, layout[0]);
            render_footer(f, app, layout[2]);
        }
        InputMode::ModelSelect => {
            let rects =
                Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(f.size());

            render_models(f, app, rects[0]);
            render_scrollbar(f, app, rects[0]);
            render_footer(f, app, rects[1]);
        }
        _ => {
            let rects = Layout::vertical([
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(f.size());

            render_bp(f, app, rects[0]);
            render_scrollbar(f, app, rects[0]);
            render_justification(f, app, rects[1]);
            render_cur_details(f, app, rects[2]);
            render_footer(f, app, rects[3]);
        }
    }
}

fn render_bp(f: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default()
        .fg(app.colors.header_fg)
        .bg(app.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(app.colors.selected_style_fg);

    let header = ["Moniker", "Severity", "Path"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
    let rows = app.items.iter().enumerate().map(|(i, data)| {
        let color = match i % 2 {
            0 => app.colors.normal_row_color,
            _ => app.colors.alt_row_color,
        };
        let item = data.ref_array();
        item.into_iter()
            .map(|content| Cell::from(Text::from(format!("{content}"))))
            .collect::<Row>()
            .style(Style::new().fg(app.colors.row_fg).bg(color))
            .height(1)
    });
    let bar = " █ ";
    let t = Table::new(
        rows,
        [
            // + 1 is for padding.
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .highlight_style(selected_style)
    .highlight_symbol(Text::from(vec![
        "".into(),
        bar.into(),
        bar.into(),
        "".into(),
    ]))
    .bg(app.colors.buffer_bg)
    .highlight_spacing(HighlightSpacing::Always);
    f.render_stateful_widget(t, area, &mut app.state);
}

fn render_scrollbar(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let info_footer = Paragraph::new(Line::from(match app.mode {
        InputMode::Justification => INFO_TEXT_JUSTIFICATION,
        InputMode::Error => INFO_TEXT_ERROR,
        InputMode::ModelSelect => INFO_TEXT_MODELSELECT,
        _ => INFO_TEXT_NORMAL,
    }))
    .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
    .centered()
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(app.colors.footer_border_color))
            .border_type(BorderType::Double),
    );
    f.render_widget(info_footer, area);
}

fn render_cur_details(f: &mut Frame, app: &App, area: Rect) {
    let info = match app.get_selected() {
        Some(s) => s.info().clone(),
        None => "None selected".to_owned(),
    };

    let info_footer = Paragraph::new(Line::from(info))
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        // .centered()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(app.colors.footer_border_color))
                .border_type(BorderType::Double),
        );
    f.render_widget(info_footer, area);
}

fn render_justification(f: &mut Frame, app: &App, area: Rect) {
    let text = format!(
        "Justification: {}",
        match app.get_selected() {
            Some(s) => s.Justification.clone(),
            None => "".to_owned(),
        }
    );
    let info_footer = Paragraph::new(Line::from(text))
        .style(match app.mode {
            InputMode::Justification => Style::default().fg(Color::Yellow),
            _ => Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg),
        })
        // .centered()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(app.colors.footer_border_color))
                .border_type(BorderType::Double),
        );
    f.render_widget(info_footer, area);
}

fn render_error(f: &mut Frame, app: &App, area: Rect) {
    let info_footer = Paragraph::new(Line::from(app.error_message.clone()))
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .centered()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(app.colors.footer_border_color))
                .border_type(BorderType::Double),
        );
    f.render_widget(info_footer, area);
}

fn render_models(f: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default()
        .fg(app.colors.header_fg)
        .bg(app.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(app.colors.selected_style_fg);

    let header = ["Name"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
    let rows = app.config.models.iter().enumerate().map(|(i, model)| {
        let color = match i % 2 {
            0 => app.colors.normal_row_color,
            _ => app.colors.alt_row_color,
        };
        let item = [model.clone()];
        item.into_iter()
            .map(|content| Cell::from(Text::from(format!("{content}"))))
            .collect::<Row>()
            .style(Style::new().fg(app.colors.row_fg).bg(color))
            .height(1)
    });
    let bar = " █ ";
    let t = Table::new(rows, [Constraint::Min(20)])
        .header(header)
        .highlight_style(selected_style)
        .highlight_symbol(Text::from(vec![
            "".into(),
            bar.into(),
            bar.into(),
            "".into(),
        ]))
        .bg(app.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always);
    f.render_stateful_widget(t, area, &mut app.state);
}
