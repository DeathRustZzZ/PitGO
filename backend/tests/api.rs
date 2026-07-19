//! Black-box HTTP contract tests for the backend router.
//!
//! Every test creates a fresh in-memory application and interacts with it only
//! through HTTP requests and responses. No TCP listener is required: axum's
//! Router is a tower Service and `oneshot` drives it in-process.
//!
//! Чёрноящичные HTTP-тесты контракта backend-роутера.
//!
//! Каждый тест создаёт новое приложение с хранилищем в памяти и взаимодействует
//! с ним только через HTTP-запросы и ответы. TCP-слушатель не нужен: Router из
//! axum — tower Service, а `oneshot` запускает его внутри процесса.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
    response::Response,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

fn test_app() -> Router {
    backend::app(backend::AppState::in_memory())
}

fn request(method: &str, uri: &str, body: Body) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .expect("test request should be valid")
}

async fn send(app: Router, request: Request<Body>) -> Response {
    app.oneshot(request)
        .await
        .expect("router should produce a response")
}

async fn json_body(response: Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body should be readable")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("response body should be valid JSON")
}

async fn text_body(response: Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body should be readable")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("response body should be UTF-8")
}

#[tokio::test]
async fn health_returns_ok() {
    let response = send(test_app(), request("GET", "/health", Body::empty())).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(text_body(response).await, "OK");
}

#[tokio::test]
async fn customer_can_be_created_and_fetched() {
    let app = test_app();
    let customer_id = "11111111-1111-4111-8111-111111111111";

    let create = send(
        app.clone(),
        request(
            "POST",
            "/customers",
            Body::from(json!({ "customer_id": customer_id }).to_string()),
        ),
    )
    .await;
    assert_eq!(create.status(), StatusCode::CREATED);

    let get = send(
        app,
        request("GET", &format!("/customers/{customer_id}"), Body::empty()),
    )
    .await;
    assert_eq!(get.status(), StatusCode::OK);

    let body = json_body(get).await;
    assert_eq!(body["customer_id"], customer_id);
    assert_eq!(body["status"], "Draft");
}

#[tokio::test]
async fn missing_customer_returns_semantic_not_found_code() {
    let response = send(
        test_app(),
        request(
            "GET",
            "/customers/11111111-1111-4111-8111-111111111111",
            Body::empty(),
        ),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_body(response).await["error"], "customer_not_found");
}

#[tokio::test]
async fn duplicate_customer_create_returns_already_exists() {
    let app = test_app();
    let customer_id = "11111111-1111-4111-8111-111111111111";
    let body = json!({ "customer_id": customer_id }).to_string();

    let first = send(
        app.clone(),
        request("POST", "/customers", Body::from(body.clone())),
    )
    .await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let duplicate = send(app, request("POST", "/customers", Body::from(body))).await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);
    assert_eq!(json_body(duplicate).await["error"], "already_exists");
}

#[tokio::test]
async fn second_ownership_for_vehicle_returns_active_ownership_conflict() {
    let app = test_app();
    let vehicle_id = "22222222-2222-4222-8222-222222222222";

    let vehicle = send(
        app.clone(),
        request(
            "POST",
            "/vehicles",
            Body::from(json!({ "vehicle_id": vehicle_id }).to_string()),
        ),
    )
    .await;
    assert_eq!(vehicle.status(), StatusCode::CREATED);

    let first = send(
        app.clone(),
        request(
            "POST",
            &format!("/vehicles/{vehicle_id}/ownerships"),
            Body::from(
                json!({
                    "ownership_id": "33333333-3333-4333-8333-333333333333",
                    "owner_customer_id": "44444444-4444-4444-8444-444444444444",
                    "ownership_type": "private"
                })
                .to_string(),
            ),
        ),
    )
    .await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let second = send(
        app,
        request(
            "POST",
            &format!("/vehicles/{vehicle_id}/ownerships"),
            Body::from(
                json!({
                    "ownership_id": "55555555-5555-4555-8555-555555555555",
                    "owner_customer_id": "66666666-6666-4666-8666-666666666666",
                    "ownership_type": "company"
                })
                .to_string(),
            ),
        ),
    )
    .await;

    assert_eq!(second.status(), StatusCode::CONFLICT);
    assert_eq!(
        json_body(second).await["error"],
        "active_ownership_already_exists"
    );
}

#[tokio::test]
async fn ownership_is_scoped_to_its_vehicle_url() {
    let app = test_app();
    let vehicle_id = "22222222-2222-4222-8222-222222222222";
    let ownership_id = "33333333-3333-4333-8333-333333333333";

    let _vehicle = send(
        app.clone(),
        request(
            "POST",
            "/vehicles",
            Body::from(json!({ "vehicle_id": vehicle_id }).to_string()),
        ),
    )
    .await;
    let ownership = send(
        app.clone(),
        request(
            "POST",
            &format!("/vehicles/{vehicle_id}/ownerships"),
            Body::from(
                json!({
                    "ownership_id": ownership_id,
                    "owner_customer_id": "44444444-4444-4444-8444-444444444444",
                    "ownership_type": "private"
                })
                .to_string(),
            ),
        ),
    )
    .await;
    assert_eq!(ownership.status(), StatusCode::CREATED);

    let found = send(
        app.clone(),
        request(
            "GET",
            &format!("/vehicles/{vehicle_id}/ownerships/{ownership_id}"),
            Body::empty(),
        ),
    )
    .await;
    assert_eq!(found.status(), StatusCode::OK);
    let found_body = json_body(found).await;
    assert_eq!(found_body["ownership_id"], ownership_id);
    assert_eq!(found_body["status"], "pending_verification");
    assert_eq!(found_body["ended_at"], Value::Null);

    let wrong_vehicle = send(
        app,
        request(
            "GET",
            &format!("/vehicles/77777777-7777-4777-8777-777777777777/ownerships/{ownership_id}"),
            Body::empty(),
        ),
    )
    .await;
    assert_eq!(wrong_vehicle.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        json_body(wrong_vehicle).await["error"],
        "ownership_not_found"
    );
}

#[tokio::test]
async fn malformed_customer_json_uses_axums_current_default_error_response() {
    // Characterization test: extractor failures are not yet mapped through
    // ApiError. A uniform `{ error, message }` body is a separate Phase 2 task.
    //
    // Характеризационный тест: ошибки extractor пока не проходят через ApiError.
    // Единое тело `{ error, message }` — отдельная задача Фазы 2.
    let response = send(
        test_app(),
        request("POST", "/customers", Body::from("{not valid JSON")),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .expect("extractor response should have a content type")
            .to_str()
            .expect("content type should be valid ASCII")
            .starts_with("text/plain")
    );
    assert!(!text_body(response).await.is_empty());
}
