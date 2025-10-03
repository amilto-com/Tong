use std::process::{Command, Stdio};

fn run_prog(src: &str) -> (bool, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("prog.tong");
    std::fs::write(&path, src).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_tong"))
        .arg(&path)
        .stderr(Stdio::piped())
        .output()
        .expect("run");
    (output.status.success(), String::from_utf8_lossy(&output.stdout).into_owned(), String::from_utf8_lossy(&output.stderr).into_owned())
}

#[test]
fn oob_simple() {
    let program = r#"fn main(){ let xs = [1,2,3] print(xs[5]) } main()"#;
    let (ok, _out, err) = run_prog(program);
    assert!(!ok, "expected failure, got success: stdout={} stderr={}", _out, err);
    assert!(err.contains("index out of bounds"), "missing error message: {}", err);
}

#[test]
fn oob_update_sugar() {
    let program = r#"fn main(){ let arr = [0,1] arr[2] = 9 } main()"#;
    let (ok, _out, err) = run_prog(program);
    assert!(!ok, "expected failure on update sugar, got success");
    assert!(err.contains("index out of bounds"), "missing oob error for update sugar: {}", err);
}

#[test]
fn oob_nested_chain() {
    let program = r#"fn main(){ let grid = [[1,2],[3,4]] print(grid[1][2]) } main()"#;
    let (ok, _out, err) = run_prog(program);
    assert!(!ok, "expected failure on nested chain");
    assert!(err.contains("index out of bounds"), "missing oob error nested: {}", err);
}
