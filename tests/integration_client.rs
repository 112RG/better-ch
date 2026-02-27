use httpmock::Method::GET;
use httpmock::MockServer;

use better_ch::api::CloudHubClient;
use better_ch::auth::oauth::Token;

#[tokio::test]
async fn client_list_applications_success() {
    // Start a mock server
    let server = MockServer::start_async().await;

    // Mock /applications endpoint
    let mock = server.mock(|when, then| {
        when.method(GET).path("/applications");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(r#"{"data":[{"name":"app1","domain":"app1.example","status":"STARTED","workers":{"type":"Micro","quantity":1}}]}"#);
    });

    let base = format!("http://{}", server.address());
    let client = CloudHubClient::new(&base);

    // Provide a valid token (use a Token with is_expired = false)
    let token = Token::test_token_not_expired();
    let mut client = client;
    client.set_token(token);

    let apps = client.list_applications().await.expect("list apps");
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].name, "app1");

    mock.assert();
}

#[tokio::test]
async fn client_requires_authentication() {
    let server = MockServer::start_async().await;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/applications");
        then.status(200).body(r#"{"data":[]}"#);
    });

    let base = format!("http://{}", server.address());
    let client = CloudHubClient::new(&base);

    // No token set: should return AuthError::TokenExpired via Error::Auth
    let res = client.list_applications().await;
    assert!(res.is_err());

    mock.assert_hits(0);
}
