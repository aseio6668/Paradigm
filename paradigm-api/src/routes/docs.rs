use axum::{routing::get, Router, response::Html};

pub fn router() -> Router {
    Router::new()
        .route("/docs", get(api_docs))
        .route("/docs/openapi.json", get(openapi_spec))
}

async fn api_docs() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Paradigm API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@3.25.0/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@3.25.0/swagger-ui-bundle.js"></script>
    <script>
        SwaggerUIBundle({
            url: '/docs/openapi.json',
            dom_id: '#swagger-ui',
            presets: [
                SwaggerUIBundle.presets.apis,
                SwaggerUIBundle.presets.standalone
            ]
        });
    </script>
</body>
</html>
    "#)
}

async fn openapi_spec() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Paradigm API",
            "version": "1.0.0",
            "description": "Enterprise REST API for Paradigm blockchain network"
        },
        "servers": [
            {
                "url": "/api/v1",
                "description": "API v1"
            }
        ],
        "paths": {
            "/health": {
                "get": {
                    "summary": "Health check",
                    "responses": {
                        "200": {
                            "description": "Service is healthy"
                        }
                    }
                }
            }
        }
    }))
}