use actix_files::NamedFile;

pub async fn homepage() -> actix_web::Result<NamedFile> {
    let path = "static/homepage.html";
    Ok(NamedFile::open(path)?)
}
