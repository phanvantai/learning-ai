//! Gmail client against a fake Gmail API.

use base64::engine::general_purpose::{STANDARD, URL_SAFE};
use base64::Engine;
use lai_harness::gmail::GmailClient;
use serde_json::{json, Value};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn search_returns_headers_and_snippets() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/gmail/v1/users/me/messages"))
        .and(query_param("q", "is:unread"))
        .and(header("authorization", "Bearer tok"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"messages": [{"id": "abc123"}]})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/gmail/v1/users/me/messages/abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "abc123",
            "labelIds": ["INBOX", "UNREAD"],
            "snippet": "Gửi anh báo giá",
            "payload": {"headers": [
                {"name": "From", "value": "Lan <lan@example.com>"},
                {"name": "subject", "value": "Báo giá tháng 11"},
            ]},
        })))
        .mount(&server)
        .await;

    let found = GmailClient::with_static_token(&server.uri(), "tok").search("is:unread", 5).await.unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["from"], "Lan <lan@example.com>");
    assert_eq!(found[0]["subject"], "Báo giá tháng 11");
    assert_eq!(found[0]["unread"], true);
}

#[tokio::test]
async fn send_posts_a_decodable_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/gmail/v1/users/me/messages/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "sent1"})))
        .mount(&server)
        .await;

    let id = GmailClient::with_static_token(&server.uri(), "tok")
        .send("lan@example.com", "Chào", "Nội dung thư")
        .await
        .unwrap();
    assert_eq!(id, "sent1");

    let body: Value = server.received_requests().await.unwrap()[0].body_json().unwrap();
    let mime = String::from_utf8(URL_SAFE.decode(body["raw"].as_str().unwrap()).unwrap()).unwrap();
    let (headers, text) = mime.split_once("\r\n\r\n").unwrap();
    assert!(headers.starts_with("To: lan@example.com\r\n"));
    assert_eq!(STANDARD.decode(text.trim()).unwrap(), "Nội dung thư".as_bytes());
}

#[tokio::test]
async fn api_errors_surface_the_message() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({"error": {"message": "Insufficient Permission"}})))
        .mount(&server)
        .await;
    let err = GmailClient::with_static_token(&server.uri(), "tok").read("abc").await.unwrap_err();
    assert!(err.to_string().contains("Insufficient Permission"));
}
