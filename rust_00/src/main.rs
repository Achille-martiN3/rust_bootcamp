use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "hello",
    about = "Petit programme de salutation en ligne de commande",
    disable_help_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    /// Name to greet
    #[arg(value_name = "NAME", default_value = "World")]
    name: String,

    /// Convert to uppercase
    #[arg(long)]
    upper: bool,

    /// Repeat greeting N times
    #[arg(long, default_value_t = 1, value_name = "N")]
    repeat: u32,

    /// Custom help flag
    #[arg(short = 'h', long = "help")]
    help: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.help {
        print_help();
        return;
    }

    let mut name = cli.name;

    if cli.upper {
        name = name.to_uppercase();
    }

    for _ in 0..cli.repeat {
        println!("Hello, {}!", name);
    }
}

fn print_help() {
    println!("Usage: hello [OPTIONS] [NAME]\n");

    println!("Arguments:");
    println!("[NAME] Name to greet [default: World]\n");

    println!("Options:");
    println!("--upper Convert to uppercase");
    println!("--repeat Repeat greeting N times [default: 1]");
    println!("-h, --help Print help");
}
