use std::net::TcpListener;
use sqlx::{PgConnection, Connection, postgres::PgColumn};
use ztp::configuration::get_configuration;

#[tokio::test]
async fn health_check_works() {
  let address = spawn_app();

  let client = reqwest::Client::new();
  let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

  assert!(response.status().is_success());
  assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
  let app_address = spawn_app();
  let configuration = get_configuration().expect("fail to read the connection");
  let connection_string = configuration.database.connection_string();
  let connection = PgConnection::connect(&connection_string)
        .await
        .expect("failed to connect to the Postgres");
  let client = reqwest::Client::new();

  let body = "name=eric%20wang&email=eric_wang%40gmail.com";
  let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

  assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
  let app_address = spawn_app();
  let client = reqwest::Client::new();

  let test_cases = vec![
    ("name=eric%20wang", " missing name"),
    ("email=eric_wang%40gmail.com", "missing email"),
    ("", "missing both name and email")
  ];

  for(invalid_body, error_message) in test_cases {
    let response = client
          .post(&format!("{}/subscriptions", &app_address))
          .header("Content-Type", "application/x-www-form-urlencoded")
          .body(invalid_body)
          .send()
          .await
          .expect("Failed to execute the request.");

    assert_eq!(
      400, 
      response.status().as_u16(), 
      "bad request with payload {}.", error_message
    );

    println!("{:?}", response);
  }
}


fn spawn_app() -> String {
  // Bind a random port;
  let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
  let port = listener.local_addr().unwrap().port();
  println!("port is {:?}", &port);

  let server = ztp::startup::run(listener).expect("Failed to bind address");
  let _ = tokio::spawn(server);

  format!("http://127.0.0.1:{}", port)
}