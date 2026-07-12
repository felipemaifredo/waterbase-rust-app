//Libs
use actix_web::{web, HttpRequest, HttpResponse, Responder};

//Imports
use waterbase_rust_app::SharedDb;

//Modules
pub mod auth;
pub mod db;

//Re-exports
pub use auth::{api_auth_login, api_auth_logout, api_auth_register, api_auth_revalidate};
pub use db::{api_create_document, api_delete_document, api_get_document, api_list_collections, api_list_documents, api_query_documents, api_update_document};

//Funcs
pub fn is_api_authenticated(req: &HttpRequest) -> bool {
    let expected_key = std::env::var("API_KEY").unwrap_or_else(|_| "waterbase_secret_token_123".to_string());
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                return token == expected_key;
            }
        }
    }
    false
}

pub async fn health(db: web::Data<SharedDb>) -> impl Responder {
    let collection_count = db.collections.read().await.len();
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "collections": collection_count
    }))
}
