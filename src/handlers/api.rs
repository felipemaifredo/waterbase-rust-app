//Libs
use actix_web::{web, HttpResponse, Responder, HttpRequest};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

//Imports
use waterbase_rust_app::{Document, Value, Query, SharedDb};

//Types
#[derive(serde::Deserialize, Default)]
pub struct CreateDocumentQuery {
    pub timestamp: Option<bool>,
}

#[derive(serde::Deserialize, Default)]
pub struct ListDocumentsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

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

pub async fn api_list_collections(req: HttpRequest, db: web::Data<SharedDb>) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let collections = db.list_collections().await;
    HttpResponse::Ok().json(collections)
}

pub async fn api_list_documents(
    req: HttpRequest,
    db: web::Data<SharedDb>,
    path: web::Path<String>,
    query: web::Query<ListDocumentsQuery>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let collection_name = path.into_inner();
    match db.list_documents(&collection_name).await {
        Ok(docs) => {
            let mut list = Vec::new();
            let offset = query.offset.unwrap_or(0);
            let limit = query.limit;

            let doc_iter = docs.into_iter().skip(offset);
            let final_docs: Vec<_> = if let Some(lim) = limit {
                doc_iter.take(lim).collect()
            } else {
                doc_iter.collect()
            };

            for (id, doc) in final_docs {
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
    db: web::Data<SharedDb>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let (collection_name, doc_id) = path.into_inner();
    if let Some(doc) = db.get_document(&collection_name, &doc_id).await {
        HttpResponse::Ok().json(doc)
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Document '{}' not found in '{}'", doc_id, collection_name)
        }))
    }
}

pub async fn api_create_document(
    req: HttpRequest,
    db: web::Data<SharedDb>,
    path: web::Path<(String, String)>,
    query: web::Query<CreateDocumentQuery>,
    body: web::Json<Document>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let (collection_name, mut doc_id) = path.into_inner();

    if doc_id == "new_id" {
        doc_id = uuid::Uuid::new_v4().to_string();
    }

    let mut doc = body.into_inner();

    if query.timestamp.unwrap_or(false) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64;
        doc.fields.insert("_created_at".to_string(), Value::Number(now_ms));
    }

    match db.create_document(&collection_name, doc_id.clone(), doc).await {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({
            "status": "success",
            "id": doc_id
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

pub async fn api_update_document(
    req: HttpRequest,
    db: web::Data<SharedDb>,
    path: web::Path<(String, String)>,
    body: web::Json<HashMap<String, Value>>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let (collection_name, doc_id) = path.into_inner();
    match db.update_document(&collection_name, &doc_id, body.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "success" })),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({ "error": e })),
    }
}

pub async fn api_delete_document(
    req: HttpRequest,
    db: web::Data<SharedDb>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let (collection_name, doc_id) = path.into_inner();
    match db.delete_document(&collection_name, &doc_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "success" })),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({ "error": e })),
    }
}

pub async fn api_query_documents(
    req: HttpRequest,
    db: web::Data<SharedDb>,
    path: web::Path<String>,
    body: web::Json<Query>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let collection_name = path.into_inner();
    match db.execute_query(&collection_name, body.into_inner()).await {
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

#[allow(dead_code)]
pub async fn api_delete_collection(
    req: HttpRequest,
    db: web::Data<SharedDb>,
    path: web::Path<String>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let collection_name = path.into_inner();
    match db.delete_collection(&collection_name).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "success" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

pub async fn health(db: web::Data<SharedDb>) -> impl Responder {
    let collection_count = db.collections.read().await.len();
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "collections": collection_count
    }))
}
