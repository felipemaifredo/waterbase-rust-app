//Libs
use actix_web::{web, HttpResponse, Responder, HttpRequest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use waterbase_rust_app::{Database, Document, Value, Query};

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

pub async fn api_list_collections(req: HttpRequest, db: web::Data<Arc<RwLock<Database>>>) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let db_read = db.read().await;
    let collections: Vec<String> = db_read.collections.keys().cloned().collect();
    HttpResponse::Ok().json(collections)
}

pub async fn api_list_documents(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    path: web::Path<String>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let collection_name = path.into_inner();
    let db_read = db.read().await;
    match db_read.list_documents(&collection_name) {
        Ok(docs) => {
            let mut list = Vec::new();
            for (id, doc) in docs {
                let mut doc_json = serde_json::to_value(&doc).unwrap_or(serde_json::Value::Null);
                if let serde_json::Value::Object(ref mut map) = doc_json {
                    map.insert("id".to_string(), serde_json::Value::String(id));
                }
                list.push(doc_json);
            }
            HttpResponse::Ok().json(list)
        }
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({ "error": e })),
    }
}

pub async fn api_get_document(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let (collection_name, doc_id) = path.into_inner();
    let db_read = db.read().await;
    if let Some(doc) = db_read.get_document(&collection_name, &doc_id) {
        HttpResponse::Ok().json(doc)
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Document '{}' not found in '{}'", doc_id, collection_name)
        }))
    }
}

pub async fn api_create_document(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    path: web::Path<(String, String)>,
    body: web::Json<Document>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let (collection_name, mut doc_id) = path.into_inner();
    
    if doc_id == "new_id" {
        doc_id = uuid::Uuid::new_v4().to_string();
    }

    let mut db_write = db.write().await;
    match db_write.create_document(&collection_name, doc_id.clone(), body.into_inner()) {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "status": "success",
            "id": doc_id
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

pub async fn api_update_document(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    path: web::Path<(String, String)>,
    body: web::Json<HashMap<String, Value>>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let (collection_name, doc_id) = path.into_inner();
    let mut db_write = db.write().await;
    match db_write.update_document(&collection_name, &doc_id, body.into_inner()) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "success" })),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({ "error": e })),
    }
}

pub async fn api_delete_document(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let (collection_name, doc_id) = path.into_inner();
    let mut db_write = db.write().await;
    match db_write.delete_document(&collection_name, &doc_id) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "success" })),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({ "error": e })),
    }
}

pub async fn api_query_documents(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    path: web::Path<String>,
    body: web::Json<Query>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let collection_name = path.into_inner();
    let db_read = db.read().await;
    match db_read.execute_query(&collection_name, body.into_inner()) {
        Ok(docs) => {
            let mut list = Vec::new();
            for (id, doc) in docs {
                let mut doc_json = serde_json::to_value(&doc).unwrap_or(serde_json::Value::Null);
                if let serde_json::Value::Object(ref mut map) = doc_json {
                    map.insert("id".to_string(), serde_json::Value::String(id));
                }
                list.push(doc_json);
            }
            HttpResponse::Ok().json(list)
        }
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({ "error": e })),
    }
}
