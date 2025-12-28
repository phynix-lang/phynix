use crate::diagnostics::{DiagnosticCodeStr, DiagnosticErrorMessage};
use crate::token::TokenKind;

#[derive(Debug)]
pub enum ParseDiagnosticCode {
    ExpectedExpression,
    ExpectedIdent,
    ExpectedStatement,

    ExpectedToken { expected: Vec<TokenKind> },

    ExpectedOneOf { codes: Vec<ParseDiagnosticCode> },

    // TODO: Do I even need that?
    UnexpectedToken,

    ExpectedAtLeastOneArgument,
    ExpectedCaseOrDefaultInSwitch,
    ExpectedCatchExceptionType,
    ExpectedCatchOrFinally,
    ExpectedIfAfterElse,
    ExpectedIntLiteral,
    InvalidFloatLiteral,
    InvalidIntLiteral,
    PositionalAfterNamedArg,
    UnpackedArrayItemWithFatArrow,
    VariadicAfterNamedArg,
}

impl ParseDiagnosticCode {
    pub fn expected_token(token: TokenKind) -> Self {
        Self::ExpectedToken {
            expected: vec![token],
        }
    }

    pub fn expected_tokens(
        tokens: impl IntoIterator<Item = TokenKind>,
    ) -> Self {
        Self::ExpectedToken {
            expected: tokens.into_iter().collect(),
        }
    }

    pub fn expected_one_of(
        codes: impl IntoIterator<Item = ParseDiagnosticCode>,
    ) -> Self {
        Self::ExpectedOneOf {
            codes: codes.into_iter().collect(),
        }
    }

    fn expectation_name(&self) -> String {
        match self {
            Self::ExpectedExpression => "expression".to_string(),
            Self::ExpectedIdent => "identifier".to_string(),
            Self::ExpectedStatement => "statement".to_string(),
            Self::ExpectedToken { expected } => expected
                .iter()
                .map(|token| format!("'{}'", token.display_name()))
                .collect::<Vec<_>>()
                .join(" or "),
            other => other.default_message(),
        }
    }
}

impl DiagnosticErrorMessage for ParseDiagnosticCode {
    fn default_message(&self) -> String {
        match self {
            Self::ExpectedExpression => "expected expression".to_string(),
            Self::ExpectedIdent => "expected identifier".to_string(),
            Self::ExpectedStatement => "expected statement".to_string(),

            Self::ExpectedToken { expected } => match expected.len() {
                1 => format!("expected '{}'", expected[0].display_name()),
                2 => format!(
                    "expected '{}' or '{}'",
                    expected[0].display_name(),
                    expected[1].display_name()
                ),
                _ => {
                    let last = expected.last().unwrap();
                    let rest: Vec<_> = expected[..expected.len() - 1]
                        .iter()
                        .map(|t| format!("'{}'", t.display_name()))
                        .collect();
                    format!(
                        "expected {}, or '{}'",
                        rest.join(", "),
                        last.display_name()
                    )
                },
            },

            Self::ExpectedOneOf { codes } => {
                let items: Vec<String> =
                    codes.iter().map(|code| code.expectation_name()).collect();

                match items.len() {
                    1 => format!("expected {}", items[0]),
                    2 => format!("expected {} or {}", items[0], items[1]),
                    _ => {
                        let last = items.last().unwrap();
                        let rest = items[..items.len() - 1].join(", ");
                        format!("expected {}, or {}", rest, last)
                    },
                }
            },

            Self::UnexpectedToken => "unexpected token".to_string(),

            Self::ExpectedAtLeastOneArgument => {
                "expected at least one argument".to_string()
            },
            Self::ExpectedCaseOrDefaultInSwitch => {
                "expected 'case' or 'default' in switch".to_string()
            },
            Self::ExpectedCatchExceptionType => {
                "expected exception type".to_string()
            },
            Self::ExpectedCatchOrFinally => {
                "expected 'catch' or 'finally'".to_string()
            },
            Self::ExpectedIfAfterElse => {
                "expected 'if' after 'else'".to_string()
            },
            Self::ExpectedIntLiteral => "expected integer literal".to_string(),
            Self::InvalidFloatLiteral => "invalid float literal".to_string(),
            Self::InvalidIntLiteral => "invalid integer literal".to_string(),
            Self::PositionalAfterNamedArg => {
                "positional argument not allowed after named arguments"
                    .to_string()
            },
            Self::UnpackedArrayItemWithFatArrow => {
                "unpacked array item cannot have a key".to_string()
            },
            Self::VariadicAfterNamedArg => {
                "variadic unpack not allowed after named arguments".to_string()
            },
        }
    }
}

impl DiagnosticCodeStr for ParseDiagnosticCode {
    fn as_str(&self) -> &'static str {
        match self {
            ParseDiagnosticCode::ExpectedExpression => {
                "parse.expected_expression"
            },
            ParseDiagnosticCode::ExpectedIdent => "parse.expected_ident",
            ParseDiagnosticCode::ExpectedStatement => {
                "parse.expected_statement"
            },

            ParseDiagnosticCode::ExpectedToken { .. } => "parse.expected_token",

            ParseDiagnosticCode::ExpectedOneOf { .. } => {
                "parse.expected_one_of"
            },

            ParseDiagnosticCode::UnexpectedToken { .. } => {
                "parse.unexpected_token"
            },

            ParseDiagnosticCode::ExpectedAtLeastOneArgument => {
                "parse.expected_at_least_one_argument"
            },
            ParseDiagnosticCode::ExpectedCaseOrDefaultInSwitch => {
                "parse.expected_case_or_default_in_switch"
            },
            ParseDiagnosticCode::ExpectedCatchExceptionType => {
                "parse.expected_catch_exception_type"
            },
            ParseDiagnosticCode::ExpectedCatchOrFinally => {
                "parse.expected_catch_or_finally_after_try"
            },
            ParseDiagnosticCode::ExpectedIfAfterElse => {
                "parse.expected_if_after_else"
            },
            ParseDiagnosticCode::ExpectedIntLiteral => {
                "parse.expected_int_literal"
            },
            ParseDiagnosticCode::InvalidFloatLiteral => {
                "parse.invalid_float_literal"
            },
            ParseDiagnosticCode::InvalidIntLiteral => {
                "parse.invalid_int_literal"
            },
            ParseDiagnosticCode::PositionalAfterNamedArg => {
                "parse.positional_after_named_arg"
            },
            ParseDiagnosticCode::UnpackedArrayItemWithFatArrow => {
                "parse.unpacked_array_item_with_fat_arrow"
            },
            ParseDiagnosticCode::VariadicAfterNamedArg => {
                "parse.variadic_after_named_arg"
            },
        }
    }
}
