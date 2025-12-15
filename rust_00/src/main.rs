use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "hello",
    about = "Petit programme de salutation en ligne de commande",
    disable_help_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[arg(value_name = "NAME", default_value = "World")]
    name: String,

    #[arg(long)]
    upper: bool,

    #[arg(long, default_value_t = 1, value_name = "N")]
    repeat: u32,

    #[arg(short = 'h', long = "help")]
    help: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.help {
        print_help();
        return;
    }

    let name = if cli.upper {
        cli.name.to_uppercase()
    } else {
        cli.name
    };
    let greeting = if cli.upper { "HELLO" } else { "Hello" };
    let output = format!("{}, {}!", greeting, name);

    for _ in 0..cli.repeat {
        println!("{}", output);
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
