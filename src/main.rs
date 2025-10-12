use actix_web::{App, HttpServer, web, HttpResponse, Responder};
use actix_files as fs;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use chrono::Local;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Post {
    id: usize,
    title: String,
    author: String,
    content: String,
    date: String,
}

struct AppState {
    posts: Mutex<Vec<Post>>,
}

async fn index(data: web::Data<AppState>) -> impl Responder {
    let posts = data.posts.lock().unwrap();
    let html = include_str!("../static/index.html");
    
    let mut posts_html = String::new();
    for post in posts.iter().rev() {
        posts_html.push_str(&format!(
            r#"<tr onclick="location.href='/post/{}'">
                <td>{}</td>
                <td class="title">{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>"#,
            post.id, post.id, post.title, post.author, post.date
        ));
    }
    
    let html = html.replace("{{posts}}", &posts_html);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn write_page() -> impl Responder {
    let html = include_str!("../static/write.html");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn create_post(
    data: web::Data<AppState>,
    form: web::Form<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let mut posts = data.posts.lock().unwrap();
    let id = posts.len() + 1;
    
    let post = Post {
        id,
        title: form.get("title").unwrap_or(&String::from("제목 없음")).clone(),
        author: form.get("author").unwrap_or(&String::from("익명")).clone(),
        content: form.get("content").unwrap_or(&String::from("")).clone(),
        date: Local::now().format("%Y-%m-%d %H:%M").to_string(),
    };
    
    posts.push(post);
    
    HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish()
}

async fn view_post(data: web::Data<AppState>, path: web::Path<usize>) -> impl Responder {
    let posts = data.posts.lock().unwrap();
    let post_id = path.into_inner();
    
    if let Some(post) = posts.iter().find(|p| p.id == post_id) {
        let html = include_str!("../static/view.html");
        let html = html
            .replace("{{title}}", &post.title)
            .replace("{{author}}", &post.author)
            .replace("{{date}}", &post.date)
            .replace("{{content}}", &post.content.replace("\n", "<br>"));
        
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)
    } else {
        HttpResponse::NotFound().body("게시글을 찾을 수 없습니다.")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("서버가 http://localhost:8081 에서 실행 중입니다");
    
    let app_state = web::Data::new(AppState {
        posts: Mutex::new(Vec::new()),
    });
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/write", web::get().to(write_page))
            .route("/write", web::post().to(create_post))
            .route("/post/{id}", web::get().to(view_post))
            .service(fs::Files::new("/static", "./static"))
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
