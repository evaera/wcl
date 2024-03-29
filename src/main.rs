use clap::{Parser, Subcommand};

pub mod combat_log;
pub mod warcraft_logs;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Download { report_id: String, fight_id: u32 },
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Download {
            report_id,
            fight_id,
        } => {
            let wcl = warcraft_logs::WarcraftLogs::new();
            dbg!(wcl.get_report(&report_id, fight_id).await.unwrap());
        }
    }
}
