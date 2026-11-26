use super::*;

#[test]
fn output_is_captured_and_bounded() {
    let out = run("/bin/sh", &["-c", "printf abc; exit 0"], 100).unwrap();
    assert!(out.success);
    assert_eq!(out.stdout, b"abc");
    let out = run("/bin/sh", &["-c", "printf abcdef"], 3).unwrap();
    assert_eq!(
        out.stdout.len(),
        4,
        "one byte more than allowed marks the overflow"
    );
    let out = run("/bin/sh", &["-c", "exit 3"], 10).unwrap();
    assert!(!out.success);
    assert!(run("/nonexistent/program", &[], 10).is_err());
}

#[test]
fn the_environment_is_the_fixed_one_and_stdin_is_empty() {
    let out = run("/bin/sh", &["-c", "echo $PATH; cat"], 200).unwrap();
    assert_eq!(out.stdout, b"/usr/sbin:/usr/bin:/sbin:/bin\n");
}
