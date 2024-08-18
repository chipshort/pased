use std::ops::Range;

use regex::Replacer;

use crate::{position::BytePosition, str_ext::StrExt};

pub struct RangeSet {
    ranges: Vec<Range<BytePosition>>,
}

impl RangeSet {
    pub fn new(ranges: Vec<Range<BytePosition>>) -> Self {
        Self { ranges }
    }

    pub fn contains(&self, pos: BytePosition) -> bool {
        self.ranges.iter().any(|range| range.contains(&pos))
    }
}

pub struct PositionalReplacer<'a> {
    file_content: &'a str,
    ranges: RangeSet,
    replacement: &'a str,
}

impl<'a> PositionalReplacer<'a> {
    pub fn new(file_content: &'a str, ranges: RangeSet, replacement: &'a str) -> Self {
        Self {
            file_content,
            ranges,
            replacement,
        }
    }
}

impl<'a> Replacer for PositionalReplacer<'a> {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        let match_str = caps.get(0).unwrap().as_str();
        let match_offset = self
            .file_content
            .subslice_offset(match_str)
            .expect("PositionalReplacer was configured incorrectly");

        // only replace if the match is within the ranges
        if self.ranges.contains(BytePosition(match_offset)) {
            caps.expand(self.replacement, dst);
        } else {
            dst.push_str(match_str);
        }
    }
}
