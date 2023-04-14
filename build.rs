use copy_to_output::copy_to_output;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=assets/*");
    copy_to_output("assets", &env::var("PROFILE").unwrap()).expect("Could not copy");
    copy_to_output("SDL2.dll", &env::var("PROFILE").unwrap()).expect("Could not copy");
}
