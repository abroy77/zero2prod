use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<FormData>) -> HttpResponse {
    print!("Subscribing...");
    print!("name: {}", form.name);
    print!("email: {}", form.email);
    HttpResponse::Ok().finish()
}
