use std::{
    io::Read,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::entry::TOTPEntry;

use arboard::Clipboard;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{self, Constraint, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Cell, Gauge, Paragraph, Row, Table, Widget},
};

pub struct App {
    entries: Vec<TOTPEntry>,
    selected_index: Option<usize>,
    should_quit: bool,
    clipboard: Option<Arc<Mutex<Clipboard>>>,
}

impl App {
    pub fn new() -> Self {
        let data_file = Self::init_file();
        println!("Using data file: {}", data_file);
        let entries = Self::load_entries(&data_file);
        let selected_index = if entries.is_empty() { None } else { Some(0) };
        let clipboard = match Clipboard::new() {
            Ok(c) => Some(Arc::new(Mutex::new(c))),
            Err(_) => None,
        };
        Self {
            entries,
            selected_index,
            should_quit: false,
            clipboard,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal, tick_rate: Duration) -> Result<()> {
        let mut last_tick = Instant::now();

        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));

            if event::poll(timeout)?
                && let Event::Key(key) = event::read()?
            {
                self.handle_key(key);
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn init_file() -> String {
        let mut home_dir = dirs::data_dir().expect("Cannot find data directory");
        home_dir.push("rustotp.txt");
        if !home_dir.exists() {
            std::fs::File::create(&home_dir)
                .unwrap_or_else(|_| panic!("Cannot create file '{}'", home_dir.display()));
        }
        home_dir.display().to_string()
    }

    fn load_entries(file: &str) -> Vec<TOTPEntry> {
        let mut data_file = std::fs::File::open(file).expect("Cannot open --data file");
        let mut content: String = String::new();
        data_file
            .read_to_string(&mut content)
            .expect("Cannot read content of --data file");

        let mut entries: Vec<TOTPEntry> = Vec::new();

        for line in content.lines() {
            let entry = TOTPEntry::parse(line);
            entries.push(entry);
        }

        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }

    fn move_up(&mut self) {
        if let Some(index) = self.selected_index {
            if index > 0 {
                self.selected_index = Some(index - 1);
            }
        } else if !self.entries.is_empty() {
            self.selected_index = Some(0);
        }
    }

    fn move_down(&mut self) {
        if let Some(index) = self.selected_index {
            if index + 1 < self.entries.len() {
                self.selected_index = Some(index + 1);
            }
        } else if !self.entries.is_empty() {
            self.selected_index = Some(0);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Char('c') => {
                if let Some(index) = self.selected_index {
                    let entry = &self.entries[index];
                    let code = entry.current_code();
                    self.copy_to_clipboard(&code);
                }
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let main_layout = Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(frame.area());
        let (header_area, content_area, progress_area, footer_area) = (
            main_layout[0],
            main_layout[1],
            main_layout[2],
            main_layout[3],
        );

        frame.render_widget(Self::get_header(), header_area);
        frame.render_widget(self.get_progress(), progress_area);
        frame.render_widget(self.get_content(), content_area);
        frame.render_widget(Self::get_footer(), footer_area);
    }

    fn get_content(&mut self) -> impl Widget {
        let (_, remaining) = self.get_percent_remaining().unwrap_or((100, 30));

        let header = ["Name", "Code"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .height(1);

        let rows = self.entries.iter().enumerate().map(|(idx, data)| {
            let is_selected = match self.selected_index {
                Some(selected_index) => idx == selected_index,
                None => false,
            };
            let name = data.name.clone();
            let code = data.current_code();
            let name = Cell::from(Text::from(name));
            let code = Cell::from(Text::from(code));
            let row = Row::new([name, code]);

            row.style(Self::get_cell_style(remaining, is_selected))
        });

        Table::new(rows, [Constraint::Length(25), Constraint::Min(10)])
            .header(header)
            .row_highlight_style(Style::default().bg(Color::Blue))
            .block(Block::bordered().title("TOTP codes"))
    }

    fn get_cell_style(remaining: u16, is_selected: bool) -> Style {
        let mut style = Style::default();

        if is_selected && remaining <= 5 {
            return style.bg(Color::Yellow).fg(Color::Black);
        }

        if is_selected {
            style = style.bg(Color::White).fg(Color::Black);
        }

        if remaining <= 5 {
            style = style.bg(Color::Black).fg(Color::Yellow);
        }

        style
    }

    fn get_footer() -> impl Widget {
        Paragraph::new("q: Quit, c: Copy code").block(Block::bordered().title("Hotkeys"))
    }

    fn get_header() -> impl Widget {
        Paragraph::new("RustOTP - TUI OTP Viewer").block(Block::bordered())
    }

    fn get_progress(&self) -> impl Widget {
        let (percent, remaining) = match self.get_percent_remaining() {
            Some((p, r)) => (p, r),
            None => (100, 30),
        };

        Gauge::default()
            .block(Block::bordered())
            .percent(percent)
            .label(format!("{remaining}s"))
    }

    fn get_percent_remaining(&self) -> Option<(u16, u16)> {
        if let Some(first_entry) = self.entries.first() {
            let remaining = first_entry.remaining_seconds();
            let percent = ((30 - remaining) as f64 / 30.0 * 100.0).round() as u16;
            Some((100 - percent, remaining))
        } else {
            None
        }
    }

    fn copy_to_clipboard(&mut self, text: &str) {
        if let Some(clipboard) = &self.clipboard
            && let Ok(mut cb) = clipboard.lock()
        {
            let _ = cb.set_text(text.to_string());
        }
    }
}
