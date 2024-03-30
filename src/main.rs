use chrono::Offset;
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
    Download {
        report_id: String,
        fight_id: u32,
    },
    Report {
        report_id: String,
        fight_id: Option<u32>,

        #[arg(
            long,
            short,
            help = "Show all fights in the report (default only encounters)"
        )]
        all: bool,
        #[arg(long, short, help = "Only show kills")]
        kills: bool,
    },
}

fn format_unix_time(time: i64, format: &str) -> String {
    chrono::DateTime::from_timestamp_millis(time)
        .unwrap()
        .with_timezone(&chrono::offset::Local)
        .format(format)
        .to_string()
}

fn format_duration(d: std::time::Duration) -> String {
    let total_seconds = d.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{}h {}m {}s", hours, minutes, seconds)
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        dotenvy::dotenv().ok();
    }

    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Download {
            report_id,
            fight_id,
        } => {
            let wcl = warcraft_logs::WarcraftLogs::new();
            dbg!(wcl.get_combat_log(&report_id, fight_id).await.unwrap());
        }
        Commands::Report {
            report_id,
            all,
            kills,
            fight_id,
        } => {
            let wcl = warcraft_logs::WarcraftLogs::new();
            let report = wcl.get_report(&report_id).await.unwrap();

            match fight_id {
                Some(fight_id) => {
                    let fight = report.fights.iter().find(|f| f.data.id == fight_id);

                    let Some(fight) = fight else {
                        eprintln!("Fight not found");
                        std::process::exit(1);
                    };

                    println!("Report: {}", report.data.title);
                    println!("Fight: {}", fight.data.name);
                    println!(
                        "Date: {}, Duration: {}",
                        format_unix_time(fight.start_time() as i64, "%Y-%m-%d %I:%M%P"),
                        format_duration(fight.duration())
                    );
                    println!(
                        "Boss Health: {}%, Kill: {}",
                        fight
                            .data
                            .boss_percentage
                            .map(|p| p.to_string())
                            .unwrap_or("--".to_string()),
                        if fight.data.kill { "Yes" } else { "No" }
                    );

                    println!("Players:");
                    for player in fight.friendly_players.iter() {
                        println!(
                            "  {: <20}{}",
                            player.name,
                            player.server.clone().unwrap_or("Unknown".to_owned())
                        );
                    }
                }
                None => {
                    println!("Name: {}", report.data.title);
                    println!(
                        "Date: {} to {} (UTC{})",
                        format_unix_time(report.data.start_time as i64, "%Y-%m-%d %I:%M%P"),
                        format_unix_time(report.data.end_time as i64, "%Y-%m-%d %I:%M%P"),
                        chrono::Local::now().offset().fix()
                    );
                    println!();
                    print!("Fights in this report:");
                    if !all {
                        print!(
                            " ({} non-boss fights excluded)",
                            report
                                .fights
                                .iter()
                                .filter(|f| f.data.encounter_id.is_none())
                                .count()
                        );
                    }
                    println!();

                    for fight in report.fights {
                        if !all && fight.data.encounter_id.is_none() {
                            continue;
                        }

                        if kills && !fight.data.kill {
                            continue;
                        }

                        println!(
                            "  [{:0>2}] {}: {} ({}%)",
                            format!("{}", fight.data.id),
                            format_unix_time(fight.start_time() as i64, "%I:%M%P"),
                            fight.data.name,
                            fight
                                .data
                                .boss_percentage
                                .map(|p| p.to_string())
                                .unwrap_or("--".to_string())
                        );
                    }
                }
            }
        }
    }
}
