use crate::diagnostics::DiagnosticCodeStr;

#[derive(Debug)]
pub enum ParseDiagnosticCode {
    ExpectedExpression,
    ExpectedIdent,
    ExpectedStatement,
    ExpectedToken,

    UnexpectedToken,

    ExpectedAtLeastOneArgument,
    ExpectedCaseOrDefaultInSwitch,
    ExpectedCatchExceptionType,
    ExpectedCatchOrFinallyAfterTry,
    ExpectedIfAfterElse,
    ExpectedIntLiteral,
    InvalidFloatLiteral,
    InvalidIntLiteral,
    PositionalAfterNamedArg,
    UnpackedArrayItemWithFatArrow,
    VariadicAfterNamedArg,
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
            ParseDiagnosticCode::ExpectedToken => "parse.expected_token",

            ParseDiagnosticCode::UnexpectedToken => "parse.unexpected_token",

            ParseDiagnosticCode::ExpectedAtLeastOneArgument => {
                "parse.expected_at_least_one_argument"
            },
            ParseDiagnosticCode::ExpectedCaseOrDefaultInSwitch => {
                "parse.expected_case_or_default_in_switch"
            },
            ParseDiagnosticCode::ExpectedCatchExceptionType => {
                "parse.expected_catch_exception_type"
            },
            ParseDiagnosticCode::ExpectedCatchOrFinallyAfterTry => {
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
