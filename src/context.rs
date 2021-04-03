use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct CodeContext {
    pub hljsclass: &'static str,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct PageContext {
    pub code: String,
    pub url: String,
    pub webroot: String,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AppContext {
    pub title: String,
    pub label: String,
    pub webroot: String,
    pub url: String,
    pub languages: Vec<String>,
}
