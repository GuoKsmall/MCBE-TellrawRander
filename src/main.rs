use actix_files::Files;
use actix_web::{
    middleware, web, App, Error, HttpResponse, HttpServer,
    HttpRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct RenderForm {
    mode: String,
    content: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// Index served as static `web/index.html`

// Handle render request
async fn render_post(req: HttpRequest, body: web::Bytes) -> Result<HttpResponse, Error> {
    // Log content-type for debugging
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    log::debug!("/api/render called with Content-Type: {}", content_type);

    // Parse JSON body into RenderForm
    let form: RenderForm = match serde_json::from_slice(&body) {
        Ok(f) => f,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid JSON: {}", e),
            }));
        }
    };

    // Validate input
    if form.content.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Content cannot be empty".to_string(),
        }));
    }

    let mode = form.mode.as_str();
    if mode != "text" && mode != "tellraw" {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Mode must be 'text' or 'tellraw'".to_string(),
        }));
    }

    let content = form.content.as_str();

    // Prepare substitutions (empty for now)
    let selectors_sub: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let scores_sub: std::collections::HashMap<String, std::collections::HashMap<String, i32>> = std::collections::HashMap::new();

    let text_to_render = if mode == "tellraw" {
        match serde_json::from_str::<serde_json::Value>(content) {
            Ok(mut val) => {
                let translated = mcbe_text_impact::translate_tellraw(&mut val, &selectors_sub, &scores_sub);
                if translated.is_string() {
                    translated.as_str().unwrap().to_string()
                } else if let Some(arr) = translated.get("rawtext").and_then(|v| v.as_array()) {
                    let mut out = String::new();
                    for el in arr.iter() {
                        if let Some(s) = el.get("text").and_then(|v| v.as_str()) {
                            out.push_str(s);
                        }
                    }
                    out
                } else {
                    serde_json::to_string(&translated).unwrap_or_default()
                }
            }
            Err(_) => {
                // If JSON parse fails, return error
                return Ok(HttpResponse::BadRequest().body("Invalid JSON"));
            }
        }
    } else {
        content.to_string()
    };

    // Call existing render function; font directory is `font_png`
    let img = mcbe_text_impact::render("font_png", &text_to_render, None);

    // Convert RgbaImage to bytes
    let mut buf: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageOutputFormat::Png).unwrap();

    Ok(HttpResponse::Ok().content_type("image/png").body(buf))
}

#[actix_web::main]
// Main server function
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // Check required directories
    for dir in &["font_png", "web"] {
        let path = std::path::Path::new(dir);
        if !path.exists() {
            eprintln!("Error: Required directory '{}' not found", dir);
            std::process::exit(1);
        }
    }

    let server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("Access-Control-Allow-Origin", "*"))
                    .add(("Access-Control-Allow-Methods", "POST, GET, OPTIONS"))
                    .add(("Access-Control-Allow-Headers", "Content-Type"))
                    // Disable cache
                    .add(("Cache-Control", "no-store, no-cache, must-revalidate, proxy-revalidate, max-age=0"))
                    .add(("Pragma", "no-cache"))
                    .add(("Expires", "0"))
            )
            .wrap(middleware::Compress::default())
            .service(
                web::scope("/api")
                    .route("/render", web::post().to(render_post))
            )
            .service(
                Files::new("/", "web")
                    .show_files_listing()
                    .index_file("index.html")
                    .prefer_utf8(true)
            )
    })
    .workers(2);

    println!("Starting server at http://127.0.0.1:8080");
    server.bind("127.0.0.1:8080")?.run().await
}