use actix_web::{web, HttpResponse};
use std::sync::Arc;

use crate::application::dtos::{LoginRequest, RegisterUserRequest};
use crate::application::services::AuthService;
use crate::errors::ApiResult;
use crate::infrastructure::http::authentication::{AdminUser, AuthUser};
use crate::shared::ValidatedJson;  

pub struct AuthController;

impl AuthController {
    /// Register with automatic validation
    pub async fn register(
        service: web::Data<Arc<AuthService>>,
        req: ValidatedJson<RegisterUserRequest>,  
    ) -> ApiResult<HttpResponse> {
        let response = service.register(req.0).await?;  
        Ok(HttpResponse::Created().json(response))
    }

    /// Login with automatic validation
    pub async fn login(
        service: web::Data<Arc<AuthService>>,
        req: ValidatedJson<LoginRequest>,  
    ) -> ApiResult<HttpResponse> {
        let response = service.login(req.0).await?;
        Ok(HttpResponse::Ok().json(response))
    }

    pub async fn verify_admin(
        _service: web::Data<Arc<AuthService>>,
        _admin: AdminUser,
    ) -> ApiResult<HttpResponse> {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "You are an admin!",
            "verified": true
        })))
    }
}