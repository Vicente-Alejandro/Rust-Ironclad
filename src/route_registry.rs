/// A single route definition — pure data, no Actix dependency.
/// Both the HTTP server and the CLI binary can import this.
#[derive(Debug, Clone)]
pub struct RouteDefinition {
    pub method: &'static str,
    pub uri:    &'static str,
}

impl RouteDefinition {
    const fn new(method: &'static str, uri: &'static str) -> Self {
        Self { method, uri }
    }
}

/// All application routes declared as static data.
/// This is the single source of truth consumed by:
///   - `routes/mod.rs`  → registers routes with Actix-web
///   - `ironclad route list` → prints the route table
pub fn get_routes() -> Vec<RouteDefinition> {
    vec![
        // ── Static files ────────────────────────────────────────────────
        RouteDefinition::new("GET",    "/static/{filename:.*}"),

        // ── Docs ────────────────────────────────────────────────────────
        RouteDefinition::new("GET",    "/api/docs"),

        // ── Auth ────────────────────────────────────────────────────────
        RouteDefinition::new("POST",   "/api/auth/register"),
        RouteDefinition::new("POST",   "/api/auth/login"),
        RouteDefinition::new("GET",    "/api/auth/verify-admin"),

        // ── User ────────────────────────────────────────────────────────
        RouteDefinition::new("GET",    "/api/user/profile"),
        RouteDefinition::new("GET",    "/api/user/all"),
        RouteDefinition::new("GET",    "/api/user/{id}"),

        // ── No-auth ─────────────────────────────────────────────────────
        RouteDefinition::new("GET",    "/api/noauth/users"),

        // ── Test Items ──────────────────────────────────────────────────
        RouteDefinition::new("POST",   "/api/test-items"),
        RouteDefinition::new("GET",    "/api/test-items"),
        RouteDefinition::new("GET",    "/api/test-items/{id}"),
        RouteDefinition::new("PUT",    "/api/test-items/{id}"),
        RouteDefinition::new("DELETE", "/api/test-items/{id}"),

        // ── Administration ──────────────────────────────────────────────
        RouteDefinition::new("GET",    "/api/administration/health"),
        RouteDefinition::new("GET",    "/api/administration/uptime"),
        RouteDefinition::new("GET",    "/api/administration/system"),
        RouteDefinition::new("GET",    "/api/administration/system-json"),
    ]
}