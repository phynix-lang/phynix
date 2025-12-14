mod util;

use crate::util::lex_err_php_prefixed;

#[test]
fn unterminated_block_comment_errors() {
    let e = lex_err_php_prefixed("/* no end");
    let msg = format!("{}", e);
    assert!(msg.contains("Unterminated block comment"), "got: {msg}");
}
