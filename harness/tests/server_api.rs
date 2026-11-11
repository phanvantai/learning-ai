//! The local web app's API, end to end over HTTP, with a fake model behind it.

use std::sync::{Arc, Mutex};

use lai_harness::Host;
use serde_json::{json, Value};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[derive(Default)]
struct TestHost {
    autostart: Mutex<bool>,
}

impl Host for TestHost {
    fn autostart(&self) -> Option<bool> {
        Some(*self.autostart.lock().unwrap())
    }
    fn set_autostart(&self, enabled: bool) -> anyhow::Result<()> {
        *self.autostart.lock().unwrap() = enabled;
        Ok(())
    }
}

struct Client {
    http: reqwest::Client,
    base: String,
    token: String,
}

impl Client {
    async fn call(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> (u16, Value) {
        let mut req = self.http.request(method, format!("{}/api{path}", self.base)).header("x-lai-token", &self.token);
        if let Some(body) = body {
            req = req.json(&body);
        }
        let resp = req.send().await.unwrap();
        (resp.status().as_u16(), resp.json().await.unwrap_or(Value::Null))
    }
    async fn ok(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> Value {
        let (status, value) = self.call(method, path, body).await;
        assert_eq!(status, 200, "{path}: {value}");
        value
    }
}

#[tokio::test]
async fn ui_api_round_trip() {
    use reqwest::Method;

    // A model that first asks to send an email, then wraps up.
    let model = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"stop_reason": "tool_use", "content": [
            {"type": "tool_use", "id": "t1", "name": "gmail_send",
             "input": {"to": "lan@example.com", "subject": "Chào", "body": "Nội dung"}},
        ]})))
        .up_to_n_times(1)
        .mount(&model)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"stop_reason": "end_turn", "content": [{"type": "text", "text": "Thư đang chờ duyệt."}]}),
        ))
        .mount(&model)
        .await;

    let home = tempfile::tempdir().unwrap();
    std::env::set_var("LAI_HOME", home.path());
    std::env::set_var("LAI_SECRETS_FILE", "1");
    std::env::remove_var("ANTHROPIC_API_KEY");
    std::fs::write(
        home.path().join("config.toml"),
        format!(
            "[providers.api]\nkind = \"anthropic\"\nbase_url = \"{}\"\nfallbacks = false\n\n\
             [agents.tro-ly]\nprovider = \"api\"\ntools = [\"gmail_search\", \"gmail_send\"]\n",
            model.uri()
        ),
    )
    .unwrap();

    let url = lai_harness::server::start(Arc::new(TestHost::default())).await.unwrap();
    let (base, token) = url.split_once("/#token=").unwrap();
    let client = Client { http: reqwest::Client::new(), base: base.into(), token: token.into() };

    // The UI itself is public; the API is not.
    let page = client.http.get(base).send().await.unwrap();
    assert!(page.text().await.unwrap().contains("Trợ lý AI"));
    let anonymous = client.http.get(format!("{base}/api/state")).send().await.unwrap();
    assert_eq!(anonymous.status(), 403);
    let cross_site = client
        .http
        .get(format!("{base}/api/state"))
        .header("x-lai-token", token)
        .header("origin", "https://evil.example")
        .send()
        .await
        .unwrap();
    assert_eq!(cross_site.status(), 403);

    // Fresh install: brain has no key yet, Gmail not connected.
    let state = client.ok(Method::GET, "/state", None).await;
    assert_eq!(state["default_agent"], "tro-ly");
    assert_eq!(state["providers"][0]["ready"], false);
    assert_eq!(state["gmail"]["connected"], false);
    assert_eq!(state["autostart"], false);

    client.ok(Method::POST, "/providers/api/key", Some(json!({"key": "sk-test"}))).await;
    let state = client.ok(Method::GET, "/state", None).await;
    assert_eq!(state["providers"][0]["ready"], true);

    // Chat: the model's send is parked, never executed.
    let reply = client.ok(Method::POST, "/chat", Some(json!({"message": "gửi thư cho Lan"}))).await;
    assert_eq!(reply["answer"], "Thư đang chờ duyệt.");
    let sent: Value = model.received_requests().await.unwrap()[0].body_json().unwrap();
    assert!(sent["messages"][0]["content"].as_str().unwrap().contains("gửi thư cho Lan"));
    let state = client.ok(Method::GET, "/state", None).await;
    let pending = state["pending"].as_array().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["input"]["to"], "lan@example.com");
    let id = pending[0]["id"].as_str().unwrap();

    // Approving without Gmail fails and leaves it pending; rejecting clears it.
    let (status, error) = client.call(Method::POST, &format!("/approvals/{id}/approve"), None).await;
    assert_eq!(status, 400);
    assert!(error["error"].as_str().unwrap().contains("vẫn nằm trong hàng chờ"), "{error}");
    client.ok(Method::POST, &format!("/approvals/{id}/reject"), None).await;
    assert_eq!(client.ok(Method::GET, "/state", None).await["pending"], json!([]));

    // Jobs: create, validate, run, read the report, delete.
    let job = json!({"schedule": "30 8 * * 1-5", "prompt": "Tóm tắt thư"});
    client.ok(Method::PUT, "/jobs/tom-tat", Some(job)).await;
    let (status, error) =
        client.call(Method::PUT, "/jobs/hong", Some(json!({"schedule": "sai lịch", "prompt": "x"}))).await;
    assert_eq!(status, 400, "{error}");
    let state = client.ok(Method::GET, "/state", None).await;
    assert_eq!(state["jobs"].as_array().unwrap().len(), 1);
    assert!(state["jobs"][0]["next_run"].is_string());

    let run = client.ok(Method::POST, "/jobs/tom-tat/run", None).await;
    assert_eq!(run["report"], "Thư đang chờ duyệt.");
    let file = run["file"].as_str().unwrap();
    let report = client.ok(Method::GET, &format!("/reports/tom-tat/{file}"), None).await;
    assert_eq!(report["text"], "Thư đang chờ duyệt.");
    let (status, _) = client.call(Method::GET, "/reports/tom-tat/..%2F..%2Fconfig.toml", None).await;
    assert_eq!(status, 400);
    client.ok(Method::DELETE, "/jobs/tom-tat", None).await;

    // Brains: add one, switch the agent to it, and the one in use cannot be removed.
    let brain = json!({"kind": "openai", "model": "m", "api_key": "k"});
    client.ok(Method::PUT, "/providers/khac", Some(brain)).await;
    let agent = json!({"provider": "khac", "instructions": "Xưng em."});
    client.ok(Method::PUT, "/agents/tro-ly", Some(agent)).await;
    let (status, error) = client.call(Method::DELETE, "/providers/khac", None).await;
    assert_eq!(status, 400, "{error}");
    client.ok(Method::DELETE, "/providers/api", None).await;
    let state = client.ok(Method::GET, "/state", None).await;
    assert_eq!(state["agents"][0]["provider"], "khac");
    assert_eq!(state["agents"][0]["instructions"], "Xưng em.");
    assert_eq!(state["providers"].as_array().unwrap().len(), 1);

    client.ok(Method::POST, "/autostart", Some(json!({"enabled": true}))).await;
    assert_eq!(client.ok(Method::GET, "/state", None).await["autostart"], true);
}
