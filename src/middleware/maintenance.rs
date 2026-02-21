use actix_web::{
    body::{BoxBody, EitherBody},
    cookie::Cookie, // Importación necesaria para la cookie de bypass
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
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
        if !Path::new(MAINTENANCE_FILE).exists() {
            // No maintenance mode, proceed normally
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        // Read maintenance data
        let maintenance_data = fs::read_to_string(MAINTENANCE_FILE)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok());

        if maintenance_data.is_none() {
            // Invalid maintenance file, proceed normally
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }

        let data = maintenance_data.unwrap();
        let secret_opt = data.get("secret").and_then(|s| s.as_str());

        // ========================================
        // FIX 1: Secret bypass (Cookie Validation)
        // ========================================
        if let Some(secret) = secret_opt {
            // Check if the user already has a valid bypass cookie
            if let Some(cookie) = req.cookie("maintenance_bypass") {
                if cookie.value() == secret {
                    // Valid cookie, bypass maintenance mode completely
                    let fut = self.service.call(req);
                    return Box::pin(async move {
                        let res = fut.await?;
                        Ok(res.map_into_left_body())
                    });
                }
            }

            // Check if the user is trying to authenticate via URL suffix
            let path = req.path().to_string();
            let secret_suffix = format!("/{}", secret);
            
            if path.ends_with(&secret_suffix) {
                // Remove secret from path to determine where to redirect
                let new_path = path.trim_end_matches(&secret_suffix);
                let redirect_to = if new_path.is_empty() { "/" } else { new_path }.to_string();
                
                // Create a session cookie for the bypass
                let mut cookie = Cookie::new("maintenance_bypass", secret);
                cookie.set_path("/");
                cookie.set_http_only(true);

                let (http_req, _) = req.into_parts();
                
                // Redirect to the clean URL, setting the cookie
                let response = HttpResponse::TemporaryRedirect()
                    .insert_header((header::LOCATION, redirect_to.as_str()))
                    .cookie(cookie)
                    .finish();

                return Box::pin(async move {
                    Ok(ServiceResponse::new(http_req, response).map_into_right_body())
                });
            }
        }

        // ========================================
        // FIX 2: Redirect (avoid infinite loop)
        // ========================================
        if let Some(redirect) = data.get("redirect").and_then(|r| r.as_str()) {
            let current_path = req.path();
            
            // Avoid redirect loop - don't redirect if already on target path
            if current_path != redirect {
                let (http_req, _) = req.into_parts();
                let response = HttpResponse::TemporaryRedirect()
                    .insert_header(("Location", redirect))
                    .finish();
                
                return Box::pin(async move {
                    Ok(ServiceResponse::new(http_req, response).map_into_right_body())
                });
            } else {
                // Already on target path, show maintenance page
                let (http_req, _) = req.into_parts();
                let response = render_json_response(&data);
                return Box::pin(async move {
                    Ok(ServiceResponse::new(http_req, response).map_into_right_body())
                });
            }
        }

        // ========================================
        // Response type determination
        // ========================================
        let is_browser_request = is_browser(&req);
        let norender = data.get("norender").and_then(|n| n.as_bool()).unwrap_or(false);
        let has_render = data.get("render").is_some();

        let response = if norender {
            // Force JSON even for browser requests
            render_json_response(&data)
        } else if is_browser_request && has_render {
            // Render custom HTML template
            render_custom_html(&data)
        } else if is_browser_request {
            // Default HTML template
            render_default_html(&data)
        } else {
            // JSON response for API requests
            render_json_response(&data)
        };

        let (http_req, _) = req.into_parts();
        Box::pin(async move {
            Ok(ServiceResponse::new(http_req, response).map_into_right_body())
        })
    }
}

// Helper: Check if request is from a browser
fn is_browser(req: &ServiceRequest) -> bool {
    if let Some(accept) = req.headers().get("Accept") {
        if let Ok(accept_str) = accept.to_str() {
            return accept_str.contains("text/html");
        }
    }
    false
}

// Helper: Render JSON response
fn render_json_response(data: &serde_json::Value) -> HttpResponse {
    let retry = data["retry"].as_u64().unwrap_or(60);
    HttpResponse::ServiceUnavailable()
        .insert_header(("Retry-After", retry.to_string()))
        .json(serde_json::json!({
            "error": "Service Unavailable",
            "message": data["message"].as_str().unwrap_or("Application is down for maintenance"),
            "status": 503,
            "retry_after": retry,
        }))
}

// Helper: Render default HTML template
fn render_default_html(data: &serde_json::Value) -> HttpResponse {
    let template = include_str!("../templates/render/down/default.html");
    
    let message = data["message"].as_str().unwrap_or("Application is down for maintenance");
    let retry = data["retry"].as_u64().unwrap_or(60);
    
    let html = template
        .replace("{{MESSAGE}}", message)
        .replace("{{RETRY}}", &retry.to_string());
    
    HttpResponse::ServiceUnavailable()
        .insert_header(("Content-Type", "text/html; charset=utf-8"))
        .insert_header(("Retry-After", retry.to_string()))
        .body(html)
}

// Helper: Render custom HTML template
fn render_custom_html(data: &serde_json::Value) -> HttpResponse {
    let template_name = data["render"].as_str().unwrap_or("");
    
    // Parse template path
    let template_path = if template_name.is_empty() {
        "templates/render/down/default.html".to_string()
    } else if template_name.contains("::") {
        let parts: Vec<&str> = template_name.split("::").collect();
        if parts.len() == 2 {
            format!("templates/render/down/{}/{}.html", parts[0], parts[1])
        } else {
            "templates/render/down/default.html".to_string()
        }
    } else {
        format!("templates/render/down/{}/default.html", template_name)
    };
    
    // Try to read custom template
    let template = if let Ok(content) = fs::read_to_string(&template_path) {
        content
    } else {
        // Fallback to default if template not found
        tracing::warn!("Template not found: {}, using default", template_path);
        include_str!("../templates/render/down/default.html").to_string()
    };
    
    let message = data["message"].as_str().unwrap_or("Application is down for maintenance");
    let retry = data["retry"].as_u64().unwrap_or(60);
    let timestamp = data["created_at"].as_str().unwrap_or("N/A");
    
    let html = template
        .replace("{{MESSAGE}}", message)
        .replace("{{RETRY}}", &retry.to_string())
        .replace("{{TIMESTAMP}}", timestamp);
    
    HttpResponse::ServiceUnavailable()
        .insert_header(("Content-Type", "text/html; charset=utf-8"))
        .insert_header(("Retry-After", retry.to_string()))
        .body(html)
}