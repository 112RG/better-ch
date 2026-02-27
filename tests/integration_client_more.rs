use httpmock::Method::{GET, POST};
use httpmock::MockServer;

use better_ch::api::CloudHubClient;
use better_ch::auth::oauth::Token;

#[tokio::test]
async fn client_start_stop_restart() {
    let server = MockServer::start_async().await;

    // Start
    let mock_start = server.mock(|when, then| {
        when.method(POST).path("/applications/test-app/start");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(r#"{"status":"STARTED","message":null}"#);
    });

    // Stop
    let mock_stop = server.mock(|when, then| {
        when.method(POST).path("/applications/test-app/stop");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(r#"{"status":"STOPPED","message":null}"#);
    });

    // Restart
    let mock_restart = server.mock(|when, then| {
        when.method(POST).path("/applications/test-app/restart");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(r#"{"status":"RESTARTED","message":null}"#);
    });

    let base = format!("http://{}", server.address());
    let mut client = CloudHubClient::new(&base);
    client.set_token(Token::test_token_not_expired());

    let start = client.start_application("test-app").await.expect("start");
    assert_eq!(start.status, "STARTED");
    mock_start.assert();

    let stop = client.stop_application("test-app").await.expect("stop");
    assert_eq!(stop.status, "STOPPED");
    mock_stop.assert();

    let restart = client
        .restart_application("test-app")
        .await
        .expect("restart");
    assert_eq!(restart.status, "RESTARTED");
    mock_restart.assert();
}

#[tokio::test]
async fn client_get_application_logs_with_limit() {
    let server = MockServer::start_async().await;

    let timestamp = chrono::Utc::now().to_rfc3339();

    let mock = server.mock(|when, then| {
        when.method(GET).path("/applications/test-app/logs").query_param("limit", "10");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(format!(r#"{{"data":[{{"timestamp":"{}","level":"INFO","message":"started","applicationName":"test-app"}}]}}"#, timestamp));
    });

    let base = format!("http://{}", server.address());
    let mut client = CloudHubClient::new(&base);
    client.set_token(Token::test_token_not_expired());

    let logs = client
        .get_application_logs("test-app", Some(10))
        .await
        .expect("logs");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].message, "started");

    mock.assert();
}

#[tokio::test]
async fn client_list_deployments() {
    let server = MockServer::start_async().await;

    let mock = server.mock(|when, then| {
        when.method(GET).path("/applications/test-app/deployments");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(r#"{"data":[{"id":"d1","applicationName":"test-app","status":"DEPLOYED"}]}"#);
    });

    let base = format!("http://{}", server.address());
    let mut client = CloudHubClient::new(&base);
    client.set_token(Token::test_token_not_expired());

    let deps = client
        .list_deployments("test-app")
        .await
        .expect("deployments");
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].id, "d1");

    mock.assert();
}
