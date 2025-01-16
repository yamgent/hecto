use anyhow::Result;

use crate::{
    buffer::Buffer,
    math::Pos2u,
    terminal::{self, TerminalPos},
};

pub struct View {
    buffer: Buffer,
    size: Pos2u,

    cursor_pos: Pos2u,
    scroll_offset: Pos2u,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewCommand {
    MoveCursorUp,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorToTopOfBuffer,
    MoveCursorToBottomOfBuffer,
    MoveCursorToStartOfLine,
    MoveCursorToEndOfLine,
}

impl View {
    pub fn new(size: Pos2u) -> Self {
        Self {
            buffer: Buffer::new(),
            size,
            cursor_pos: Pos2u::ZERO,
            scroll_offset: Pos2u::ZERO,
        }
    }

    pub fn new_with_buffer(buffer: Buffer, size: Pos2u) -> Self {
        Self {
            buffer,
            size,
            cursor_pos: Pos2u::ZERO,
            scroll_offset: Pos2u::ZERO,
        }
    }

    pub fn resize(&mut self, size: Pos2u) {
        self.size = size;
        self.adjust_scroll_to_cursor_pos();
    }

    pub fn render(&self) -> Result<TerminalPos> {
        self.buffer
            .content
            .iter()
            .skip(self.scroll_offset.y as usize)
            .take(self.size.y as usize)
            .enumerate()
            .map(|(y, line)| {
                terminal::draw_text(
                    TerminalPos {
                        x: 0,
                        // y could not be bigger than size.y, which is u16
                        #[allow(clippy::cast_possible_truncation)]
                        y: y as u16,
                    },
                    line.chars()
                        .skip(self.scroll_offset.x as usize)
                        .take(self.size.x as usize)
                        .collect::<String>(),
                )
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        (self.buffer.content.len()..(self.size.y as usize))
            .map(|y| {
                terminal::draw_text(
                    TerminalPos {
                        x: 0,
                        // y could not be bigger than size.y, which is u16
                        #[allow(clippy::cast_possible_truncation)]
                        y: y as u16,
                    },
                    "~",
                )
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))?;

        Ok(TerminalPos {
            x: (self.cursor_pos.x - self.scroll_offset.x) as u16,
            y: (self.cursor_pos.y - self.scroll_offset.y) as u16,
        })
    }

    fn adjust_scroll_to_cursor_pos(&mut self) {
        if self.cursor_pos.x < self.scroll_offset.x {
            self.scroll_offset.x = self.cursor_pos.x;
        }

        if self.cursor_pos.y < self.scroll_offset.y {
            self.scroll_offset.y = self.cursor_pos.y;
        }

        if self.cursor_pos.x >= self.scroll_offset.x + self.size.x {
            self.scroll_offset.x = self.cursor_pos.x - self.size.x + 1;
        }

        if self.cursor_pos.y >= self.scroll_offset.y + self.size.y {
            self.scroll_offset.y = self.cursor_pos.y - self.size.y + 1;
        }
    }

    pub fn execute_command(&mut self, command: ViewCommand) {
        match command {
            ViewCommand::MoveCursorUp => {
                if self.cursor_pos.y > 0 {
                    self.cursor_pos.y -= 1;
                }
                self.adjust_scroll_to_cursor_pos();
            }
            ViewCommand::MoveCursorDown => {
                self.cursor_pos.y += 1;
                self.adjust_scroll_to_cursor_pos();
            }
            ViewCommand::MoveCursorLeft => {
                if self.cursor_pos.x > 0 {
                    self.cursor_pos.x -= 1;
                }
                self.adjust_scroll_to_cursor_pos();
            }
            ViewCommand::MoveCursorRight => {
                self.cursor_pos.x += 1;
                self.adjust_scroll_to_cursor_pos();
            }
            ViewCommand::MoveCursorToTopOfBuffer => {
                self.cursor_pos.y = 0;
                self.adjust_scroll_to_cursor_pos();
            }
            ViewCommand::MoveCursorToBottomOfBuffer => {
                self.cursor_pos.y = self.buffer.content.len() as u64;
                self.adjust_scroll_to_cursor_pos();
            }
            ViewCommand::MoveCursorToStartOfLine => {
                self.cursor_pos.x = 0;
                self.adjust_scroll_to_cursor_pos();
            }
            ViewCommand::MoveCursorToEndOfLine => {
                self.cursor_pos.x =
                    if let Some(line) = self.buffer.content.get(self.cursor_pos.y as usize) {
                        (line.chars().count().saturating_sub(1)) as u64
                    } else {
                        0
                    };
                self.adjust_scroll_to_cursor_pos();
            }
        }
    }
}
