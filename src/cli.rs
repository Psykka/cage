use std::env;

use cage::config::Config;

pub fn run() -> i32 {
    let args: Vec<String> = env::args().skip(1).collect();

    match dispatch(&args) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Erro: {e}");
            1
        }
    }
}

fn dispatch(args: &[String]) -> Result<(), String> {
    let Some(cmd) = args.first() else {
        return Err("Shell not implemented yet".into());
    };

    match cmd.as_str() {
        "init" => init_in(),
        unknown => Err(format!("Unknown command: {unknown}")),
    }
}

fn init_in() -> Result<(), String> {
    let path =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    Config::init_in(&path, false).map_err(|e| format!("Failed to initialize config: {e}"))?;

    Ok(())
}
