use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use ztp::configuration::get_configuration;
use ztp::startup::run;
use ztp::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("ztp".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    //env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to postgres");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    run(listener, connection)?.await
}
