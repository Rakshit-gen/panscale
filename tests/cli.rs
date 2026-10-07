use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_panscale"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn stdout(o: &Output) -> String {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout.clone()).unwrap()
}

#[test]
fn scales_stdin_by_a_factor() {
    let o = run(&["scale", "--by", "2"], "Serves 2\n- 1 cup rice\n- 3 eggs\n");
    assert_eq!(stdout(&o), "Serves 4\n- 2 cups rice\n- 6 eggs\n");
}

#[test]
fn scales_to_a_number_of_servings() {
    let o = run(&["scale", "--serves", "6"], "Serves 4\n- 2 tsp cumin\n");
    assert_eq!(stdout(&o), "Serves 6\n- 1 tbsp cumin\n");
}

#[test]
fn serves_needs_a_yield_line() {
    let o = run(&["scale", "--serves", "6"], "- 2 tsp cumin\n");
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("use --by"));
}

#[test]
fn scales_the_example_to_a_bigger_pan() {
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/brownies.txt");
    let out = stdout(&run(&["scale", file, "--from-pan", "8in square", "--pan", "9x13"], ""));
    assert!(out.contains("Makes 29 squares"));
    assert!(out.contains("- 1 7/8 cups sugar"));
    assert!(out.contains("4 eggs"));
    // The method is left as written.
    assert!(out.contains("beat in the sugar, 2 eggs"));
    assert!(out.contains("350°F"));
}

#[test]
fn metric_example() {
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/brownies.txt");
    let out = stdout(&run(&["scale", file, "--by", "1", "--metric"], ""));
    assert!(out.contains("- 115 g butter"));
    assert!(out.contains("- 85 g dark chocolate"));
    assert!(out.contains("175°C"));
}

#[test]
fn pan_command_and_bad_pans() {
    let out = stdout(&run(&["pan", "8in round", "9in round"], ""));
    assert_eq!(out, "9in round holds 1.27 times as much as 8in round.\n");
    let o = run(&["pan", "big", "9x13"], "");
    assert!(!o.status.success());
}

#[test]
fn rejects_a_zero_factor() {
    assert!(!run(&["scale", "--by", "0"], "- 1 cup rice\n").status.success());
}
