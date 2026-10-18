
mod app;
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
    verbose: Option<bool>
}

fn main() {

    let loaded = dotenv::dotenv();
    if loaded.is_err() {
        println!("{}", "[Error]: Env loading failed ".red());
        return;
    }

    let args = Args::parse();
    let verbose = args.verbose.unwrap_or(false);
    init_tracing(verbose);

    debug!("Welcome on the voice recognition game !");

    if let Err(e) = App::launch() {
        println!("{}", format!("[Error]: {e}").red());
    }

    
}
