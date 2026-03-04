use clap::{Parser, Subcommand};
use sqlx::PgPool;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::env;

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
        
        /// Secret token to bypass maintenance mode
        #[arg(long)]
        secret: Option<String>,
        
        /// Template to render (e.g., "emergency" or "emergency::low")
        #[arg(long)]
        render: Option<String>,
        
        /// Force JSON response (no HTML rendering)
        #[arg(long)]
        norender: bool,
        
        /// Redirect all requests to this path
        #[arg(long)]
        redirect: Option<String>,
    },
    
    /// Bring the application out of maintenance mode
    Up,

    /// Storage management commands
    Storage {
        #[command(subcommand)]
        action: StorageAction,
    },
    
    /// Check CLI setup
    Test,
}

#[derive(Subcommand)]
enum StorageAction {
    /// Check storage directories and permissions
    Check,
    
    /// Create public storage symlink
    Link {
        /// Force recreate symlink if exists
        #[arg(long)]
        force: bool,
    },
    
    /// Show storage directories information
    Info,
    
    /// Initialize storage directory structure
    Init,
}

const MAINTENANCE_FILE: &str = "storage/framework/maintenance.json";

// Storage directories configuration
const STORAGE_DIRS: &[&str] = &[
    "storage",
    "storage/app",
    "storage/app/public",
    "storage/framework",
    "storage/framework/cache",
    "storage/logs",
];

#[tokio::main]
async fn main() {
    // ✅ FIX: Detect and change to project root directory
    ensure_project_root();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => {
            println!("╔════════════════════════════════════════╗");
            println!("║   🦀 Rust Ironclad Framework v1.0.0   ║");
            println!("╚════════════════════════════════════════╝");
            println!();
            println!("Framework: Rust Ironclad");
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Working directory: {:?}", env::current_dir().unwrap());
            println!();
        }
        
        Some(Commands::DbCheck) => {
            check_database().await;
        }
        
        Some(Commands::Down { message, retry, secret, render, norender, redirect }) => {
            maintenance_down(message, retry, secret, render, norender, redirect);
        }
        
        Some(Commands::Up) => {
            maintenance_up();
        }

        Some(Commands::Storage { action }) => match action {
            StorageAction::Check => storage_check(),
            StorageAction::Link { force } => storage_link(force),
            StorageAction::Info => storage_info(),
            StorageAction::Init => storage_init(),
        },
        
        Some(Commands::Test) => {
            println!("🔍 Running CLI diagnostics...");
            println!();
            println!("✅ CLI binary is working");
            println!("✅ Clap argument parsing is working");
            println!("✅ Project structure is correct");
            println!("📁 Working directory: {:?}", env::current_dir().unwrap());
            println!();
            println!("🎉 Everything looks good!");
        }
        
        None => {
            println!("Run 'ironclad --help' to see available commands");
        }
    }
}

/// Ensure we're running from the project root
fn ensure_project_root() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    
    // Check if Cargo.toml exists in current directory
    if current_dir.join("Cargo.toml").exists() {
        // Already in project root
        return;
    }
    
    // Try to find project root by going up directories
    let mut search_dir = current_dir.clone();
    loop {
        if search_dir.join("Cargo.toml").exists() {
            // Found project root, change to it
            env::set_current_dir(&search_dir)
                .expect("Failed to change to project root");
            println!("📁 Changed working directory to: {:?}", search_dir);
            return;
        }
        
        // Go up one directory
        match search_dir.parent() {
            Some(parent) => search_dir = parent.to_path_buf(),
            None => {
                eprintln!("❌ Could not find project root (Cargo.toml)");
                eprintln!("   Please run this command from within the project directory");
                process::exit(1);
            }
        }
    }
}

