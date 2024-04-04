use std::net::TcpListener;
use zero2prod::configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let settings = configuration::get_configuration().expect("Failed to read configuration.");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", settings.application_port))?;
    run(listener)?.await
}
