//! The Gmail sign-in flow end to end, with a stand-in for Google.

use lai_harness::gmail::{self, Endpoints, GmailClient};
use lai_harness::secrets;
use serde_json::json;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn signs_in_stores_the_token_and_reads_the_mailbox() {
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("LAI_HOME", home.path());
    std::env::set_var("LAI_SECRETS_FILE", "1");

    let google = MockServer::start().await;
    // Code exchange must carry the code and the PKCE verifier.
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .and(body_string_contains("code=the-code"))
        .and(body_string_contains("code_verifier="))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"refresh_token": "refresh-1", "access_token": "a0"})))
        .mount(&google)
        .await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .and(body_string_contains("refresh_token=refresh-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token": "access-1", "expires_in": 3600})))
        .mount(&google)
        .await;
    Mock::given(method("GET"))
        .and(path("/gmail/v1/users/me/profile"))
        .and(header("authorization", "Bearer access-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"emailAddress": "an@example.com"})))
        .mount(&google)
        .await;
    let endpoints = Endpoints {
        auth: format!("{}/auth", google.uri()),
        token: format!("{}/token", google.uri()),
        api: google.uri(),
    };

    // Play the user's browser: follow the consent URL's redirect back to the app.
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let browser = tokio::spawn(async move {
        let consent = url::Url::parse(&rx.await.unwrap()).unwrap();
        let param = |k: &str| consent.query_pairs().find(|(key, _)| key == k).unwrap().1.to_string();
        assert_eq!(param("client_id"), "client-1");
        assert_eq!(param("code_challenge_method"), "S256");
        assert!(param("scope").contains("gmail.readonly"));
        let redirect = param("redirect_uri");
        let http = reqwest::Client::new();
        // A stray request and a forged state must not complete the sign-in.
        http.get(format!("{redirect}/favicon.ico")).send().await.unwrap();
        http.get(format!("{redirect}/?code=evil&state=wrong")).send().await.unwrap();
        let page = http.get(format!("{redirect}/?code=the-code&state={}", param("state"))).send().await.unwrap();
        assert!(page.text().await.unwrap().contains("Đăng nhập xong"));
    });

    gmail::login_with("client-1", "secret-1", &endpoints, |url| tx.send(url.to_string()).unwrap()).await.unwrap();
    browser.await.unwrap();

    assert_eq!(secrets::get(secrets::GMAIL_REFRESH_TOKEN).unwrap().as_deref(), Some("refresh-1"));
    // The stored login is enough to get a fresh access token and call the API.
    let address = GmailClient::from_secrets_at(&endpoints).profile_email().await.unwrap();
    assert_eq!(address, "an@example.com");
}
