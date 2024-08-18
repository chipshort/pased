use std::{
    collections::HashMap,
    io::{stdin, Read},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};
use derive_more::derive::Display;
use eyre::{Context, Result};
use itertools::Itertools;
use position_matcher::{Position, PositionMatcher, RegexPositionMatcher, RustPositionMatcher};
use positional_replacer::{PositionalReplacer, RangeSet};
use regex::Regex;

mod position_matcher;
mod positional_replacer;

/// Replace all occurrences of a regex with a string.
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("position_matcher")
        .required(true)
        .args(&["rust", "position_regex"]),
))]
struct Cli {
    /// Regex to search for
    search: String,

    /// The string to replace the regex with
    ///
    /// You can use `$0` for the whole match, `$1` for the first capture group, etc. or
    /// `$NAME` to refer to the matched groups.
    /// If you want to use a `$` literal, you can escape it with a backslash: `\$`.
    replace: String,

    /// The number of lines to look at before and after the match
    #[arg(short, long, value_name = "LINES")]
    lines: u32,

    /// Expect Rust compiler output as position input.
    /// You can specify which level of Rust compiler messages to include the positions for
    #[arg(short, long, default_value = "all", conflicts_with = "position_regex")]
    rust: Option<RustLevel>,

    /// A regex to match the file positions.
    /// This expects the file path to be in the first capture group, the line number in the second and the column in the third.
    #[arg(short, long, conflicts_with = "rust")]
    position_regex: Option<String>,
}

#[derive(Clone, Debug, Default, ValueEnum, Display)]
pub enum RustLevel {
    #[default]
    All,
    Warning,
    Error,
}

impl RustLevel {
    pub fn matches(&self, line: &str) -> bool {
        let has_keyword = match self {
            RustLevel::All => line.starts_with("warning: ") || line.starts_with("error: "),
            RustLevel::Warning => line.starts_with("warning: "),
            RustLevel::Error => line.starts_with("error: "),
        };

        has_keyword && !(line.contains("generated") && line.contains("cargo fix"))
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let search = Regex::new(&args.search).wrap_err("error parsing search regex")?;

    match (args.rust, args.position_regex) {
        (Some(rust), None) => {
            let matcher = RustPositionMatcher::new(rust);
            let positions = find_positions(matcher)?;
            replace_all(&search, &args.replace, positions, args.lines)?;
        }
        (None, Some(regex)) => {
            let matcher = RegexPositionMatcher::new(&regex)?;
            let positions = find_positions(matcher)?;
            replace_all(&search, &args.replace, positions, args.lines)?;
        }
        _ => unreachable!("clap should prevent this"),
    }

    Ok(())
}

fn replace_all(
    search: &Regex,
    replace: &str,
    positions: HashMap<PathBuf, Vec<Position>>,
    lines: u32,
) -> Result<()> {
    // iterate through positions
    for (path, positions) in positions {
        // read the file
        let file = std::fs::read_to_string(&path)?;

        // create a RangeSet from the positions
        let ranges = RangeSet::new(
            positions
                .into_iter()
                .map(|p| p.lines_around(lines))
                .map(|(start, end)| start.to_byte_position(&file)..end.to_byte_position(&file))
                .collect(),
        );

        // replace all occurrences of the regex with the replace string
        let new_content = search.replace_all(
            file.as_str(),
            PositionalReplacer::new(&file, ranges, replace),
        );
        std::fs::write(&path, new_content.as_bytes()).wrap_err("error writing results")?;
    }

    Ok(())
}

fn find_positions(matcher: impl PositionMatcher) -> Result<HashMap<PathBuf, Vec<Position>>> {
    // read stdin (TODO: this could be made streaming)
    let mut input = String::new();
    stdin().read_to_string(&mut input)?;

    let positions = matcher
        .find_positions(&input)?
        .into_iter()
        .map(|p| (p.path, p.position))
        .into_group_map();
    Ok(positions)
}
