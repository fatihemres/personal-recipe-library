use std::{path::PathBuf, process::ExitCode};
fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(args.len() == 2 || (args.len() == 3 && args[2] == "--allow-validation")) {
        eprintln!("Usage: catalog_import <data-directory> <manifest.json> [--allow-validation]");
        return ExitCode::FAILURE;
    }
    match personal_recipe_library_lib::catalog_tool::import(
        &PathBuf::from(&args[0]),
        &PathBuf::from(&args[1]),
        args.len() == 3,
    ) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}", serde_json::to_string(&error).unwrap());
            ExitCode::FAILURE
        }
    }
}
