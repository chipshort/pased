use std::{path::PathBuf, str::FromStr};

use eyre::{eyre, Context, Result};
use itertools::Itertools;
use regex::Regex;

use crate::RustLevel;

/// A position matcher takes input from stdin and produces positions in files from that input.
pub trait PositionMatcher {
    /// Find positions in the input.
    fn find_positions(&self, input: &str) -> Result<Vec<FilePosition>>;
}

/// A position in a file.
pub struct FilePosition {
    /// The file path.
    pub path: PathBuf,
    /// The position inside the file.
    pub position: Position,
}

/// A position in a file.
/// Line and column are 1-based.
pub enum Position {
    Line(u32),
    LineColumn(u32, u32),
    // Byte(u32),
}

/// A position in a file in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BytePosition(pub usize);

impl Position {
    pub fn lines_around(&self, lines: u32) -> (Position, Position) {
        let start = match self {
            Position::Line(line) | Position::LineColumn(line, _) => {
                Position::Line(line.saturating_sub(lines).max(1))
            }
        };
        let end = match self {
            Position::Line(line) | Position::LineColumn(line, _) => Position::Line(line + lines),
        };

        (start, end)
    }

    pub fn to_byte_position(&self, file_content: &str) -> BytePosition {
        let (line, col) = match self {
            Position::Line(line) => (*line, 1),
            Position::LineColumn(line, column) => (*line, *column),
        };
        let line_slice = file_content
            .lines()
            .nth(line as usize - 1)
            .map(|line| &line[col as usize - 1..])
            .unwrap_or_else(|| file_content.lines().next().unwrap());

        BytePosition(file_content.subslice_offset(line_slice).unwrap())
    }
}

pub struct RustPositionMatcher {
    level: RustLevel,
}

impl RustPositionMatcher {
    pub fn new(level: RustLevel) -> Self {
        Self { level }
    }
}

impl PositionMatcher for RustPositionMatcher {
    fn find_positions(&self, input: &str) -> Result<Vec<FilePosition>> {
        input
            .lines()
            .tuple_windows()
            .filter(|(line1, _)| self.level.matches(line1))
            .map(|(_, line2)| -> Result<_> {
                // parse the position from line2
                let file_position_regex = Regex::new(r"--> (.+\.rs):(\d+):(\d+)").unwrap();
                let captures = file_position_regex
                    .captures(line2)
                    .ok_or_else(|| eyre!("failed to detect file position: {line2}"))?;
                let path = PathBuf::from_str(captures.get(1).unwrap().as_str())
                    .wrap_err("error parsing path")?;
                let line = u32::from_str(captures.get(2).unwrap().as_str()).unwrap();
                let column = u32::from_str(captures.get(3).unwrap().as_str()).unwrap();

                Ok(FilePosition {
                    path,
                    position: Position::LineColumn(line, column),
                })
            })
            .collect()
    }
}

pub struct RegexPositionMatcher {
    regex: Regex,
}

impl RegexPositionMatcher {
    pub fn new(regex: &str) -> Result<Self> {
        let regex = Regex::new(regex).wrap_err("error parsing position regex")?;
        Ok(Self { regex })
    }
}

impl PositionMatcher for RegexPositionMatcher {
    fn find_positions(&self, input: &str) -> Result<Vec<FilePosition>> {
        let captures = self.regex.captures_iter(input);
        captures
            .map(|captures| -> Result<_> {
                let path = PathBuf::from_str(captures.get(1).unwrap().as_str())
                    .wrap_err("error parsing path")?;
                let line = u32::from_str(captures.get(2).unwrap().as_str()).unwrap();
                let column = u32::from_str(captures.get(3).unwrap().as_str()).unwrap();

                Ok(FilePosition {
                    path,
                    position: Position::LineColumn(line, column),
                })
            })
            .collect()
    }
}

pub trait StrExt {
    fn subslice_offset(&self, inner: &str) -> Option<usize>;
}

impl StrExt for &str {
    fn subslice_offset(&self, inner: &str) -> Option<usize> {
        let self_beg = self.as_ptr();
        let inner = inner.as_ptr();
        if inner < self_beg || inner > self_beg.wrapping_add(self.len()) {
            None
        } else {
            // SAFETY: we just checked that `inner` is inside `self`
            Some(unsafe { inner.offset_from(self_beg) } as usize)
        }
    }
}
