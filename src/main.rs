use clap::Parser;

mod cli;
pub mod output;

fn main() {
    if let Err(err) = cli::Cli::parse().run() {
        eprintln!("{}", err);
    }
}
