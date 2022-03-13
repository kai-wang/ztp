use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use ztp::configuration::{get_configuration, DatabaseSettings};
use ztp::startup::{get_connection_pool, Application};
use ztp::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "debug".to_string();
    let subscriber_name = "test".to_string();

    // If TEST_LOG is set, use std::io::stdout for output
    // Otherwise, send all logs into std::io::sink
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    // Make sure it is only initialized once
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configurations.");
        // Randomise the database name to isolate the testing
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("failed to build application.");

    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // let mut connection =
    //     PgConnection::connect(&config.connection_string_without_db().expose_secret())
    //         .await
    //         .expect("failed to connect to postgres");

    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("failed to connect to postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("failed to create database");

    // Migrate database
    // let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
    //     .await
    //     .expect("failed to connect to Postgres");
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("failed to migrate the database");

    connection_pool
}