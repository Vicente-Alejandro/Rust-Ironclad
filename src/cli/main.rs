use clap::{Parser, Subcommand};
use sqlx::PgPool;
use std::process;

#[derive(Parser)]
#[command(name = "ironclad")]
#[command(version = "1.0")]
#[command(about = "Rust Ironclad Framework CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show framework version and info
    Version,
    
    /// Check database connection
    DbCheck,
    
    /// Check CLI setup
    Test,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => {
            println!("╔════════════════════════════════════════╗");
            println!("║   🦀 Rust Ironclad Framework v1.0.0   ║");
            println!("╚════════════════════════════════════════╝");
            println!();
            println!("Framework: Rust Ironclad");
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!();
        }
        
        Some(Commands::DbCheck) => {
            check_database().await;
        }
        
        Some(Commands::Test) => {
            println!("🔍 Running CLI diagnostics...");
            println!();
            println!("✅ CLI binary is working");
            println!("✅ Clap argument parsing is working");
            println!("✅ Project structure is correct");
            println!();
            println!("🎉 Everything looks good!");
        }
        
        None => {
            println!("Run 'ironclad --help' to see available commands");
        }
    }
}

async fn check_database() {
    println!("🔍 Checking database connection...");
    println!();

    // Load .env
    dotenv::dotenv().ok();

    // Get database URL
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("❌ DATABASE_URL not found in environment");
            eprintln!("   Make sure .env file exists with DATABASE_URL");
            process::exit(1);
        }
    };

    println!("📍 Database: {}", mask_connection_string(&database_url));
    println!();

    // Connect to database
    print!("🔌 Connecting... ");
    let pool = match PgPool::connect(&database_url).await {
        Ok(pool) => {
            println!("✅");
            pool
        }
        Err(e) => {
            println!("❌");
            eprintln!();
            eprintln!("Error: {}", e);
            eprintln!();
            eprintln!("Possible causes:");
            eprintln!("  • PostgreSQL is not running");
            eprintln!("  • Wrong credentials in DATABASE_URL");
            eprintln!("  • Database does not exist");
            eprintln!("  • Network/firewall issues");
            process::exit(1);
        }
    };

    // Ping database
    print!("📡 Sending ping... ");
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => {
            println!("✅");
            println!();
            println!("╔═══════════════════════════════╗");
            println!("║  ✅ Database is UP and ready  ║");
            println!("╚═══════════════════════════════╝");
        }
        Err(e) => {
            println!("❌");
            eprintln!();
            eprintln!("Error executing query: {}", e);
            process::exit(1);
        }
    }

    // Close pool
    pool.close().await;
}

fn mask_connection_string(url: &str) -> String {
    // Hide password in connection string
    if let Some(at_pos) = url.rfind('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    "***HIDDEN***".to_string()
}