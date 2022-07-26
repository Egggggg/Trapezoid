use std::path::Path;

use glob::{Pattern, PatternError};

pub fn to_pattern(input: &str) -> Result<Pattern, PatternError> {
    let pattern = Pattern::new(input)?;

    Ok(pattern)
}

pub fn to_path(input: &str) -> &Path {
    Path::new(input)
}
