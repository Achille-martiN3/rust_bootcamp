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

    let name = cli.name;
    let mut output = format!("Hello, {}!", name);

    if cli.upper {
        output = output.to_uppercase();
    }

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
