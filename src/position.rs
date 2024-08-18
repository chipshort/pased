use std::path::PathBuf;

use crate::str_ext::StrExt;

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
