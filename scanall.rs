// cargo-deps: walkdir="*"

extern crate walkdir;

use std::process::Command;
use walkdir::WalkDir;

fn main() {
    for entry in WalkDir::new("./out/language_c") {
        let entry = entry.unwrap();
        if !entry.file_type().is_file() {
            continue;
        }

        let output = Command::new("rustc")
                .args(&["-Zprint-link-args", "-Zparse-only", &entry.path().display().to_string()])
                .output()
                .expect("failed to execute process");

        if output.status.success() {
            println!("..... {}", entry.path().display());
        } else {
            println!("BAD!! {}", entry.path().display());
            print!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }
}
