use ravel::cli;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file_args: Vec<&String> = args.iter().filter(|a| *a != "--help").collect();

    if args.iter().any(|a| a == "--help") {
        cli::print_help();
        return;
    }

    if file_args.len() > 1 {
        cli::run(file_args[1]);
        return;
    }

    cli::repl();
}
