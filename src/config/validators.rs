use crate::config::AppConfig;

/// Validates security configuration on startup
pub fn validate_security_config(config: &AppConfig) {
    validate_bcrypt_cost(config);
    validate_jwt_config(config);
}

/// Validate bcrypt cost based on environment
fn validate_bcrypt_cost(config: &AppConfig) {
    if config.bcrypt.cost < 4 {
        tracing::error!(
            "🔴 BCRYPT_COST={} is CRITICALLY LOW for security (minimum: 4)", 
            config.bcrypt.cost
        );
    } else if config.bcrypt.cost < 8 {
        tracing::warn!(
            "⚠️  BCRYPT_COST={} is LOW for security (recommended: 8+)", 
            config.bcrypt.cost
        );
    }

    match config.server.env.as_str() {
        "production" => validate_production_bcrypt(config),
        "staging" => validate_staging_bcrypt(config),
        "development" => validate_development_bcrypt(config),
        _ => {
            tracing::warn!("⚠️  Unknown environment: {}", config.server.env);
        }
    }
}

fn validate_production_bcrypt(config: &AppConfig) {
    if config.bcrypt.cost < 10 {
        tracing::error!(
            "🔴 BCRYPT_COST={} is TOO LOW for production (minimum: 10, recommended: 12)", 
            config.bcrypt.cost
        );
        tracing::error!("   Production deployment blocked for security reasons");
        std::process::exit(1);  
    } else if config.bcrypt.cost >= 12 {
        tracing::info!(
            "✅ BCRYPT_COST={} is IDEAL for production", 
            config.bcrypt.cost
        );
    } else {
        tracing::warn!(
            "⚠️  BCRYPT_COST={} is acceptable for production (recommended: 12)", 
            config.bcrypt.cost
        );
    }
}

fn validate_staging_bcrypt(config: &AppConfig) {
    if config.bcrypt.cost < 8 {
        tracing::warn!(
            "⚠️  BCRYPT_COST={} is too low for staging (minimum: 8)", 
            config.bcrypt.cost
        );
    } else if config.bcrypt.cost >= 10 {
        tracing::info!("✅ BCRYPT_COST={} is good for staging", config.bcrypt.cost);
    }
}

fn validate_development_bcrypt(config: &AppConfig) {
    if config.bcrypt.cost < 6 {
        tracing::warn!(
            "⚠️  BCRYPT_COST={} may be too low even for development", 
            config.bcrypt.cost
        );
    } else {
        tracing::info!(
            "ℹ️  BCRYPT_COST={} (development mode - faster hashing)", 
            config.bcrypt.cost
        );
    }
}

/// Validate JWT configuration
fn validate_jwt_config(config: &AppConfig) {
    if config.server.env == "production" {
        if config.jwt.secret.len() < 32 {
            tracing::error!("🔴 JWT_SECRET is too short for production (minimum: 32 characters)");
            std::process::exit(1);
        }
        
        if config.jwt.secret.contains("change") || config.jwt.secret.contains("secret") {
            tracing::error!("🔴 JWT_SECRET appears to be a default value - change it!");
            std::process::exit(1);
        }
        
        tracing::info!("✅ JWT configuration is secure for production");
    }
}

/// Validate all configuration settings
pub fn validate_all_config(config: &AppConfig) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Bcrypt validation
    if config.bcrypt.cost < 4 {
        errors.push(format!("BCRYPT_COST={} is too low (minimum: 4)", config.bcrypt.cost));
    }

    // JWT validation
    if config.server.env == "production" && config.jwt.secret.len() < 32 {
        errors.push("JWT_SECRET is too short for production".to_string());
    }

    // Postgress validation
    if config.db_postgres.postgres_url.is_empty() {
        errors.push("DATABASE_URL is not set".to_string());
    }

    // Mysql MariaDB validation TODO
    // if config.db_mysql.mysql_url.is_empty() {
    //     errors.push("MYSQL_URL is not set".to_string());
    // }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}