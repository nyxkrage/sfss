pub mod sfss_format;
pub mod sfss_templates;

#[macro_use]
extern crate rocket;
extern crate lazy_static;

use rocket::{http::Status, response::content::Html};

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
}

lazy_static::lazy_static! {
    static ref APP_CONTEXT: AppContext = {
        dotenv::dotenv().ok();
        AppContext {
            title: std::env::var("SFSS_TITLE").unwrap(),
            label: std::env::var("SFSS_LABEL").unwrap(),
            webroot: std::env::var("SFSS_ROOT").unwrap(),
            url: std::env::var("SFSS_URL").unwrap(),
        }
    };
}

#[post("/upload", data = "<data>")]
async fn upload(data: SfssFile) -> Result<Html<String>, Status> {
    let template = if data.password.is_some() {
        sfss_templates::UPLOAD_PASSWORD
    } else {
        sfss_templates::UPLOAD
    };
    let ctx = Context {
        code: data.hash, //sfss_file.hash,
        url: APP_CONTEXT.url.clone(),
        webroot: APP_CONTEXT.webroot.clone(),
        password: data.password,
    };
    match handlebars::Handlebars::new().render_template(template, &ctx) {
        Ok(v) => Ok(Html(v)),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(Status::InternalServerError)
        }
    }
}

#[post("/upload/api", data = "<data>")]
fn api_upload(data: SfssFile) -> Result<String, Status> {
    let template = if data.password.is_some() {
        sfss_templates::UPLOAD_API_PASSWORD
    } else {
        sfss_templates::UPLOAD_API
    };
    let ctx = Context {
        code: data.hash, //sfss_file.hash,
        url: APP_CONTEXT.url.clone(),
        webroot: APP_CONTEXT.webroot.clone(),
        password: data.password,
    };
    match handlebars::Handlebars::new().render_template(template, &ctx) {
        Ok(v) => Ok(v),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(Status::InternalServerError)
        }
    }
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

#[get("/favicon.ico")]
fn favicon() -> Status {
    Status::NotFound
}

// The launch attribute, tells that this is the entry point for the application
#[launch]
async fn rocket() -> rocket::Rocket {
    dotenv::dotenv().ok();
    rocket::ignite().mount("/", routes![file, upload, api_upload, root, favicon])
}
