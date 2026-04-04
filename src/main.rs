use ravel::cli;

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

    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();

    if args.iter().any(|a| a == "--build") {
        if positional.len() > 1 {
            cli::build(positional[1]);
        } else {
            eprintln!("Error: --build requires a script file");
            eprintln!("Usage: ravel --build <file>");
            std::process::exit(1);
        }
        return;
    }

    if positional.len() > 1 {
        cli::run(positional[1]);
        return;
    }

    cli::repl();
}
