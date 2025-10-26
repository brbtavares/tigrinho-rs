use chrono::Utc;
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};

#[derive(Parser)]
#[command(name = "tigrinho-cli", about = "Admin CLI for tigrinho server")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Database URL, default sqlite://tigrinho.db
    #[arg(long, value_parser, env = "DATABASE_URL")]
    database_url: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Rotate server seed to a new secret
    RotateSeed { new_seed: String },
    /// View last N log entries
    ViewLogs {
        #[arg(default_value_t = 20)]
        n: i64,
    },
    /// Export spins to CSV path
    ExportCsv { path: String },
}

#[derive(sqlx::FromRow)]
struct ParamsRow {
    server_seed: String,
    server_seed_hash: String,
    rtp_target: f64,
    paytable_json: String,
    nonce: i64,
}

async fn get_pool(url: Option<String>) -> anyhow::Result<SqlitePool> {
    let url = url.unwrap_or_else(|| "sqlite://tigrinho.db".into());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let pool = get_pool(cli.database_url).await?;

    match cli.command {
        Commands::RotateSeed { new_seed } => {
            let hash = {
                let mut h = Sha256::new();
                h.update(new_seed.as_bytes());
                hex::encode(h.finalize())
            };
            sqlx::query(
                "UPDATE params SET server_seed = ?, server_seed_hash = ?, nonce = 0 WHERE id = 1",
            )
            .bind(new_seed)
            .bind(hash.clone())
            .execute(&pool)
            .await?;
            println!("Rotated server seed. New hash: {}", hash);
        }
        Commands::ViewLogs { n } => {
            let rows = sqlx::query("SELECT id, ts, client_seed, nonce, server_seed_hash, payout FROM spins ORDER BY id DESC LIMIT ?")
                .bind(n)
                .fetch_all(&pool).await?;
            for r in rows {
                let id: i64 = r.get("id");
                let ts: String = r.get("ts");
                let client_seed: String = r.get("client_seed");
                let nonce: i64 = r.get("nonce");
                let server_seed_hash: String = r.get("server_seed_hash");
                let payout: f64 = r.get("payout");
                println!(
                    "#{:>6} {} seed={} nonce={} hash={} payout={}",
                    id, ts, client_seed, nonce, server_seed_hash, payout
                );
            }
        }
        Commands::ExportCsv { path } => {
            let mut wtr = csv::Writer::from_path(&path)?;
            let rows = sqlx::query("SELECT id, ts, client_seed, nonce, server_seed_hash, result_reels_json, payout FROM spins ORDER BY id ASC")
                .fetch_all(&pool).await?;
            let total = rows.len();
            for r in &rows {
                use sqlx::Row;
                wtr.write_record(&[
                    r.get::<i64, _>("id").to_string(),
                    r.get::<String, _>("ts"),
                    r.get::<String, _>("client_seed"),
                    r.get::<i64, _>("nonce").to_string(),
                    r.get::<String, _>("server_seed_hash"),
                    r.get::<String, _>("result_reels_json"),
                    r.get::<f64, _>("payout").to_string(),
                ])?;
            }
            wtr.flush()?;
            println!("Exported {} rows to {}", total, path);
        }
    }

    Ok(())
}
