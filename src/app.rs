use std::{
    io::Read,
    time::{Duration, Instant},
};

use crate::{args::Args, entry::TOTPEntry};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{self, Constraint, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Cell, Paragraph, Row, Table, Widget},
};

pub struct App {
    path: String,
    entries: Vec<TOTPEntry>,
    selected_index: Option<usize>,
    should_quit: bool,
}

impl App {
    pub fn new(config: Args) -> Self {
        Self {
            path: config.data.clone(),
            entries: Self::load_entries(&config.data),
            should_quit: false,
            selected_index: None,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal, tick_rate: Duration) -> Result<()> {
        let mut last_tick = Instant::now();

        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        Ok(())
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

    fn save_entries(&self) {
        let mut data_file = std::fs::File::create(&self.path).expect("Cannot open --data file");
        for entry in &self.entries {
            use std::io::Write;
            writeln!(data_file, "{}", entry.dump()).expect("Cannot write to data file");
        }
    }

    fn delete_entry(&mut self) {
        if let Some(index) = self.selected_index {
            if index < self.entries.len() {
                self.entries.remove(index);
                self.selected_index = None;
            }
        }
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
            KeyCode::Char('d') | KeyCode::Delete => self.delete_entry(),
            KeyCode::Char('s') => self.save_entries(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            // KeyCode::Char('a') | KeyCode::Insert => self.add_new_entry(),
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
                ]
                .as_ref(),
            )
            .split(frame.area());
        let (header_area, content_area, footer_area) =
            (main_layout[0], main_layout[1], main_layout[2]);

        frame.render_widget(Self::get_header(), header_area);

        frame.render_widget(self.get_content(), content_area);

        frame.render_widget(Self::get_footer(), footer_area);
    }

    fn get_content(&mut self) -> impl Widget {
        let row_highlight_style = Style::default().bg(Color::White).fg(Color::Black);

        let header = ["Name", "Code"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .height(1);

        let rows = self.entries.iter().enumerate().map(|(idx, data)| {
            let name = data.name.clone();
            let code = data.current_code();
            let row = [name, code]
                .into_iter()
                .map(|content| Cell::from(Text::from(format!("{content}"))))
                .collect::<Row>()
                .height(1);

            if let Some(selected_index) = self.selected_index {
                if idx == selected_index {
                    return row.style(row_highlight_style);
                }
            }
            row
        });

        Table::new(rows, [Constraint::Length(25), Constraint::Min(10)])
            .header(header)
            .row_highlight_style(Style::default().bg(Color::Blue))
            .block(Block::bordered().title("Entries"))
    }

    fn get_footer() -> impl Widget {
        Paragraph::new("q: Quit, a: Add, d: Delete").block(Block::bordered().title("Hotkeys"))
    }

    fn get_header() -> impl Widget {
        Paragraph::new("RustOTP - Experimental TUI OTP Manager")
            .block(Block::bordered().title("Header"))
    }
}
