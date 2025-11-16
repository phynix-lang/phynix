use crate::Span;

#[derive(Debug)]
pub enum Severity {
    Error,
    Warning,
    Legacy,
    Info,
}

pub struct Label {
    pub span: Span,
    pub message: Option<String>,
    pub is_primary: bool,
}

pub struct Diagnostic {
    pub span: Span,
    pub severity: Severity,
    pub message: String,
}

// TODO
// pub struct Diagnostic {
//     pub code: &'static str,
//     pub message: String,
//     pub severity: Severity,
//     pub labels: Vec<Label>,
//     pub notes: Vec<String>,
//     pub help: Option<String>,
// }
