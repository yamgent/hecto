use std::io::{self, Write};

use anyhow::Result;
use crossterm::{cursor, queue, style, terminal};

use crate::math::Pos2u;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub x: u16,
    pub y: u16,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct TerminalPos {
    pub x: u16,
    pub y: u16,
}

pub struct TerminalRestoreState {
    pub cursor_pos: TerminalPos,
}

pub fn init_terminal() -> Result<()> {
    terminal::enable_raw_mode()?;

    queue!(io::stdout(), terminal::EnterAlternateScreen)?;
    io::stdout().flush()?;

    Ok(())
}

pub fn end_terminal() -> Result<()> {
    queue!(io::stdout(), terminal::LeaveAlternateScreen)?;
    io::stdout().flush()?;

    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn start_draw() -> Result<TerminalRestoreState> {
    queue!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
    hide_cursor()?;

    Ok(TerminalRestoreState {
        cursor_pos: get_cursor_pos()?,
    })
}

pub fn end_draw(restore_state: &TerminalRestoreState) -> Result<()> {
    move_cursor(restore_state.cursor_pos)?;
    show_cursor()?;

    io::stdout().flush()?;
    Ok(())
}

pub fn size() -> Result<TerminalSize> {
    let size = terminal::size()?;
    Ok(TerminalSize {
        x: size.0,
        y: size.1,
    })
}

pub fn size_u64() -> Result<Pos2u> {
    let size = terminal::size()?;
    Ok(Pos2u {
        x: size.0.into(),
        y: size.1.into(),
    })
}

pub fn hide_cursor() -> Result<()> {
    queue!(io::stdout(), cursor::Hide)?;
    Ok(())
}

fn show_cursor() -> Result<()> {
    queue!(io::stdout(), cursor::Show)?;
    Ok(())
}

fn move_cursor(pos: TerminalPos) -> Result<()> {
    queue!(io::stdout(), cursor::MoveTo(pos.x, pos.y))?;
    Ok(())
}

pub fn draw_text<T: AsRef<str>>(pos: TerminalPos, text: T) -> Result<()> {
    move_cursor(pos)?;
    queue!(io::stdout(), style::Print(text.as_ref()))?;
    Ok(())
}

fn get_cursor_pos() -> Result<TerminalPos> {
    let pos = cursor::position()?;
    Ok(TerminalPos { x: pos.0, y: pos.1 })
}
