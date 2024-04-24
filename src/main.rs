use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration;
use zero2prod::startup::run;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let settings = configuration::get_configuration().expect("Failed to read configuration.");
    let connection_pool =
        PgPool::connect_lazy(settings.database.connection_string().expose_secret())
            .expect("Failed to connect to Postgres.");
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
