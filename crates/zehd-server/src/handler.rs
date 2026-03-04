use std::sync::Arc;
use std::time::Instant;

use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{Method as HttpMethod, StatusCode};
use axum::response::{IntoResponse, Response};
use owo_colors::OwoColorize;
use zehd_rune::value::Value;

use crate::json::value_to_json;
use crate::router::{Method, RouteTable};

/// Axum fallback handler — dispatches requests to the route table, logs the result.
pub async fn handle_request(
    request: Request,
    route_table: Arc<RouteTable>,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = Instant::now();

    let (status, response) = dispatch(request, &route_table).await;

    log_request(&method, &path, status, start.elapsed());

    response
}

/// Pure dispatch — resolves the route, executes the handler, returns status + response.
async fn dispatch(request: Request, route_table: &RouteTable) -> (StatusCode, Response) {
    // Extract request data before consuming the body.
    let method_str = request.method().to_string();
    let url_path = request.uri().path().to_string();
    let query_str = request.uri().query().unwrap_or("").to_string();
    let http_method = request.method().clone();

    // Extract headers as key-value pairs.
    let header_fields: Vec<(String, Value)> = request
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                Value::String(value.to_str().unwrap_or("").to_string()),
            )
        })
        .collect();

    // Read body (consumes the request).
    let body_bytes = to_bytes(request.into_body(), 1024 * 1024) // 1MB limit
        .await
        .unwrap_or_default();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();

    // Look up the route
    let entry = match route_table.routes.get(&url_path) {
        Some(entry) => Arc::clone(entry),
        None => return (StatusCode::NOT_FOUND, not_found(&url_path)),
    };

    // Convert HTTP method
    let method = match Method::from_axum(&http_method) {
        Some(m) => m,
        None => {
            return (
                StatusCode::METHOD_NOT_ALLOWED,
                method_not_allowed(&entry.allowed_methods),
            )
        }
    };

    // Look up handler for this method
    let handler_index = match entry.method_map.get(&method) {
        Some(&idx) => idx,
        None => {
            return (
                StatusCode::METHOD_NOT_ALLOWED,
                method_not_allowed(&entry.allowed_methods),
            )
        }
    };

    // Build self value: { request: { ... }, response: { ... }, params: { } }
    let self_value = Value::Object(vec![
        (
            "request".to_string(),
            Value::Object(vec![
                ("method".to_string(), Value::String(method_str)),
                ("path".to_string(), Value::String(url_path.clone())),
                ("headers".to_string(), Value::Object(header_fields)),
                ("body".to_string(), Value::String(body_str)),
                ("query".to_string(), Value::String(query_str)),
            ]),
        ),
        (
            "response".to_string(),
            Value::Object(vec![
                ("status".to_string(), Value::Int(200)),
            ]),
        ),
        (
            "params".to_string(),
            Value::Object(vec![]),
        ),
    ]);

    // Lock VM and execute handler
    let result = {
        let mut vm = entry.vm.lock().unwrap();
        vm.execute_handler(handler_index, &entry.context, self_value)
    };

    match result {
        Ok(value) => match value_to_json(&value) {
            Some(json) => (StatusCode::OK, json_response(StatusCode::OK, &json)),
            None => (StatusCode::NO_CONTENT, no_content()),
        },
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            internal_error(&err.message),
        ),
    }
}

/// Print a single request log line with colored status.
fn log_request(
    method: &HttpMethod,
    path: &str,
    status: StatusCode,
    duration: std::time::Duration,
) {
    let status_code = status.as_u16();

    let duration_str = {
        let nanos = duration.as_nanos();
        if nanos < 1_000 {
            format!("{nanos}ns")
        } else if nanos < 1_000_000 {
            format!("{:.1}µs", nanos as f64 / 1_000.0)
        } else if nanos < 1_000_000_000 {
            format!("{:.2}ms", nanos as f64 / 1_000_000.0)
        } else {
            format!("{:.2}s", duration.as_secs_f64())
        }
    };

    // Color the status code: green 2xx, yellow 4xx, red 5xx
    let status_str = if status_code < 400 {
        format!("{}", status_code.green())
    } else if status_code < 500 {
        format!("{}", status_code.yellow())
    } else {
        format!("{}", status_code.red())
    };

    println!(
        "  {:<6} {} → {} in {}",
        method.as_str().dimmed(),
        path,
        status_str,
        duration_str.dimmed()
    );
}

// ── Response builders ───────────────────────────────────────────

fn json_response(status: StatusCode, value: &serde_json::Value) -> Response {
    let body = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());
    (
        status,
        [("content-type", "application/json")],
        body,
    )
        .into_response()
}

fn no_content() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

fn not_found(path: &str) -> Response {
    let body = serde_json::json!({
        "error": "Not Found",
        "path": path,
    });
    json_response(StatusCode::NOT_FOUND, &body)
}

fn method_not_allowed(allowed: &[Method]) -> Response {
    let allow_header: String = allowed
        .iter()
        .map(|m| m.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let body = serde_json::json!({
        "error": "Method Not Allowed",
        "allowed": allow_header,
    });
    let json_body = serde_json::to_string(&body).unwrap_or_else(|_| "null".to_string());
    (
        StatusCode::METHOD_NOT_ALLOWED,
        [
            ("content-type", "application/json"),
            ("allow", &allow_header),
        ],
        json_body,
    )
        .into_response()
}

fn internal_error(message: &str) -> Response {
    let body = serde_json::json!({
        "error": "Internal Server Error",
        "message": message,
    });
    json_response(StatusCode::INTERNAL_SERVER_ERROR, &body)
}
