use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum StringStyle {
    SingleQuoted,
    DoubleQuoted,
    Heredoc,
    Nowdoc,
    Backtick,
}
