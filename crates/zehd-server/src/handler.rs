use std::sync::Arc;
use std::time::Instant;

use axum::extract::Request;
use axum::http::{Method as HttpMethod, StatusCode};
use axum::response::{IntoResponse, Response};
use owo_colors::OwoColorize;

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

    let (status, response) = dispatch(request, &route_table);

    log_request(&method, &path, status, start.elapsed());

    response
}

/// Pure dispatch — resolves the route, executes the handler, returns status + response.
fn dispatch(request: Request, route_table: &RouteTable) -> (StatusCode, Response) {
    let url_path = request.uri().path().to_string();
    let http_method = request.method().clone();

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

    // Lock VM and execute handler
    let result = {
        let mut vm = entry.vm.lock().unwrap();
        vm.execute_handler(handler_index, &entry.context)
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
    let ms = duration.as_millis();

    // Color the status code: green 2xx, yellow 4xx, red 5xx
    let status_str = if status_code < 400 {
        format!("{}", status_code.green())
    } else if status_code < 500 {
        format!("{}", status_code.yellow())
    } else {
        format!("{}", status_code.red())
    };

    println!(
        "  {:<6} {} → {} in {}ms",
        method.as_str().dimmed(),
        path,
        status_str,
        ms
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
