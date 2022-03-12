use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
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
    // let connection =
    //     PgPool::connect_lazy(&configuration.database.connection_string().expose_secret())
    //         .expect("Failed to connect to postgres");

    let connection = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;

    run(listener, connection)?.await
}
