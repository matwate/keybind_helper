use std::{
    cmp::{self, max_by},
    fs, io,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, ScrollbarState, Widget},
};
#[derive(Debug, Default, Clone)]
pub struct Binding {
    combo: String,
    action: String,
    idx: usize,
}

const MAX_HEIGHT: usize = 64;

fn main() {
    let mut bindings: Vec<Binding> = Vec::new();
    // Parse my mango config file.
    const PATH: &str = "~/.config/mango/config.conf";

    let path_expanded = shellexpand::tilde(PATH);
    let path_str = path_expanded.as_ref();
    if !std::path::Path::new(path_str).exists() {
        eprintln!("Config file not found at: {path_str}");
        eprintln!("This tool expects a mango config file in that location.");
        std::process::exit(1);
    }
    let contents: String =
        fs::read_to_string(path_expanded.into_owned()).expect("Failed to read config");

    let mut idx = 0;
    for line in contents.lines() {
        if line.starts_with("bind=") {
            let cut = line.trim_start_matches("bind=");
            let fields: Vec<&str> = cut.split(",").collect();
            if fields.len() < 3 {
                continue;
            }
            // First two fields are the key combo, then the third elem onwards is the action
            let combo = format!("{}+{}", fields[0], fields[1]);
            let action: String = fields[2..].join(" ");

            bindings.push(Binding { combo, action, idx });
            idx += 1;
        }
    }
    if idx == 0 {
        eprintln!("No bindings found");
        std::process::exit(1);
    }

    let mut app = App {
        exit: false,
        bindings: bindings,
        scroll: 0,
        highlighted: 0,
    };

    let _ = ratatui::run(|terminal| app.run(terminal));
}

#[derive(Debug)]
pub struct App {
    bindings: Vec<Binding>,
    exit: bool,
    scroll: usize,
    highlighted: usize,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('k') => {
                self.highlighted = self.highlighted.saturating_sub(1);
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Char('j') => {
                let max_idx = self.bindings.len().saturating_sub(1);
                self.highlighted = self.highlighted.saturating_add(1).min(max_idx);
                let max_scroll = self.bindings.len().saturating_sub(1);
                self.scroll = self.scroll.saturating_add(1).min(max_scroll);
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Your Keybinds  ");

        let instructions = Line::from(vec![
            " Scroll Down ".into(),
            "<j>".blue().bold(),
            " Scroll Up ".into(),
            "<k>".blue().bold(),
            " Quit ".into(),
            "<q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED);

        let bindings_text: Text = self
            .bindings
            .iter()
            .map(|binding| {
                let line_content = format!("{} -> {}", binding.combo, binding.action);
                if binding.idx == self.highlighted {
                    Line::from(line_content)
                        .bg(ratatui::style::Color::DarkGray) // Background color
                        .fg(ratatui::style::Color::White) // Foreground color
                        .bold() // Make it bold
                } else {
                    Line::from(line_content)
                }
            })
            .collect();

        Paragraph::new(bindings_text)
            .scroll((self.scroll as u16, 0))
            .centered()
            .block(block)
            .render(area, buf);
    }
}
