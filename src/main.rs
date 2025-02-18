use clap::Parser;

pub mod cli;

fn main() {
    if let Err(err) = cli::Cli::parse().run() {
        eprintln!("{}", err);
    }
}
