mod cli;

use std::process;

fn main() {
    let code = cli::run();
    process::exit(code)
}
