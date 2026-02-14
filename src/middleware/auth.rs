use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures::future::{Ready, ready};
use crate::errors::ApiError;
use crate::models::user::Claims;
use crate::utils::jwt::verify_token;
use crate::config::AppConfig;

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