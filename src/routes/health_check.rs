use actix_files::NamedFile;

pub async fn health_check() -> actix_web::Result<NamedFile> {
    let path = "static/health_check.html";
    Ok(NamedFile::open(path)?)
}
