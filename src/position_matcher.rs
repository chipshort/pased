use std::{path::PathBuf, str::FromStr};

use eyre::{eyre, Context, Result};
use itertools::Itertools;
use regex::Regex;

use crate::{
    position::{FilePosition, Position},
    RustLevel,
};

/// A position matcher takes input from stdin and produces positions in files from that input.
pub trait PositionMatcher {
    /// Find positions in the input.
    fn find_positions(&self, input: &str) -> Result<Vec<FilePosition>>;
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
