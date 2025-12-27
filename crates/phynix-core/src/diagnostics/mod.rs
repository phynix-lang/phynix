pub mod parser;

use crate::diagnostics::parser::ParseDiagnosticCode;
use crate::Span;

#[derive(Debug)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug)]
pub enum DiagnosticCode {
    Parse(ParseDiagnosticCode),
}

impl From<ParseDiagnosticCode> for DiagnosticCode {
    #[inline]
    fn from(code: ParseDiagnosticCode) -> Self {
        DiagnosticCode::Parse(code)
    }
}

pub trait DiagnosticCodeStr {
    fn as_str(&self) -> &'static str;
}

impl DiagnosticCodeStr for DiagnosticCode {
    fn as_str(&self) -> &'static str {
        match self {
            DiagnosticCode::Parse(code) => code.as_str(),
        }
    }
}

#[derive(Debug)]
pub struct Label {
    pub span: Span,
    pub message: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug)]
pub struct TextEdit {
    pub span: Span,
    pub replacement: String,
}

#[derive(Debug)]
pub struct Fix {
    pub title: String,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub severity: Severity,

    pub span: Span,

    /// Extra highlights (“related info” in IDE)
    ///
    /// Use when > 1 locations matter.
    pub labels: Vec<Label>,

    /// Extra text
    ///
    /// Use when explanation > 1 sentence.
    pub notes: Vec<String>,

    /// Extra help message
    ///
    /// Use when there's a clear next step.
    pub help: Option<String>,

    /// Quick-fixes / code actions
    ///
    /// Use when automated edits are possible.
    pub fixes: Vec<Fix>,
}

impl Diagnostic {
    #[inline]
    pub fn error<C: Into<DiagnosticCode>>(
        code: C,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self::new(Severity::Error, code.into(), span, message)
    }

    #[inline]
    pub fn warning<C: Into<DiagnosticCode>>(
        code: C,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self::new(Severity::Warning, code.into(), span, message)
    }

    #[inline]
    pub fn info<C: Into<DiagnosticCode>>(
        code: C,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self::new(Severity::Info, code.into(), span, message)
    }

    #[inline]
    fn new(
        severity: Severity,
        code: DiagnosticCode,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            severity,
            span,
            labels: Vec::new(),
            notes: Vec::new(),
            help: None,
            fixes: Vec::new(),
        }
    }

    #[inline]
    pub fn primary_label(
        mut self,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        self.labels.push(Label {
            span,
            message: Some(message.into()),
            is_primary: true,
        });
        self
    }

    #[inline]
    pub fn label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            message: Some(message.into()),
            is_primary: false,
        });
        self
    }

    #[inline]
    pub fn note(mut self, message: impl Into<String>) -> Self {
        self.notes.push(message.into());
        self
    }

    #[inline]
    pub fn help(mut self, message: impl Into<String>) -> Self {
        self.help = Some(message.into());
        self
    }

    #[inline]
    pub fn fix(
        mut self,
        title: impl Into<String>,
        edits: Vec<TextEdit>,
    ) -> Self {
        self.fixes.push(Fix {
            title: title.into(),
            edits,
        });
        self
    }
}
