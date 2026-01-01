use crate::test_error;

test_error!(if_missing_paren_recovery, "<?php if (true echo 1;");
test_error!(if_missing_cond_recovery, "<?php if () echo 1;");
test_error!(while_missing_paren_recovery, "<?php while (true echo 1;");
test_error!(
    for_missing_semicolon_recovery,
    "<?php for ($i = 0 $i < 10; $i++) echo $i;"
);
test_error!(
    switch_missing_brace_recovery,
    "<?php switch ($a) case 1: echo 1; endswitch;"
);
test_error!(
    match_missing_arm_recovery,
    "<?php match ($a) { 1 => 'one', };"
);
test_error!(
    match_missing_expr_recovery,
    "<?php match ($a) { 1 => , 2 => 'two' };"
);
test_error!(
    missing_semicolon_in_block_recovery,
    "<?php { echo 1 echo 2; }"
);
test_error!(class_missing_name_recovery, "<?php class { }");
test_error!(
    function_missing_params_recovery,
    "<?php function foo echo 1;"
);
test_error!(class_member_single_error, "<?php class Foo { public; }");