fn maintenance_down(
    message: Option<String>, 
    retry: u32,
    secret: Option<String>,
    render: Option<String>,
    norender: bool,
    redirect: Option<String>,
) {
    println!("🔧 Putting application into maintenance mode...");
    println!();

    // Create storage directory if it doesn't exist
    if let Err(e) = fs::create_dir_all("storage/framework") {
        eprintln!("❌ Failed to create storage directory: {}", e);
        process::exit(1);
    }

    // Validate conflicting options
    if norender && render.is_some() {
        eprintln!("❌ Cannot use both --norender and --render options");
        process::exit(1);
    }
    
    if render.is_some() && redirect.is_some() {
        eprintln!("❌ Cannot use both --render and --redirect options");
        process::exit(1);
    }

    // Create maintenance payload
    let mut maintenance_data = serde_json::json!({
        "time": chrono::Utc::now().timestamp(),
        "message": message.unwrap_or_else(|| "Application is down for maintenance".to_string()),
        "retry": retry,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    // Add optional fields
    if let Some(secret_value) = secret {
        maintenance_data["secret"] = serde_json::json!(secret_value);
        println!("🔑 Secret bypass enabled: /{}", secret_value);
    }

    if norender {
        maintenance_data["norender"] = serde_json::json!(true);
        println!("📋 JSON-only mode (no HTML rendering)");
    }

    if let Some(render_template) = &render {
        // Validate template path exists
        let template_path = parse_template_path(render_template);
        if Path::new(&template_path).exists() {
            println!("✅ Template found: {}", template_path);
        } else {
            eprintln!("⚠️  Warning: Template not found: {}", template_path);
            eprintln!("   Will fall back to default template");
        }
        
        maintenance_data["render"] = serde_json::json!(render_template);
        println!("🎨 HTML template: {}", render_template);
    }

    if let Some(redirect_path) = redirect {
        maintenance_data["redirect"] = serde_json::json!(redirect_path);
        println!("↪️  Redirect to: {}", redirect_path);
    }

    // Write maintenance file
    let maintenance_file_path = Path::new(MAINTENANCE_FILE);
    println!("📝 Writing to: {:?}", maintenance_file_path.canonicalize().unwrap_or(maintenance_file_path.to_path_buf()));
    
    match fs::write(MAINTENANCE_FILE, maintenance_data.to_string()) {
        Ok(_) => {
            println!();
            println!("✅ Application is now in maintenance mode");
            println!();
            println!("   All requests will receive a 503 response");
            if maintenance_data.get("secret").is_some() {
                println!("   Bypass: Add /{} to any URL", maintenance_data["secret"].as_str().unwrap());
            }
            println!("   To bring the application back up, run:");
            println!("   cargo run --bin ironclad -- up");
        }
        Err(e) => {
            eprintln!("❌ Failed to create maintenance file: {}", e);
            process::exit(1);
        }
    }
}

fn parse_template_path(template: &str) -> String {
    if template.contains("::") {
        let parts: Vec<&str> = template.split("::").collect();
        format!("templates/render/down/{}/{}.html", parts[0], parts[1])
    } else {
        format!("templates/render/down/{}/default.html", template)
    }
}

fn maintenance_up() {
    println!("🚀 Bringing application out of maintenance mode...");
    println!();

    if !Path::new(MAINTENANCE_FILE).exists() {
        println!("ℹ️  Application is not in maintenance mode");
        println!("   (No maintenance file found at: {})", MAINTENANCE_FILE);
        return;
    }

    let maintenance_file_path = Path::new(MAINTENANCE_FILE);
    println!("📝 Removing: {:?}", maintenance_file_path.canonicalize().unwrap_or(maintenance_file_path.to_path_buf()));

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

    dotenv::dotenv().ok();

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

    pool.close().await;
}

fn mask_connection_string(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    "***HIDDEN***".to_string()
}

// ==========================================
// STORAGE COMMANDS IMPLEMENTATION
// ==========================================

fn storage_check() {
    println!("🔍 Checking storage directories and permissions...");
    println!();

    let mut all_ok = true;

    for dir in STORAGE_DIRS {
        let path = Path::new(dir);
        
        print!("  📁 {} ... ", dir);
        
        if !path.exists() {
            println!("❌ NOT FOUND");
            all_ok = false;
            continue;
        }

        let can_read = path.read_dir().is_ok();
        let test_file = path.join(".permission_test");
        let can_write = fs::write(&test_file, "test").is_ok();
        if can_write {
            let _ = fs::remove_file(&test_file);
        }

        match (can_read, can_write) {
            (true, true) => println!("✅ OK (R/W)"),
            (true, false) => {
                println!("⚠️  READ ONLY");
                all_ok = false;
            }
            (false, _) => {
                println!("❌ NO ACCESS");
                all_ok = false;
            }
        }
    }

    // Check storage symlink (cambiado de public/storage a static/storage)
    let symlink_path = Path::new("static/storage");
    print!("\n  🔗 static/storage symlink ... ");
    
    if symlink_path.exists() {
        if symlink_path.is_symlink() || symlink_path.read_link().is_ok() {
            println!("✅ EXISTS");
        } else {
            println!("⚠️  EXISTS (not a symlink)");
        }
    } else {
        println!("❌ NOT FOUND");
        println!("     Run: cargo run --bin ironclad -- storage link");
        all_ok = false;
    }

    println!();
    if all_ok {
        println!("╔═══════════════════════════════════╗");
        println!("║  ✅ All storage checks passed!   ║");
        println!("╚═══════════════════════════════════╝");
    } else {
        println!("╔═══════════════════════════════════════════════╗");
        println!("║  ⚠️  Some storage checks failed              ║");
        println!("║     Run: cargo run --bin ironclad -- storage init  ║");
        println!("╚═══════════════════════════════════════════════╝");
    }
}

/// Create storage symlink in static directory (not public)
fn storage_link(force: bool) {
    println!("🔗 Creating static/storage symlink...");
    println!();

    let target = Path::new("storage/app/public");
    let link = Path::new("static/storage");  // ✅ Cambiar de public/storage a static/storage

    // Check if target exists
    if !target.exists() {
        eprintln!("❌ Target directory not found: storage/app/public");
        eprintln!("   Run: cargo run --bin ironclad -- storage init");
        process::exit(1);
    }

    // Check if link already exists
    if link.exists() {
        if !force {
            eprintln!("❌ Symlink already exists: static/storage");
            eprintln!("   Use --force to recreate it");
            process::exit(1);
        }
        
        println!("⚠️  Removing existing symlink...");
        if let Err(e) = fs::remove_file(link).or_else(|_| fs::remove_dir_all(link)) {
            eprintln!("❌ Failed to remove existing symlink: {}", e);
            process::exit(1);
        }
    }

    // Create parent directory if needed
    if let Some(parent) = link.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("❌ Failed to create static directory: {}", e);
                process::exit(1);
            }
        }
    }

    // Create symlink (platform-specific)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        
        let target_abs = env::current_dir().unwrap().join(target);
        match symlink(&target_abs, link) {
            Ok(_) => {
                println!("✅ Symlink created successfully!");
                println!();
                println!("   From: static/storage");
                println!("   To:   {}", target_abs.display());
            }
            Err(e) => {
                eprintln!("❌ Failed to create symlink: {}", e);
                process::exit(1);
            }
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        
        let target_abs = env::current_dir().unwrap().join(target);
        match symlink_dir(&target_abs, link) {
            Ok(_) => {
                println!("✅ Symlink created successfully!");
                println!();
                println!("   From: static\\storage");
                println!("   To:   {}", target_abs.display());
                println!();
                println!("ℹ️  Note: On Windows, you may need Administrator privileges");
            }
            Err(e) => {
                eprintln!("❌ Failed to create symlink: {}", e);
                eprintln!();
                eprintln!("💡 On Windows, you may need to:");
                eprintln!("   1. Run as Administrator");
                eprintln!("   2. Enable Developer Mode in Windows Settings");
                process::exit(1);
            }
        }
    }
}

