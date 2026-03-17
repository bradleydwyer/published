use clap::{Parser, Subcommand};
use rmcp::{ServiceExt, transport::stdio};

use published::checker;
use published::mcp::PublishedMcp;
use published::store;
use published::types::Availability;

#[derive(Parser)]
#[command(
    name = "published",
    about = "App store name availability checker",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// App names to check
    names: Vec<String>,

    /// Comma-separated store IDs (default: all stores)
    #[arg(short, long)]
    stores: Option<String>,

    /// Check all stores
    #[arg(short, long)]
    all: bool,

    /// Output results as JSON
    #[arg(short, long)]
    json: bool,

    /// Show per-store detail
    #[arg(short, long)]
    verbose: bool,

    /// Show available stores
    #[arg(long)]
    list_stores: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Start MCP server (stdio transport)
    Mcp,
}

fn resolve_stores(cli: &Cli) -> Vec<store::Store> {
    if cli.all {
        return store::all_stores().to_vec();
    }
    if let Some(ref ids) = cli.stores {
        let ids: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
        return store::stores_by_ids(&ids);
    }
    store::all_stores().to_vec()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(Command::Mcp) = cli.command {
        let server = PublishedMcp::new();
        let service = server.serve(stdio()).await?;
        service.waiting().await?;
        return Ok(());
    }

    if cli.list_stores {
        println!("{:<16} {:<16} PLATFORM", "ID", "NAME");
        println!("{}", "-".repeat(48));
        for s in store::all_stores() {
            println!("{:<16} {:<16} {}", s.id(), s.name(), s.platform(),);
        }
        return Ok(());
    }

    if cli.names.is_empty() {
        eprintln!("Usage: published [OPTIONS] <NAMES>...");
        eprintln!("       published mcp");
        eprintln!("       published --list-stores");
        eprintln!();
        eprintln!("Run 'published --help' for more information.");
        std::process::exit(1);
    }

    let stores = resolve_stores(&cli);
    if stores.is_empty() {
        eprintln!("No matching stores found.");
        std::process::exit(1);
    }

    let results = checker::check_apps(&cli.names, &stores).await;

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        for result in &results {
            println!("{}:", result.name);
            println!(
                "  {} available, {} taken, {} unknown ({}ms)",
                result.summary.available,
                result.summary.taken,
                result.summary.unknown,
                result.elapsed_ms,
            );

            if cli.verbose {
                for store in &result.results {
                    let symbol = match store.available {
                        Availability::Available => "[+]",
                        Availability::Taken => "[-]",
                        Availability::Unknown => "[?]",
                    };
                    println!(
                        "  {} {:<20} {:<12} ({}ms)",
                        symbol, store.store_name, store.available, store.elapsed_ms,
                    );
                }
            } else {
                let available: Vec<&str> = result
                    .results
                    .iter()
                    .filter(|r| r.available == Availability::Available)
                    .map(|r| r.store_name.as_str())
                    .collect();
                let taken: Vec<&str> = result
                    .results
                    .iter()
                    .filter(|r| r.available == Availability::Taken)
                    .map(|r| r.store_name.as_str())
                    .collect();

                if !available.is_empty() {
                    println!("  available: {}", available.join(", "));
                }
                if !taken.is_empty() {
                    println!("  taken: {}", taken.join(", "));
                }
            }
            println!();
        }
    }

    Ok(())
}
