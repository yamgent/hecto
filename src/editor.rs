use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
    buffer::Buffer,
    terminal::{self, TerminalPos},
    view::View,
};

pub struct Editor {
    should_quit: bool,

    cursor_pos: TerminalPos,
    view: View,
    status_bar_text: String,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            cursor_pos: TerminalPos { x: 0, y: 0 },
            view: View::new(),
            status_bar_text: String::new(),
        }
    }

    pub fn run(&mut self) {
        terminal::init_terminal().expect("able to initialize terminal");

        self.open_arg_file();

        let repl_result = self.repl();

        terminal::end_terminal().expect("able to deinit terminal");
        repl_result.expect("repl has no fatal error");
    }

    fn open_arg_file(&mut self) {
        if let Some(filename) = std::env::args().nth(1) {
            let buffer = match Buffer::new_from_file(&filename) {
                Ok(buffer) => buffer,
                Err(err) => {
                    self.status_bar_text = format!("Cannot load {filename}: {err}");
                    return;
                }
            };
            self.view = View::new_with_buffer(buffer);
        }
    }

    fn repl(&mut self) -> Result<()> {
        while !self.should_quit {
            let event = event::read()?;
            self.handle_event(&event)?;
            self.draw()?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            let size = terminal::size()?;

            match (code, modifiers) {
                (&KeyCode::Char('q'), &KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                (&KeyCode::Left, _) => {
                    if self.cursor_pos.x > 0 {
                        self.cursor_pos.x -= 1;
                    }
                }
                (&KeyCode::Right, _) => {
                    if self.cursor_pos.x < size.x - 1 {
                        self.cursor_pos.x += 1;
                    }
                }
                (&KeyCode::Up, _) => {
                    if self.cursor_pos.y > 0 {
                        self.cursor_pos.y -= 1;
                    }
                }
                (&KeyCode::Down, _) => {
                    if self.cursor_pos.y < size.y - 1 {
                        self.cursor_pos.y += 1;
                    }
                }
                (&KeyCode::Home, _) => {
                    self.cursor_pos.x = 0;
                }
                (&KeyCode::End, _) => {
                    self.cursor_pos.x = size.x - 1;
                }
                (&KeyCode::PageUp, _) => {
                    self.cursor_pos.y = 0;
                }
                (&KeyCode::PageDown, _) => {
                    self.cursor_pos.y = size.y - 1;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn draw(&self) -> Result<()> {
        let mut state = terminal::start_draw()?;

        self.view.render()?;

        if !self.status_bar_text.is_empty() {
            let size = terminal::size()?;
            terminal::draw_text(
                TerminalPos {
                    x: 0,
                    y: size.y - 1,
                },
                &self.status_bar_text,
            )?;
        }

        state.cursor_pos = self.cursor_pos;

        terminal::end_draw(&state)?;
        Ok(())
    }
}
