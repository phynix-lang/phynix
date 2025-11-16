mod util;

use crate::util::lex_err;

#[test]
fn unterminated_block_comment_errors() {
    let e = lex_err("/* no end");
    let msg = format!("{}", e);
    assert!(msg.contains("Unterminated block comment"), "got: {msg}");
}
