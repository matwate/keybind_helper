use std::{fs, io};

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
}

const LINES_PER_SCROLL: usize = 2;

fn main() {
    let mut bindings: Vec<Binding> = Vec::new();
    // Parse my mango config file.
    const path: &str = "~/.config/mango/config.conf";

    let contents: String =
        fs::read_to_string(shellexpand::tilde(path).into_owned()).expect("No config found?");

    for line in contents.lines() {
        if line.starts_with("bind=") {
            let cut = line.trim_start_matches("bind=");
            let fields: Vec<&str> = cut.split(",").collect();
            // First two fields are the key combo, then the third elem onwards is the action
            let combo = format!("{}+{}", fields[0], fields[1]);
            let action: String = fields[2..].join(" ");
            bindings.push(Binding {
                combo: combo,
                action: action,
            });
        }
    }

    let mut app = App {
        exit: false,
        bindings: bindings,
        scroll_state: ScrollbarState::new(0).position(0),
        scroll: 0,
    };

    let _ = ratatui::run(|terminal| app.run(terminal));
}

#[derive(Debug)]
pub struct App {
    bindings: Vec<Binding>,
    exit: bool,
    scroll_state: ScrollbarState,
    scroll: usize,
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
                // Scroll up (decrease scroll offset)
                self.scroll = self.scroll.saturating_sub(LINES_PER_SCROLL);
                self.scroll_state = self.scroll_state.position(self.scroll);
            }
            KeyCode::Char('j') => {
                // Scroll down (increase scroll offset)
                self.scroll = self.scroll.saturating_add(LINES_PER_SCROLL);
                self.scroll_state = self.scroll_state.position(self.scroll);
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
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED);

        let bindings_text: Text = self
            .bindings
            .iter()
            .map(|binding| Line::from(format!("{} -> {}", binding.combo, binding.action)))
            .collect();

        Paragraph::new(bindings_text)
            .scroll((self.scroll as u16, 0))
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
