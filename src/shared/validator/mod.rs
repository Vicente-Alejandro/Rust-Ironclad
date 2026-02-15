use validator::ValidationError;

/// Validador personalizado para passwords fuertes
pub fn validate_strong_password(password: &str) -> Result<(), ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if has_uppercase && has_lowercase && has_digit && has_special {
        Ok(())
    } else {
        Err(ValidationError::new("weak_password"))
    }
}

/// Validator for usernames (only alphanumeric and dashes)
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    let is_valid = username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-');

    if is_valid {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_username"))
    }
}