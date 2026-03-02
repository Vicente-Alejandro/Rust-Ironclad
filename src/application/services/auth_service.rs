use std::sync::Arc;
use crate::application::dtos::{AuthResponse, LoginRequest, RegisterUserRequest};
use crate::config::AppConfig;
use crate::domain::entities::User;
use crate::domain::value_objects::{EmailAddress, Username};
use crate::errors::ApiError;
use crate::interfaces::UserRepository;
use crate::utils::auth::hash_password;
use crate::utils::jwt::create_token;
use crate::shared::validator::validate_strong_password;

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
        // 1. Validar reglas del Application Service (Ej: Password fuerte en texto plano)
        if validate_strong_password(&request.password).is_err() {
            return Err(ApiError::ValidationError("Password does not meet security requirements".to_string()));
        }

        // 2. Validate asynchronous business logic rules (I/O)
        if self.user_repository.exists_by_email(&request.email).await? {
            return Err(ApiError::Conflict("User already exists".to_string()));
        }
        
        // 3. Create Value Objects (Produce DomainError which is automatically converted to ApiError by the impl From)
        let email_vo = EmailAddress::new(request.email)?;
        let username_vo = Username::new(request.username)?;
        
        let password_hash = hash_password(&request.password, &self.config)?;
        
        // 4. Crear entidad de forma segura
        let user = User::new(email_vo, username_vo, password_hash)?;
        
        // 5. Persistir
        let created_user = self.user_repository.create(&user).await?;
        
        // 6. Generar token
        let token = create_token(
            &created_user.id,
            created_user.email.as_str(),
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
        let user = self
            .user_repository
            .get_by_email(&request.email)
            .await?
            .ok_or(ApiError::Unauthorized)?;

        if !user.is_active() {
            return Err(ApiError::Forbidden("Account is disabled".to_string()));
        }

        if !crate::utils::auth::verify_password(&request.password, &user.password_hash)? {
            return Err(ApiError::Unauthorized);
        }

        let token = create_token(
            &user.id,
            user.email.as_str(),
            &user.role.to_string(),
            &self.config,
        )?;

        Ok(AuthResponse {
            user: user.to_response(),
            token,
        })
    }
}