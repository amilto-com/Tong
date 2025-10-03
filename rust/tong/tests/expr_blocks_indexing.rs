use std::process::{Command, Stdio};
use std::str;

// Simple integration-style test invoking the binary with small inline programs via stdin (REPL) or temp file.
// Focus: nested indexing and block expression value propagation.
#[test]
fn block_expression_and_indexing() {
    // We'll craft a small program that exercises nested indexing and a block expression.
    let program = r#"
fn main() {
    let grid = [[1,2],[3,4]]
    print(grid[1][0])
    let val = {
        let a = 5
        let b = a * 3
        b + 2
    }
    print(val)
}
main()
"#;

    // Write to a temp file because the current binary expects a file arg for non-REPL.
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("prog.tong");
    std::fs::write(&path, program).expect("write prog");

    let output = Command::new(env!("CARGO_BIN_EXE_tong"))
        .arg(&path)
        .stderr(Stdio::piped())
        .output()
        .expect("run program");

    assert!(
        output.status.success(),
        "program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Expect two lines: 3 (grid[1][0]) and 17 (block: a=5, b=15, b+2=17)
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["3", "17"], "unexpected output: {:?}", lines);
}
