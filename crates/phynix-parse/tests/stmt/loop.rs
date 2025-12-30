use crate::assert_script_snapshot;

#[test]
fn while_simple() {
    assert_script_snapshot!("<?php while (true) echo 1;");
}

#[test]
fn while_colon_syntax() {
    assert_script_snapshot!("<?php while (true): echo 1; endwhile;");
}

#[test]
fn do_while_simple() {
    assert_script_snapshot!("<?php do { echo 1; } while (true);");
}

#[test]
fn for_simple() {
    assert_script_snapshot!("<?php for ($i = 0; $i < 10; $i++) echo $i;");
}

#[test]
fn for_multiple_exprs() {
    assert_script_snapshot!(
        "<?php for ($i = 0, $j = 0; $i < 10; $i++, $j++) echo $i;"
    );
}

#[test]
fn for_empty() {
    assert_script_snapshot!("<?php for (;;);");
}

#[test]
fn foreach_simple() {
    assert_script_snapshot!("<?php foreach ($arr as $val) echo $val;");
}

#[test]
fn foreach_with_key() {
    assert_script_snapshot!("<?php foreach ($arr as $key => $val) echo $val;");
}

#[test]
fn foreach_colon_syntax() {
    assert_script_snapshot!(
        "<?php foreach ($arr as $val): echo $val; endforeach;"
    );
}
