mod auth;
use crate::auth::{get_login, get_register, login, register, Logins, SessionCache};
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;
use hashlink::LinkedHashMap;
use std::collections::HashMap;
use std::sync::Mutex;

#[get("/")]
async fn hello(session: Session, session_cache: web::Data<SessionCache>) -> impl Responder {
    let username: String = match session.get("username") {
        Ok(o) => match o {
            Some(s) => s,
            None => {
                return HttpResponse::Unauthorized().body("Please register or login none");
            }
        },
        Err(_) => {
            return HttpResponse::Unauthorized().body("Please register or login error");
        }
    };
    let session_id: String = session.get("session_id").unwrap().unwrap();
    let mut locked_session_cache = session_cache.sessions.lock().unwrap();
    if !locked_session_cache.contains_key(&username)
        || *locked_session_cache.get(&username).unwrap() != session_id
    {
        return HttpResponse::Unauthorized().body("Please register or login");
    }
    HttpResponse::Ok().body(format!("Hello {} with id {}", username, session_id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    let logins = web::Data::new(Logins {
        logins: Mutex::new(HashMap::<String, String>::new()),
    });
    let sessions = web::Data::new(SessionCache {
        sessions: Mutex::new(LinkedHashMap::<String, String>::with_capacity(1000)),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(logins.clone())
            .app_data(sessions.clone())
            .wrap(
                // Session middleware setup
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .build(),
            )
            .service(hello)
            .service(get_login)
            .service(get_register)
            .service(login)
            .service(register)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
