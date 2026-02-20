use actix_web::{
    body::{BoxBody, EitherBody}, 
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, Ready};
use futures::Future;
use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

const MAINTENANCE_FILE: &str = "storage/framework/maintenance.json";

pub struct MaintenanceMode;

impl<S, B> Transform<S, ServiceRequest> for MaintenanceMode
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Transform = MaintenanceModeMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MaintenanceModeMiddleware { service })
    }
}

pub struct MaintenanceModeMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MaintenanceModeMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if Path::new(MAINTENANCE_FILE).exists() {
            let maintenance_data = fs::read_to_string(MAINTENANCE_FILE)
                .ok()
                .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok());

            let response = if let Some(data) = maintenance_data {
                HttpResponse::ServiceUnavailable()
                    .insert_header((
                        "Retry-After",
                        data["retry"].as_u64().unwrap_or(60).to_string(),
                    ))
                    .json(serde_json::json!({
                        "error": "Service Unavailable",
                        "message": data["message"].as_str().unwrap_or("Application is down for maintenance"),
                        "status": 503,
                        "retry_after": data["retry"].as_u64().unwrap_or(60),
                    }))
            } else {
                HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": "Service Unavailable",
                    "message": "Application is down for maintenance",
                    "status": 503,
                }))
            };

            let (http_req, _) = req.into_parts();
            return Box::pin(async move {
                Ok(ServiceResponse::new(http_req, response).map_into_right_body())
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}