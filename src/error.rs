//! Errors with stable codes, so a script using `--json` can branch on them.

use std::fmt;

use serde::Serialize;

/// The error codes a script can match on. They are stable; the messages are not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Code {
    UnknownCreature,
    AmbiguousCreature,
    BadPack,
    BadDate,
    BadFlag,
    Usage,
}

/// An error a player can act on: what went wrong and, sometimes, what to try.
#[derive(Debug, Clone, Serialize)]
pub struct TinselError {
    pub code: Code,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl TinselError {
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// The error as `--json` reports it: one object on stderr.
    pub fn to_json(&self) -> String {
        #[derive(Serialize)]
        struct Wrapped<'a> {
            error: &'a TinselError,
        }
        serde_json::to_string(&Wrapped { error: self }).unwrap_or_default()
    }
}

impl fmt::Display for TinselError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.hint {
            Some(hint) => write!(f, "{}; {}", self.message, hint),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for TinselError {}

pub type Result<T> = std::result::Result<T, TinselError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_error_has_kebab_case_code_and_skips_empty_hint() {
        let e = TinselError::new(Code::UnknownCreature, "no such creature");
        assert_eq!(
            e.to_json(),
            r#"{"error":{"code":"unknown-creature","message":"no such creature"}}"#
        );
    }

    #[test]
    fn display_appends_the_hint() {
        let e = TinselError::new(Code::BadDate, "bad date").with_hint("use YYYY-MM-DD");
        assert_eq!(e.to_string(), "bad date; use YYYY-MM-DD");
    }
}
