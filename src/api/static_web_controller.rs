use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::HttpRequest;

pub async fn get_webrtc_client(_: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

pub async fn get_webrtc_js(_: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./static/webrtc.js".parse().unwrap();
    Ok(NamedFile::open(path)?)
}
