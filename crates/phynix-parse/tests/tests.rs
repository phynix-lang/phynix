mod expr;
mod stmt;
mod util;

#[test]
fn data_driven_tests() {
    use std::fs;
    let fixtures_dir = "tests/fixtures";
    let entries = fs::read_dir(fixtures_dir).expect("read fixtures dir");

    for entry in entries {
        let entry = entry.expect("valid entry");
        let path = entry.path();
        if path.extension().map(|s| s == "php").unwrap_or(false) {
            let src = fs::read_to_string(&path).expect("read fixture");
            let script = util::parse_ok(&src);

            // Use the filename as the snapshot name
            let test_name = path.file_stem().unwrap().to_str().unwrap();
            insta::with_settings!({
                snapshot_path => "fixtures/snapshots",
                prepend_module_to_snapshot => false,
            }, {
                insta::assert_yaml_snapshot!(test_name, script);
            });
        }
    }
}
