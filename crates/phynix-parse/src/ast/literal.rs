#[derive(Debug)]
pub enum StringStyle {
    SingleQuoted,
    DoubleQuoted,
    Heredoc,
    Nowdoc,
    Backtick,
}
