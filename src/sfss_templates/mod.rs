use crate::include_base_str;
pub static INDEX: &'static str = include_base_str!("templates/index.hbs");
pub static UPLOAD: &'static str = include_base_str!("templates/upload.hbs");
pub static UPLOAD_PASSWORD: &'static str = include_base_str!("templates/upload_password.hbs");
pub static UPLOAD_API: &'static str = include_base_str!("templates/upload_api.hbs");
pub static UPLOAD_API_PASSWORD: &'static str =
    include_base_str!("templates/upload_api_password.hbs");
pub static CODE: &'static str = include_base_str!("templates/code.hbs");

pub fn get_template(api: bool, password: bool) -> &'static str {
    if api {
        if password {
            UPLOAD_API_PASSWORD
        } else {
            UPLOAD_API
        }
    } else {
        if password {
            UPLOAD_PASSWORD
        } else {
            UPLOAD
        }
    }
}
