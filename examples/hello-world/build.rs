fn main() {
    // An instruction to cargo, not a log line: rebuild when anything under
    // src/www is added or changed (content!() cannot notice new files itself).
    println!("cargo::rerun-if-changed=src/www");
}
