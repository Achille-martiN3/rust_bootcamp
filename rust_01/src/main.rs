use clap::Parser;
use std::collections::HashMap;
use std::io::{self, Read};

/// Count word frequency in text
#[derive(Parser, Debug)]
#[command(name = "wordfreq")]
struct Args {
    text: Option<String>,

    #[arg(long, default_value_t = 10)]
    top: usize,

    #[arg(long, default_value_t = 1)]
    min_length: usize,

    #[arg(long)]
    ignore_case: bool,
}

fn main() {
    let args = Args::parse();

    // 1) Lire depuis argument OU stdin
    let mut input = match args.text {
        Some(t) => t,
        None => {
            // Lecture depuis stdin
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap();
            buffer
        }
    };

    // 2) Option ignore-case
    if args.ignore_case {
        input = input.to_lowercase();
    }

    // 3) Découper en mots
    let words = input.split_whitespace();

    let mut freq = HashMap::new();

    // 4) Compter avec entry() API (ownership + borrow mutable)
    for word in words {
        // Enlever la ponctuation au début et à la fin du mot
        let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric());

        if cleaned.len() >= args.min_length {
            *freq.entry(cleaned.to_string()).or_insert(0) += 1;
        }
    }

    // 5) Transformer en liste et trier par fréquence
    let mut items: Vec<_> = freq.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1)); // tri descendant

    // 6) Afficher top N
    println!("Word frequency:");
    for (word, count) in items.into_iter().take(args.top) {
        println!("{word}: {count}");
    }
}
