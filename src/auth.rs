use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use hashlink::LinkedHashMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct RegistrationData {
    username: String,
    password: String,
    key: String,
}

pub struct Logins {
    pub logins: Mutex<HashMap<String, String>>,
}

pub struct SessionCache {
    pub sessions: Mutex<LinkedHashMap<String, String>>,
}

#[post("/login")]
async fn login(
    login_data: web::Form<LoginData>,
    data: web::Data<Logins>,
    session_cache: web::Data<SessionCache>,
    session: Session,
) -> impl Responder {
    let mut locked_login = data.logins.lock().unwrap();
    let mut locked_session_cache = session_cache.sessions.lock().unwrap();
    let session_id = Uuid::new_v4().to_string();
    if locked_session_cache.len() + 10 == locked_session_cache.capacity() {
        locked_session_cache.clear();
    }
    match locked_login.get(&login_data.username) {
        Some(l) => {
            if *l == login_data.password {
                session.insert("username", &login_data.username);
                session.insert("session_id", &session_id);
                locked_session_cache.insert(login_data.username.clone(), session_id);
                HttpResponse::Ok().body(format!(
                    "login{},{}",
                    login_data.username, login_data.password
                ))
            } else {
                return HttpResponse::Unauthorized()
                    .body(include_str!("includes/login_failed.html"));
            }
        }
        None => {
            return HttpResponse::Unauthorized().body(include_str!("includes/login_failed.html"));
        }
    }
}

#[post("/register")]
async fn register(
    login_data: web::Form<RegistrationData>,
    data: web::Data<Logins>,
    session_cache: web::Data<SessionCache>,
    session: Session,
) -> impl Responder {
    let mut locked_login = data.logins.lock().unwrap();
    let mut locked_session_cache = session_cache.sessions.lock().unwrap();
    let session_id = Uuid::new_v4().to_string();
    if locked_session_cache.len() + 10 == locked_session_cache.capacity() {
        locked_session_cache.clear();
    }
    if locked_login.contains_key(&login_data.username) {
        return HttpResponse::Ok().body("username already taken");
    }
    // TODO load me
    if &login_data.key != "key" {
        return HttpResponse::Unauthorized().body("Wrong registration key");
    }
    session.insert("username", &login_data.username);
    session.insert("session_id", &session_id);
    locked_session_cache.insert(login_data.username.clone(), session_id);
    locked_login.insert(login_data.username.clone(), login_data.password.clone());
    return HttpResponse::Ok().body("registraion compleate");
}

#[get("/login")]
async fn get_login() -> impl Responder {
    HttpResponse::Ok().body(include_str!("includes/login.html"))
}

#[get("/register")]
async fn get_register() -> impl Responder {
    HttpResponse::Ok().body(include_str!("includes/register.html"))
}
