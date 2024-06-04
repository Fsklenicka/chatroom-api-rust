use actix_web::{web, App, HttpServer, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Email {
    username: String,
    email: String,
}

struct AppState {
    users: Arc<Mutex<HashMap<String, String>>>,
    emails: Arc<Mutex<HashMap<String, String>>>,
}

fn load_json<T: for<'de> Deserialize<'de>>(path: &str) -> HashMap<String, T> {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_else(|_| HashMap::new())
        },
        Err(_) => HashMap::new(),
    }
}

fn save_json<T: Serialize>(path: &str, data: &HashMap<String, T>) {
    let file = OpenOptions::new().write(true).create(true).open(path).expect("Cannot open file");
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, data).expect("Cannot write to file");
}

async fn get_user(data: web::Data<AppState>, web::Path((user_id, password)): web::Path<(String, String)>) -> HttpResponse {
    let users = data.users.lock().unwrap();
    match users.get(&user_id) {
        Some(stored_password) => {
            let mut hasher = Sha256::new();
            hasher.update(password);
            let result = format!("{:x}", hasher.finalize());
            if &result == stored_password {
                HttpResponse::Ok().body("1")
            } else {
                HttpResponse::Unauthorized().body("0")
            }
        },
        None => HttpResponse::Unauthorized().body("0"),
    }
}

async fn reg_user(data: web::Data<AppState>, web::Path((user_id, password)): web::Path<(String, String)>) -> HttpResponse {
    let mut users = data.users.lock().unwrap();
    if users.contains_key(&user_id) {
        HttpResponse::BadRequest().body("0")
    } else {
        let mut hasher = Sha256::new();
        hasher.update(password);
        let result = format!("{:x}", hasher.finalize());
        users.insert(user_id, result);
        save_json("Users.json", &*users);
        HttpResponse::Created().body("1")
    }
}

async fn check_username(data: web::Data<AppState>, web::Path(user_id: String)) -> HttpResponse {
let users = data.users.lock().unwrap();
if users.contains_key(&user_id) {
HttpResponse::Forbidden().body("0")
} else {
HttpResponse::Ok().body("1")
}
}

async fn get_user_list(data: web::Data<AppState>) -> HttpResponse {
    let users = data.users.lock().unwrap();
    let user_list: Vec<String> = users.keys().cloned().collect();
    HttpResponse::Ok().json(user_list)
}

async fn save_email(data: web::Data<AppState>, web::Path((user_id, email)): web::Path<(String, String)>) -> HttpResponse {
    let mut emails = data.emails.lock().unwrap();
    emails.insert(user_id, email);
    save_json("Emails.json", &*emails);
    HttpResponse::Ok().body("1")
}

async fn get_email(data: web::Data<AppState>, web::Path(user_id: String)) -> HttpResponse {
let emails = data.emails.lock().unwrap();
match emails.get(&user_id) {
Some(email) => HttpResponse::Ok().body(email.clone()),
None => HttpResponse::NotFound().body("0"),
}
}

async fn change_password(data: web::Data<AppState>, web::Path((user_id, password)): web::Path<(String, String)>) -> HttpResponse {
    let mut users = data.users.lock().unwrap();
    if users.contains_key(&user_id) {
        let mut hasher = Sha256::new();
        hasher.update(password);
        let result = format!("{:x}", hasher.finalize());
        users.insert(user_id, result);
        save_json("Users.json", &*users);
        HttpResponse::Ok().body("1")
    } else {
        HttpResponse::BadRequest().body("0")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let users = Arc::new(Mutex::new(load_json::<String>("Users.json")));
    let emails = Arc::new(Mutex::new(load_json::<String>("Emails.json")));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                users: Arc::clone(&users),
                emails: Arc::clone(&emails),
            }))
            .route("/get-user/{user_id}/{password}", web::get().to(get_user))
            .route("/reg-user/{user_id}/{password}", web::post().to(reg_user))
            .route("/chkusr/{user_id}", web::get().to(check_username))
            .route("/userlist/get", web::get().to(get_user_list))
            .route("/emails/save/{user_id}/{email}", web::post().to(save_email))
            .route("/emails/get/{user_id}", web::get().to(get_email))
            .route("/changepassword/{user_id}/{password}", web::post().to(change_password))
    })
        .bind("89.203.249.186:80")?
        .run()
        .await
}
