mod sfss_format;
mod sfss_templates;
#[macro_use]
mod utils;

#[macro_use]
extern crate rocket;
extern crate lazy_static;

use rocket::{http::Status, response::content::{Html, Json}};

use serde::{Deserialize, Serialize};
use sfss_format::SfssFile;

#[derive(Serialize, Deserialize)]
struct Context {
    code: String,
    url: String,
    webroot: String,
    password: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AppContext {
    title: String,
    label: String,
    webroot: String,
    url: String,
    languages: Vec<String>,
}

lazy_static::lazy_static! {
    static ref APP_CONTEXT: AppContext = {
        dotenv::dotenv().ok();
        AppContext {
            title: std::env::var("SFSS_TITLE").unwrap(),
            label: std::env::var("SFSS_LABEL").unwrap(),
            webroot: std::env::var("SFSS_ROOT").unwrap(),
            url: std::env::var("SFSS_URL").unwrap(),
            languages: serde_json::from_str(highlightjs_rs::LANGSJSON).unwrap(),
        }
    };
}

fn upload(data: SfssFile, api: bool) -> Result<Html<String>, Status> {
    let passworded = data.password.is_some();
    let ctx = Context {
        code: data.hash, //sfss_file.hash,
        url: APP_CONTEXT.url.clone(),
        webroot: APP_CONTEXT.webroot.clone(),
        password: data.password,
    };
    match handlebars::Handlebars::new().render_template(sfss_templates::get_template(api, passworded), &ctx) {
        Ok(v) => Ok(Html(v)),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[post("/upload", data = "<data>")]
fn upload_web(data: SfssFile) -> Result<Html<String>, Status> {
    upload(data, false)
}

#[post("/upload/api", data = "<data>")]
fn upload_api(data: SfssFile) -> Result<Html<String>, Status> {
    upload(data, true)
}

#[post("/upload/api?json", data = "<data>")]
fn upload_json(data: SfssFile) -> Result<Json<String>, Status> {
    if let Some(password) = data.password {
        Ok(Json(format!("{{\"hash\": \"{}\", \"password\": \"{}\"}}", data.hash, password).to_owned()))
    } else {
        Ok(Json(format!("{{\"hash\": \"{}\", \"password\": null}}", data.hash).to_owned()))
    }
}

#[get("/<code>/raw?<password>")]
fn raw(code: String, password: Option<String>) -> Result<SfssFile, Status> {
    file(code, password)
}
#[get("/<code>?<password>")]
fn file(code: String, password: Option<String>) -> Result<SfssFile, Status> {
    match SfssFile::new(code.clone(), false) {
        Ok(file) => {
            if let Some(_) = file.password {
                if file.password != password {
                    return Err(Status::Forbidden);
                };
            }
            Ok(file)
        }
        Err(e) => {
            eprintln!("Error serving file with code {}: {:?}", &code, e);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/")]
fn root() -> Result<Html<String>, Status> {
    match handlebars::Handlebars::new().render_template(sfss_templates::INDEX, &*APP_CONTEXT) {
        Ok(v) => Ok(Html(v)),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[get("/hljs.js")]
fn hljs() -> &'static str {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/highlight.js"))
}

#[get("/style.css")]
fn style() -> &'static str {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/style.css"))
}

#[get("/favicon.ico")]
fn favicon() -> Status {
    Status::NotFound
}

#[get("/robots.txt")]
fn robots() -> &'static str {
    r#"
    user-agent: *
    Allow: /$
    Disallow: /
    "#
}

#[get("/languages")]
fn langs() -> Json<&'static str> {
    Json(highlightjs_rs::LANGSJSON)
}

#[get("/languages/api")]
fn langs_api() -> &'static str {
    highlightjs_rs::CLASSRAW
}

// The launch attribute, tells that this is the entry point for the application
#[launch]
async fn rocket() -> rocket::Rocket {
    dotenv::dotenv().ok();
    rocket::ignite()
        .mount("/", routes![file, raw, upload_api, upload_json, upload_web, root, favicon, style, hljs, robots, langs, langs_api])
        .attach(rocket::fairing::AdHoc::on_response("Allow CORS", |_, res| {
            Box::pin(async move {
                res.adjoin_raw_header("Access-Control-Allow-Origin","*");
            })
        }))
}
