use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;
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

struct Logins {
    logins: Mutex<HashMap<String, String>>,
}

struct SessionCache {
    sessions: Mutex<LinkedHashMap<String, String>>,
}

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
    if *locked_session_cache.get(&username).unwrap() != session_id {
        return HttpResponse::Unauthorized().body("Please register or login");
    }
    HttpResponse::Ok().body(format!("Hello {} with id {}", username, session_id))
}

#[post("/login")]
async fn login_verify(
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
                panic!("wrong passorw")
            }
        }
        None => {
            locked_login.insert(login_data.username.clone(), login_data.password.clone());
            session.insert("username", &login_data.username);
            session.insert("session_id", &session_id);
            locked_session_cache.insert(login_data.username.clone(), session_id);
            HttpResponse::Ok().body(format!(
                "register{},{}",
                login_data.username.clone(),
                login_data.password.clone()
            ))
        }
    }
}

#[get("/login")]
async fn login() -> impl Responder {
    HttpResponse::Ok().body(r#"
<!DOCTYPE html>
<html lang="en">
<head>
  <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js"></script>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Login Form</title>
  <!-- Bootstrap CSS -->
  <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css" rel="stylesheet">
</head>
<body>
  <div class="container d-flex justify-content-center align-items-center vh-100">
    <div class="card p-4 shadow" style="max-width: 400px; width: 100%;">
      <h4 class="text-center mb-4">Login</h4>
      <form action="/login" method="POST">
        <div class="mb-3">
          <label for="username" class="form-label">Email address</label>
          <input type="text" class="form-control" id="username" name="username" placeholder="Enter your username" required>
        </div>
        <div class="mb-3">
          <label for="password" class="form-label">Password</label>
          <input type="password" class="form-control" id="password" name="password" placeholder="Enter your password" required>
        </div>
        <button type="submit" class="btn btn-primary w-100">Login</button>
      </form>
    </div>
  </div>
  <!-- Bootstrap JS (optional) -->
</body>
</html>
"#)
}
#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
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
            .service(echo)
            .service(login)
            .service(login_verify)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
