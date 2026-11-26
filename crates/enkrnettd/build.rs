// The version may be overridden at build time (`config::VERSION`): make Cargo rebuild when the
// variable changes, so that a successor built for the VM test never reuses a stale binary.
fn main() {
    println!("cargo:rerun-if-env-changed=ENKRNETT_VERSION");
}
