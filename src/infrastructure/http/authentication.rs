use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use crate::errors::ApiError;
use crate::domain::entities::user::Claims;
use crate::config::AppConfig;

// ============================================
// BASE: Extractor de usuario autenticado
// ============================================
#[derive(Clone)]
pub struct AuthUser(pub Claims);

impl FromRequest for AuthUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(extract_claims(req).map(AuthUser))
    }
}

// ============================================
// ADMIN: Solo usuarios admin
// ============================================
#[derive(Clone)]
pub struct AdminUser(pub Claims);

impl FromRequest for AdminUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            extract_claims(req).and_then(|claims| {
                if claims.is_admin() {
                    Ok(AdminUser(claims))
                } else {
                    Err(ApiError::Forbidden("Admin access required".to_string()))
                }
            })
        )
    }
}

// ============================================
// MODERATOR: Admin o Moderador
// ============================================
#[derive(Clone)]
pub struct ModeratorUser(pub Claims);

impl FromRequest for ModeratorUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            extract_claims(req).and_then(|claims| {
                if claims.has_any_role(&["admin", "moderator"]) {
                    Ok(ModeratorUser(claims))
                } else {
                    Err(ApiError::Forbidden("Moderator access required".to_string()))
                }
            })
        )
    }
}

// ============================================
// PREMIUM: Admin, Moderador o Premium
// ============================================
#[derive(Clone)]
pub struct PremiumUser(pub Claims);

impl FromRequest for PremiumUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            extract_claims(req).and_then(|claims| {
                if claims.has_any_role(&["admin", "moderator", "premium"]) {
                    Ok(PremiumUser(claims))
                } else {
                    Err(ApiError::Forbidden("Premium access required".to_string()))
                }
            })
        )
    }
}

// ============================================
// GENERIC: Flexible role with verification methods
// ============================================
#[derive(Clone)]
pub struct RoleUser(pub Claims);

impl RoleUser {
    pub fn has_role(&self, role: &str) -> bool {
        self.0.has_role(role)
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        self.0.has_any_role(roles)
    }

    pub fn is_admin(&self) -> bool {
        self.0.is_admin()
    }

    pub fn user_id(&self) -> &str {
        &self.0.sub
    }

    pub fn email(&self) -> &str {
        &self.0.email
    }

    pub fn role(&self) -> &str {
        &self.0.role
    }
}

impl FromRequest for RoleUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(extract_claims(req).map(RoleUser))
    }
}

// ============================================
// HELPER: Extract claims DRY
// ============================================
fn extract_claims(req: &HttpRequest) -> Result<Claims, ApiError> {
    let config = req
        .app_data::<actix_web::web::Data<AppConfig>>()
        .ok_or_else(|| ApiError::InternalServerError("Config not found".to_string()))?;

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)?;

    crate::utils::jwt::verify_token(token, config.get_ref())
}

// ============================================
// MACRO: Dynamic role validation
// ============================================
#[macro_export]
macro_rules! require_any_role {
    ($claims:expr, $($role:literal),+ $(,)?) => {
        {
            let allowed = vec![$($role),+];
            if allowed.contains(&$claims.role.as_str()) {
                Ok(())
            } else {
                Err($crate::errors::ApiError::Forbidden(
                    format!("Required roles: {}", allowed.join(", "))
                ))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_is_admin() {
        let claims = Claims::new("user1".to_string(), "user@test.com".to_string(), "admin".to_string(), 1000);
        assert!(claims.is_admin());
    }

    #[test]
    fn test_claims_has_any_role() {
        let claims = Claims::new("user1".to_string(), "user@test.com".to_string(), "moderator".to_string(), 1000);
        assert!(claims.has_any_role(&["admin", "moderator"]));
        assert!(!claims.has_any_role(&["admin", "premium"]));
    }
}
