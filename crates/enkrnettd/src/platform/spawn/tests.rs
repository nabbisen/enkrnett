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

#[test]
fn a_watched_program_is_killed_when_its_file_grows_past_the_limit() {
    let dir = std::env::temp_dir().join(format!("enkrnettd-spawn-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("out");
    let f = file.to_str().unwrap();
    // Within the limit: the program's exit status is the answer.
    let w = run_watching(
        "/bin/sh",
        &["-c", &format!("head -c 100 /dev/zero > {f}")],
        &file,
        100,
    )
    .unwrap();
    assert_eq!(w, Watched::Exited(true));
    assert_eq!(std::fs::metadata(&file).unwrap().len(), 100);
    let w = run_watching("/bin/sh", &["-c", "exit 2"], &file, 100).unwrap();
    assert_eq!(w, Watched::Exited(false));
    // Growing past the limit: killed while it writes, within the watch interval.
    let w = run_watching(
        "/bin/sh",
        &["-c", &format!("i=0; while [ $i -lt 100 ]; do head -c 1000 /dev/zero >> {f}; i=$((i+1)); sleep 0.05; done")],
        &file,
        5_000,
    )
    .unwrap();
    assert_eq!(w, Watched::TooLarge);
    assert!(std::fs::metadata(&file).unwrap().len() < 100_000);
    let _ = std::fs::remove_dir_all(&dir);
}
