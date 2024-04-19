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
    let connection_pool = PgPool::connect(settings.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", settings.application_port))?;
    run(listener, connection_pool)?.await
}
