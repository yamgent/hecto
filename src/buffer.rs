use std::{fs::File, io::Write, ops::Range};

use anyhow::Result;

use crate::{
    terminal::{self, TerminalPos},
    text_line::{InsertCharError, InsertCharResult, TextLine},
};

pub struct Buffer {
    content: Vec<TextLine>,
    filename: Option<String>,
    is_dirty: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            content: vec![],
            filename: None,
            is_dirty: false,
        }
    }

    pub fn new_from_file<T: AsRef<str>>(filename: T) -> Result<Self> {
        let content = std::fs::read_to_string(filename.as_ref())?;

        Ok(Self {
            content: content.lines().map(TextLine::new).collect(),
            filename: Some(filename.as_ref().to_string()),
            is_dirty: false,
        })
    }

    pub fn is_untitled_file(&self) -> bool {
        self.filename.is_none()
    }

    pub fn change_filename<T: AsRef<str>>(&mut self, filename: T) {
        self.filename = Some(filename.as_ref().to_string());
    }

    pub fn write_to_disk(&mut self) -> Result<()> {
        if let Some(filename) = &self.filename {
            let mut file = File::create(filename)?;
            self.content
                .iter()
                .map(|line| writeln!(file, "{line}"))
                .find(Result::is_err)
                .unwrap_or(Ok(()))?;
            self.is_dirty = false;
        }

        Ok(())
    }

    pub fn get_filename(&self) -> Option<String> {
        self.filename.clone()
    }

    pub fn get_is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn get_line_len(&self, line_idx: usize) -> usize {
        self.content.get(line_idx).map_or(0, TextLine::get_line_len)
    }

    pub fn get_line_text_width(&self, line_idx: usize, end_x: usize) -> u64 {
        self.content
            .get(line_idx)
            .map_or(0, |line| line.get_line_text_width(end_x))
    }

    pub fn get_total_lines(&self) -> usize {
        self.content.len()
    }

    pub fn render_line(
        &self,
        line_idx: usize,
        screen_pos: TerminalPos,
        text_offset_x: Range<u64>,
    ) -> Result<()> {
        match self.content.get(line_idx) {
            Some(line) => line.render_line(screen_pos, text_offset_x),
            None => terminal::draw_text(screen_pos, "~"),
        }
    }

    pub fn insert_character(
        &mut self,
        line_idx: usize,
        fragment_idx: usize,
        character: char,
    ) -> Result<InsertCharResult, InsertCharError> {
        if line_idx == self.content.len() {
            self.content.push(TextLine::new(character.to_string()));
            self.is_dirty = true;
            Ok(InsertCharResult {
                line_len_increased: true,
            })
        } else {
            match self.content.get_mut(line_idx) {
                Some(line) => {
                    let insert_result = line.insert_character(fragment_idx, character);
                    self.is_dirty = insert_result.is_ok();
                    insert_result
                }
                None => Err(InsertCharError::InvalidPosition),
            }
        }
    }

    pub fn remove_character(&mut self, line_idx: usize, fragment_idx: usize) {
        if let Some(line) = self.content.get_mut(line_idx) {
            line.remove_character(fragment_idx);
            self.is_dirty = true;
        }
    }

    pub fn join_line_with_below_line(&mut self, line_idx: usize) {
        let mut new_line_string = None;

        if let Some(first_line) = self.content.get(line_idx) {
            if let Some(second_line) = self.content.get(line_idx.saturating_add(1)) {
                let mut final_string = first_line.to_string();
                final_string.push_str(&second_line.to_string());
                new_line_string = Some(final_string);
            }
        }

        if let Some(new_line) = new_line_string {
            *self
                .content
                .get_mut(line_idx)
                .expect("line_idx to exist as new_line_string contains line_idx") =
                TextLine::new(new_line);
            self.content.remove(line_idx.saturating_add(1));
            self.is_dirty = true;
        }
    }

    pub fn insert_newline_at(&mut self, line_idx: usize, fragment_idx: usize) {
        assert!(line_idx <= self.content.len());

        match self.content.get_mut(line_idx) {
            Some(line) => {
                let new_line = line.split_off(fragment_idx);
                self.content.insert(line_idx.saturating_add(1), new_line);
            }
            None => {
                self.content.push(TextLine::new(""));
            }
        }

        self.is_dirty = true;
    }
}
