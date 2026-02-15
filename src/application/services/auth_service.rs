use std::sync::Arc;
use crate::application::dtos::{AuthResponse, LoginRequest, RegisterUserRequest};
use crate::config::AppConfig;
use crate::domain::entities::User;
use crate::errors::ApiError;
use crate::interfaces::UserRepository;
use crate::utils::auth::hash_password;
use crate::utils::jwt::create_token;

pub struct AuthService {
    user_repository: Arc<dyn UserRepository>,
    config: Arc<AppConfig>,
}

impl AuthService {
    pub fn new(user_repository: Arc<dyn UserRepository>, config: Arc<AppConfig>) -> Self {
        Self {
            user_repository,
            config,
        }
    }

    /// Registrar nuevo usuario
    pub async fn register(&self, request: RegisterUserRequest) -> Result<AuthResponse, ApiError> {
        // Validate unique email
        if self.user_repository.exists_by_email(&request.email).await? {
            return Err(ApiError::Conflict("User already exists".to_string()));
        }
        
        let password_hash = hash_password(&request.password, &self.config)?;
        
        // Crear usuario
        let user = User::new(request.email, request.username, password_hash);
        
        // Persistir
        let created_user = self.user_repository.create(&user).await?;
        
        // Generar token
        let token = create_token(
            &created_user.id,
            &created_user.email,
            &created_user.role.to_string(),
            &self.config,
        )?;
        
        Ok(AuthResponse {
            user: created_user.to_response(),
            token,
        })
    }

    /// Login de usuario
    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, ApiError> {
        // ✅ Buscar usuario por email
        let user = self
            .user_repository
            .get_by_email(&request.email)
            .await?
            .ok_or(ApiError::Unauthorized)?;

        // ✅ Verify that user is active
        if !user.is_active() {
            return Err(ApiError::Forbidden("Account is disabled".to_string()));
        }

        // ✅ Verificar password
        if !crate::utils::auth::verify_password(&request.password, &user.password_hash)? {
            return Err(ApiError::Unauthorized);
        }

        // ✅ Generar JWT
        let token = create_token(
            &user.id,
            &user.email,
            &user.role.to_string(),
            &self.config,
        )?;

        Ok(AuthResponse {
            user: user.to_response(),
            token,
        })
    }

}