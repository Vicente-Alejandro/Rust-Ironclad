use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use crate::errors::ApiError;
use crate::models::user::Claims;
use crate::utils::jwt::verify_token;
use crate::config::AppConfig;

/// User authenticated - any role can access
pub struct AuthUser(pub Claims);

impl FromRequest for AuthUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let config = req.app_data::<actix_web::web::Data<AppConfig>>();

        if config.is_none() {
            return ready(Err(ApiError::InternalServerError(
                "Config not found".to_string(),
            )));
        }

        let config = config.unwrap().get_ref();

        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|h| h.to_string());

        let result = match token {
            Some(token) => match verify_token(&token, config) {
                Ok(claims) => Ok(AuthUser(claims)),
                Err(e) => Err(e),
            },
            None => Err(ApiError::Unauthorized),
        };

        ready(result)
    }
}

/// User with specific role - returns error if they don't have the required role
pub struct RoleGuard {
    pub claims: Claims,
    required_roles: Vec<String>,
}

impl RoleGuard {
    pub fn new(claims: Claims, required_roles: Vec<String>) -> Result<Self, ApiError> {
        if required_roles.contains(&claims.role) {
            Ok(RoleGuard {
                claims,
                required_roles,
            })
        } else {
            Err(ApiError::Forbidden)
        }
    }

    pub fn get_claims(&self) -> &Claims {
        &self.claims
    }
}

/// Macro to create role guards easily, example: require_roles!(auth, "admin", "editor")
#[macro_export]
macro_rules! require_roles {
    ($auth:expr, $($role:expr),+) => {
        {
            let required_roles = vec![$(String::from($role)),+];
            $crate::middleware::auth::RoleGuard::new($auth.0.clone(), required_roles)
        }
    };
}

/// Extractor to enforce a specific role (example: only admin)
pub struct AdminUser(pub Claims);

impl FromRequest for AdminUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let config = req.app_data::<actix_web::web::Data<AppConfig>>();

        if config.is_none() {
            return ready(Err(ApiError::InternalServerError(
                "Config not found".to_string(),
            )));
        }

        let config = config.unwrap().get_ref();

        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|h| h.to_string());

        let result = match token {
            Some(token) => match verify_token(&token, config) {
                Ok(claims) => {
                    if claims.role == "admin" {
                        Ok(AdminUser(claims))
                    } else {
                        Err(ApiError::Forbidden)
                    }
                }
                Err(e) => Err(e),
            },
            None => Err(ApiError::Unauthorized),
        };

        ready(result)
    }
}

pub struct RoleUser {
    pub claims: Claims,
}

impl RoleUser {
    pub fn has_role(&self, role: &str) -> bool {
        self.claims.role == role
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.contains(&self.claims.role.as_str())
    }
}

impl FromRequest for RoleUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let config = req.app_data::<actix_web::web::Data<AppConfig>>();

        if config.is_none() {
            return ready(Err(ApiError::InternalServerError(
                "Config not found".to_string(),
            )));
        }

        let config = config.unwrap().get_ref();

        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|h| h.to_string());

        let result = match token {
            Some(token) => match verify_token(&token, config) {
                Ok(claims) => Ok(RoleUser { claims }),
                Err(e) => Err(e),
            },
            None => Err(ApiError::Unauthorized),
        };

        ready(result)
    }
}