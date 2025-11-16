use anyhow::Result;
use clap::{Parser, Subcommand};
use phynix_core::diagnostics::Severity;
use phynix_core::{LanguageKind, Strictness};
use phynix_lex::{lex, LexError};
use phynix_parse::ParseResult;
use std::ops::Range;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "phynix", version, about = "Phynix language toolchain")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Lex a source file and dump tokens
    Lex { path: String },
    /// Parse a source file and dump AST
    Parse { path: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Lex { path } => cmd_lex(&path)?,
        Cmd::Parse { path } => cmd_parse(&path)?,
    }
    Ok(())
}

fn cmd_lex(path: &str) -> Result<()> {
    let files = collect_php_files(path);
    if files.is_empty() {
        anyhow::bail!("no .php files found at {}", path);
    }

    for f in files {
        let fname = f.to_string_lossy();
        if IGNORE.iter().any(|x| fname.contains(x)) {
            continue;
        }

        let bytes = std::fs::read(&f)?;
        let text = match read_text(&f, &bytes) {
            Some(value) => value,
            None => continue,
        };

        match lex(&text, LanguageKind::PhpCompat, Strictness::Lenient) {
            Ok(lex_result) => {
                if Path::new(path).is_file() {
                    for tok in &lex_result.tokens {
                        let snippet = &text
                            [tok.span.start as usize..tok.span.end as usize];
                        let printable = snippet.escape_debug().to_string();
                        println!(
                            "{:?}\t{:?}\t{}",
                            tok.kind, tok.span, printable
                        );
                    }
                }
            },
            Err(LexError::At(byte_pos)) => {
                eprintln!("\n[LEX ERROR] in {}", f.display());
                print_span(&text, "Lex error here", byte_pos..byte_pos + 1);
                return Ok(());
            },
            Err(LexError::UnterminatedBlock(start_byte)) => {
                eprintln!("\n[LEX ERROR] in {}", f.display());
                print_span(
                    &text,
                    "Unterminated block comment starts here",
                    start_byte..start_byte + 2,
                );
                return Ok(());
            },
        }
    }

    Ok(())
}
fn cmd_parse(path: &str) -> Result<()> {
    let files = collect_php_files(path);
    if files.is_empty() {
        anyhow::bail!("no .php files found at {}", path);
    }

    for f in files {
        let fname = f.to_string_lossy();
        if IGNORE.iter().any(|x| fname.contains(x)) {
            continue;
        }

        print!("\n=== Processing file: {} ===", f.display());

        // let a = fname.contains("ably-loader");
        // if a {
        //     print!(" (time to debug)");
        // }

        let bytes = std::fs::read(&f)?;
        let text = match read_text(&f, &bytes) {
            Some(value) => value,
            None => continue,
        };

        let lex_result =
            match lex(&text, LanguageKind::PhpCompat, Strictness::Lenient) {
                Ok(ok) => ok,
                Err(LexError::At(byte_pos)) => {
                    eprintln!("\n[LEX ERROR] in {}", f.display());
                    print_span(&text, "Lex error here", byte_pos..byte_pos + 1);
                    return Ok(());
                },
                Err(LexError::UnterminatedBlock(start_byte)) => {
                    eprintln!("\n[LEX ERROR] in {}", f.display());
                    print_span(
                        &text,
                        "Unterminated block comment starts here",
                        start_byte..start_byte + 2,
                    );
                    return Ok(());
                },
            };

        let parse_result = phynix_parse::parse(
            &text,
            &lex_result.tokens,
            lex_result.lang,
            lex_result.strictness,
        );

        if has_parse_errors(&parse_result.diagnostics) {
            eprintln!("\n[PARSE FAIL] in {}\n", f.display());

            println!("=== TOKENS ===");
            for tok in &lex_result.tokens {
                let snippet =
                    &text[tok.span.start as usize..tok.span.end as usize];
                let printable = snippet.escape_debug().to_string();
                println!("{:?}\t{:?}\t{}", tok.kind, tok.span, printable);
            }

            println!("\n=== AST ===");
            println!("{:#?}", parse_result.ast);

            println!("\n=== Diagnostics ===");
            print_diagnostics(&text, &parse_result);

            return Ok(());
        } else if Path::new(path).is_file() {
            println!("=== AST ===");
            println!("{:#?}", parse_result.ast);

            println!("=== Diagnostics ===");
            if parse_result.diagnostics.is_empty() {
                println!("(none)");
            } else {
                print_diagnostics(&text, &parse_result);
            }
        }
    }

    Ok(())
}

