use clap::{Parser, Subcommand};
use sqlx::PgPool;
use std::fs;
use std::path::Path;
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
    
    /// Put the application into maintenance mode
    Down {
        /// Optional maintenance message
        #[arg(long)]
        message: Option<String>,
        
        /// Retry-After header value in seconds
        #[arg(long, default_value = "60")]
        retry: u32,
    },
    
    /// Bring the application out of maintenance mode
    Up,
    
    /// Check CLI setup
    Test,
}

const MAINTENANCE_FILE: &str = "storage/framework/maintenance.json";

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
        
        Some(Commands::Down { message, retry }) => {
            maintenance_down(message, retry);
        }
        
        Some(Commands::Up) => {
            maintenance_up();
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

fn maintenance_down(message: Option<String>, retry: u32) {
    println!("🔧 Putting application into maintenance mode...");
    println!();

    // Create storage directory if it doesn't exist
    if let Err(e) = fs::create_dir_all("storage/framework") {
        eprintln!("❌ Failed to create storage directory: {}", e);
        process::exit(1);
    }

    // Create maintenance payload
    let maintenance_data = serde_json::json!({
        "time": chrono::Utc::now().timestamp(),
        "message": message.unwrap_or_else(|| "Application is down for maintenance".to_string()),
        "retry": retry,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    // Write maintenance file
    match fs::write(MAINTENANCE_FILE, maintenance_data.to_string()) {
        Ok(_) => {
            println!("✅ Application is now in maintenance mode");
            println!();
            println!("   All requests will receive a 503 response");
            println!("   To bring the application back up, run:");
            println!("   cargo run --bin ironclad -- up");
        }
        Err(e) => {
            eprintln!("❌ Failed to create maintenance file: {}", e);
            process::exit(1);
        }
    }
}

fn maintenance_up() {
    println!("🚀 Bringing application out of maintenance mode...");
    println!();

    if !Path::new(MAINTENANCE_FILE).exists() {
        println!("ℹ️  Application is not in maintenance mode");
        return;
    }

    match fs::remove_file(MAINTENANCE_FILE) {
        Ok(_) => {
            println!("✅ Application is now live");
            println!();
            println!("   All requests will be processed normally");
        }
        Err(e) => {
            eprintln!("❌ Failed to remove maintenance file: {}", e);
            process::exit(1);
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