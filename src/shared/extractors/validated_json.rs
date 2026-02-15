use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use futures::future::{ready, LocalBoxFuture, FutureExt};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::errors::ApiError;

/// Extractor that automatically validates DTOs
pub struct ValidatedJson<T>(pub T);

impl<T> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + 'static,
{
    type Error = ApiError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let json_fut = web::Json::<T>::from_request(req, payload);

        async move {
            match json_fut.await {
                Ok(json) => {
                    // Validar el objeto
                    match json.0.validate() {
                        Ok(_) => Ok(ValidatedJson(json.0)),
                        Err(errors) => {
                            // Format validation errors
                            let error_messages = errors
                                .field_errors()
                                .iter()
                                .map(|(field, errors)| {
                                    let messages: Vec<String> = errors
                                        .iter()
                                        .filter_map(|e| e.message.as_ref().map(|m| m.to_string()))
                                        .collect();
                                    format!("{}: {}", field, messages.join(", "))
                                })
                                .collect::<Vec<_>>()
                                .join("; ");

                            Err(ApiError::ValidationError(error_messages))
                        }
                    }
                }
                Err(e) => Err(ApiError::ValidationError(format!("Invalid JSON: {}", e))),
            }
        }
        .boxed_local()
    }
}