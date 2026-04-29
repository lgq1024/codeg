use std::env;

fn main() {
    println!("PATH: {}", env::var("PATH").unwrap_or_default());
    match which::which("hermes") {
        Ok(p) => println!("found: {}", p.display()),
        Err(e) => println!("not found: {}", e),
    }
}
