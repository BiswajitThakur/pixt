#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;

#[cfg(not(target_arch = "wasm32"))]
mod cli;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    if let Err(err) = cli::Cli::parse().run() {
        eprintln!("{}", err);
    }
}