fn storage_info() {
    println!("📊 Storage Directory Information");
    println!();

    let current_dir = env::current_dir().unwrap();
    println!("📍 Base directory: {}", current_dir.display());
    println!();

    println!("📁 Storage structure:");
    println!();

    for dir in STORAGE_DIRS {
        let path = Path::new(dir);
        let full_path = current_dir.join(path);
        
        if path.exists() {
            let metadata = fs::metadata(path).ok();
            let size = metadata.map(|m| m.len()).unwrap_or(0);
            
            let file_count = if path.is_dir() {
                fs::read_dir(path)
                    .map(|entries| entries.count())
                    .unwrap_or(0)
            } else {
                0
            };

            println!("  ✅ {}", dir);
            println!("     Path: {}", full_path.display());
            println!("     Items: {}", file_count);
            println!("     Size: {} bytes", size);
            println!();
        } else {
            println!("  ❌ {} (not found)", dir);
            println!();
        }
    }

    // Check symlink (static/storage)
    let symlink_path = Path::new("static/storage");
    if symlink_path.exists() {
        println!("  🔗 static/storage");
        if let Ok(target) = symlink_path.read_link() {
            println!("     Target: {}", target.display());
        }
        println!();
    }

    println!("💾 Storage base: storage/");
    println!("🌐 User uploads: static/storage → storage/app/public");  
    println!("🔧 Framework files: storage/framework/");
    println!("📝 Logs: storage/logs/");
    println!("🎨 Static assets: static/framework/"); 
}

/// Initialize storage directory structure
fn storage_init() {
    println!("🚀 Initializing storage directory structure...");
    println!();

    let mut created = Vec::new();
    let mut existed = Vec::new();

    for dir in STORAGE_DIRS {
        let path = Path::new(dir);
        
        if path.exists() {
            existed.push(dir.to_string());
        } else {
            match fs::create_dir_all(path) {
                Ok(_) => {
                    println!("  ✅ Created: {}", dir);
                    created.push(dir.to_string());
                }
                Err(e) => {
                    eprintln!("  ❌ Failed to create {}: {}", dir, e);
                    process::exit(1);
                }
            }
        }
    }

    // Create .gitignore files
    create_gitignore_files();

    println!();
    println!("📊 Summary:");
    println!("  ✅ Created: {} directories", created.len());
    println!("  ⏭️  Existed: {} directories", existed.len());
    println!();

    if !created.is_empty() {
        println!("✅ Storage structure initialized successfully!");
        println!();
        println!("Next steps:");
        println!("  1. Run: cargo run --bin ironclad -- storage link");
        println!("  2. Run: cargo run --bin ironclad -- storage check");
    } else {
        println!("ℹ️  All directories already exist");
    }
}

/// Create .gitignore files for storage directories
fn create_gitignore_files() {
    let gitignores = vec![
        ("storage/app/.gitignore", "*\n!public/\n!.gitignore"),
        ("storage/app/public/.gitignore", "*\n!.gitignore"),
        ("storage/framework/.gitignore", "*\n!cache/\n!.gitignore"),
        ("storage/framework/cache/.gitignore", "*\n!.gitignore"),
        ("storage/logs/.gitignore", "*\n!.gitignore"),
    ];

    for (path, content) in gitignores {
        if !Path::new(path).exists() {
            if let Err(e) = fs::write(path, content) {
                eprintln!("  ⚠️  Failed to create {}: {}", path, e);
            }
        }
    }
}
