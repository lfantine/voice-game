
mod app;
#[cfg(feature = "train")]
mod train;
mod voice;

use std::println;
use colored::Colorize;

use clap::Parser;
use tracing_subscriber;
use tracing::{Level, debug};

use app::App;


fn init_tracing(verbose: bool) {
    let _ = tracing_subscriber::fmt()
        .with_max_level(if verbose { Level::DEBUG } else { Level::WARN })
        .init();
}

/// Simple game about making something move by recognizing pattern in audio wave
#[derive(Parser, Debug)]
#[command(about, version)]
struct Args {
    /// Verbose toggle
    #[arg(short, long)]
    verbose: bool,

    /// Train the voice model on the recorded examples instead of launching the game
    #[cfg(feature = "train")]
    #[arg(long)]
    train: bool,
}

fn main() {

    let loaded = dotenv::dotenv();
    if loaded.is_err() {
        println!("{}", "[Error]: Env loading failed ".red());
        return;
    }

    let args = Args::parse();
    init_tracing(args.verbose);

    #[cfg(feature = "train")]
    {
        if args.train {
            if let Err(e) = train::run() {
                println!("{}", format!("[Error]: {e}").red());
            }
            return;
        }
    }

    debug!("Welcome on the voice recognition game !");

    if let Err(e) = App::launch() {
        println!("{}", format!("[Error]: {e}").red());
    }

    
}
