extern crate lalrpop;

fn main() {
    println!("cargo:rerun-if-changed=src/haskell.lalrpop");
    lalrpop::process_src().unwrap();
}
