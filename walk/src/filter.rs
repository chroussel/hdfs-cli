use err::Error;
use std::ffi::OsStr;

pub trait PathFilter {
    fn is_match(&self, path: &str) -> bool;
}

pub struct TestFilter {}

impl PathFilter for TestFilter {
    fn is_match(&self, path: &str) -> bool {
        return path.ends_with(".h");
    }
}

pub struct StartFilter {
    pattern: String,
}

impl PathFilter for StartFilter {
    fn is_match(&self, path: &str) -> bool {
        path.starts_with(self.pattern.as_str())
    }
}

impl StartFilter {
    pub fn new(pattern: String) -> StartFilter {
        StartFilter { pattern: pattern }
    }
}

pub struct GlobFilter {
    pattern: glob::Pattern,
}

impl GlobFilter {
    pub fn new(path: &str) -> Result<GlobFilter, Error> {
        return Ok(GlobFilter {
            pattern: glob::Pattern::new(path)?,
        });
    }
}

impl PathFilter for GlobFilter {
    fn is_match(&self, path: &str) -> bool {
        let options = glob::MatchOptions {
            require_literal_separator: true,
            case_sensitive: true,
            require_literal_leading_dot: false,
        };
        return self.pattern.matches_with(path, &options);
    }
}