fn read_text<'a>(f: &PathBuf, bytes: &'a Vec<u8>) -> Option<&'a str> {
    let text = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping non-UTF8 file: {}", f.display());
            return None;
        },
    };
    Some(text)
}

fn print_diagnostics(text: &&str, parse_result: &ParseResult) {
    for diag in &parse_result.diagnostics {
        let label = match diag.severity {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Legacy => "Legacy",
            Severity::Info => "Info",
        };
        println!(
            "- {}: {} (bytes {}..{})",
            label, diag.message, diag.span.start, diag.span.end
        );
        print_span(&text, "", diag.span.start..diag.span.end);
    }
}

fn print_span(src: &str, extra_msg: &str, byte_range: Range<u32>) {
    let (line_no, col_no_start, line_src) = locate(src, byte_range.start);
    let (_, col_no_end, _) = locate(src, byte_range.end);

    eprintln!(
        "  at line {}, col {}..{}",
        line_no + 1,
        col_no_start + 1,
        col_no_end.max(col_no_start + 1)
    );

    eprintln!("  {}", line_src);

    let start_spaces = " ".repeat(col_no_start);
    let underline_len = (col_no_end.max(col_no_start + 1)) - col_no_start;
    let carets = "^".repeat(underline_len);

    if extra_msg.is_empty() {
        eprintln!("  {}{}", start_spaces, carets);
    } else {
        eprintln!("  {}{} {}", start_spaces, carets, extra_msg);
    }
}

fn locate(src: &str, byte_pos: u32) -> (usize, usize, String) {
    let clamped = byte_pos.min(src.len() as u32) as usize;

    let mut line_start = 0;
    let mut line_idx = 0;
    for (i, ch) in src.char_indices() {
        if i >= clamped {
            break;
        }
        if ch == '\n' {
            line_start = i + 1;
            line_idx += 1;
        }
    }

    let line_end = src[line_start..]
        .find('\n')
        .map(|off| line_start + off)
        .unwrap_or(src.len());

    let line_str = src[line_start..line_end].to_string();

    let col_str = &src[line_start..clamped];
    let col_no = col_str.chars().count();

    (line_idx, col_no, line_str)
}

fn collect_php_files(root: &str) -> Vec<PathBuf> {
    let path = Path::new(root);
    if path.is_file() && is_php_path(path) {
        return vec![path.to_path_buf()];
    }
    let mut out = Vec::new();
    if path.is_dir() {
        for entry in walkdir::WalkDir::new(root) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if entry.file_type().is_file() && is_php_path(entry.path()) {
                out.push(entry.path().to_path_buf());
            }
        }
    }
    out
}

fn has_parse_errors(diags: &[phynix_core::diagnostics::Diagnostic]) -> bool {
    diags.iter().any(|d| matches!(d.severity, Severity::Error))
}

static IGNORE: &[&str] = &[
    "examples/laravel\\tests\\Foundation\\fixtures\\bad-syntax-strategy.php",
    "examples/joomla\\libraries\\vendor\\squizlabs\\php_codesniffer\\src\\Standards\\Generic\\Tests\\Arrays\\DisallowLongArraySyntaxUnitTest.1.inc",
    "examples/joomla\\libraries\\vendor\\squizlabs\\php_codesniffer\\src\\Standards\\Generic\\Tests\\Arrays\\DisallowLongArraySyntaxUnitTest.2.inc",
    "examples/joomla\\libraries\\vendor\\squizlabs\\php_codesniffer\\src\\Standards\\Generic\\Tests\\Arrays\\DisallowLongArraySyntaxUnitTest.3.inc",
];

fn is_php_path(path: &Path) -> bool {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
    {
        Some(ref ext)
            if ["php", "phtml", "inc", "php5", "php7", "phpt"]
                .contains(&ext.as_str()) =>
        {
            true
        },
        _ => false,
    }
}
