use ravel::cli;
use ravel::config::Config;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        cli::print_help();
        return;
    }

    if args.iter().any(|a| a == "--version" || a == "-v") {
        cli::print_version();
        return;
    }

    let config = Config::load();

    if let Some(idx) = args.iter().position(|a| a == "--serve") {
        let port = args
            .get(idx + 1)
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(config.port);
        let base = args
            .iter()
            .position(|a| a == "--base")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or(&config.base);
        cli::serve(port, base);
        return;
    }

    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();

    if args.iter().any(|a| a == "--build") {
        if positional.len() > 1 {
            if !cli::build(positional[1]) {
                std::process::exit(1);
            }
        } else {
            eprintln!("Error: --build requires a script file");
            eprintln!("Usage: ravel --build <file>");
            std::process::exit(1);
        }
        return;
    }

    if positional.len() > 1 {
        if !cli::run(positional[1]) {
            std::process::exit(1);
        }
        return;
    }

    cli::repl();
}
