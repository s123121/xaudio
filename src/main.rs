mod youtube;
mod billboard;
use serde::Deserialize;
use serde_json::json;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use actix_files::Files;
use futures::StreamExt;

#[derive(Deserialize)]
struct SearchQuery {
    query: String
}

#[get("/api/search")]
async fn search(param: web::Query<SearchQuery>) -> impl Responder {
    let result = youtube::search_song(&param.query).await.unwrap_or(vec![]);
    web::Json(result)
}

#[derive(Deserialize)]
struct PlayQuery {
    id: String
}

#[derive(Deserialize)]
struct UrlQuery {
    url: String
}

#[get("/api/stream")]
async fn stream(param: web::Query<PlayQuery>) -> impl Responder {
    if let Ok(url) = youtube::get_song_url(&param.id) {
        let response = youtube::get_song_stream(&url).await;
        match response {
            Ok(body) => {
                let stream = body.bytes_stream().map(|item| item.map_err(|_| HttpResponse::Gone()));
                return HttpResponse::Ok()
                    .header("Content-Type", "audio/webm")
                    .streaming(stream);
            },
            Err(_) => {
                return HttpResponse::NotFound().json(json!({ "success": false }));
            }
        }
    }
    HttpResponse::NotFound().json(json!({ "success": false }))
}

#[post("/api/import")]
async fn import_from_url(param: web::Json<UrlQuery>) -> impl Responder {
    let result = youtube::get_songs_in_playlist(&param.url);
    match result {
        Ok(playlist) => web::Json(json!(playlist)),
        Err(_) => web::Json(json!({ "success": false }))
    }
}

#[get("/api/billboard")]
async fn get_billboard() -> impl Responder {
    let result = billboard::get_top_songs().await;
    match result {
        Ok(songs) => web::Json(json!({ "songs": songs })),
        Err(_) => web::Json(json!({ "success": false }))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or("3123".to_owned()).parse::<u16>().unwrap_or(3123);

    HttpServer::new(|| {
        App::new()
            .service(search)
            .service(stream)
            .service(get_billboard)
            .service(import_from_url)
            .service(Files::new("/", "./www").index_file("index.html"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
