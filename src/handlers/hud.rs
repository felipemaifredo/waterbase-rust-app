//Libs
use actix_web::{web, HttpResponse, Responder, HttpRequest, http::header};
use actix_web::cookie::{Cookie, SameSite};
use std::sync::Arc;
use tokio::sync::RwLock;
use waterbase_rust_app::{Database, Document};

//Imports
use crate::ui::templates::{login_page, dashboard_page, docs_page};

//Consts
pub const SESSION_COOKIE_NAME: &str = "waterbase_session";
pub const SESSION_COOKIE_VALUE: &str = "authenticated_admin";

//Types
#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateCollectionForm {
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct CreateDocumentForm {
    pub doc_id: String,
    pub json: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateDocumentForm {
    pub json: String,
}

#[derive(serde::Deserialize)]
pub struct QueryParams {
    pub collection: Option<String>,
}

//Funcs
pub fn is_authenticated(req: &HttpRequest) -> bool {
    if let Some(cookie) = req.cookie(SESSION_COOKIE_NAME) {
        cookie.value() == SESSION_COOKIE_VALUE
    } else {
        false
    }
}

pub async fn login_get(req: HttpRequest) -> impl Responder {
    if is_authenticated(&req) {
        return HttpResponse::SeeOther().insert_header((header::LOCATION, "/")).finish();
    }
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(login_page(None).into_string())
}

pub async fn login_post(form: web::Form<LoginForm>) -> impl Responder {
    let admin_user = std::env::var("ADMIN_USER").unwrap_or_else(|_| "admin".to_string());
    let admin_pass = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());

    let user_ok = form.username.as_deref() == Some(&admin_user);
    let pass_ok = form.password.as_deref() == Some(&admin_pass);

    if user_ok && pass_ok {
        let cookie = Cookie::build(SESSION_COOKIE_NAME, SESSION_COOKIE_VALUE)
            .path("/")
            .http_only(true)
            .max_age(actix_web::cookie::time::Duration::hours(8))
            .same_site(SameSite::Strict)
            .finish();
            
        HttpResponse::SeeOther()
            .insert_header((header::LOCATION, "/"))
            .cookie(cookie)
            .finish()
    } else {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(login_page(Some("Usuário ou senha incorretos.".to_string())).into_string())
    }
}

pub async fn logout() -> impl Responder {
    let cookie = Cookie::build(SESSION_COOKIE_NAME, "")
        .path("/")
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .same_site(SameSite::Strict)
        .finish();

    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, "/login"))
        .cookie(cookie)
        .finish()
}

pub async fn docs_get(req: HttpRequest) -> impl Responder {
    if !is_authenticated(&req) {
        return HttpResponse::SeeOther().insert_header((header::LOCATION, "/login")).finish();
    }
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(docs_page().into_string())
}

pub async fn index(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    query: web::Query<QueryParams>,
) -> impl Responder {
    if !is_authenticated(&req) {
        return HttpResponse::SeeOther().insert_header((header::LOCATION, "/login")).finish();
    }

    let db_read = db.read().await;
    let mut collections: Vec<String> = db_read.collections.keys().cloned().collect();
    collections.sort();

    let active_col = query.collection.clone();
    let mut documents = Vec::new();
    let mut error_msg = None;

    if let Some(ref col) = active_col {
        match db_read.list_documents(col) {
            Ok(docs) => documents = docs,
            Err(e) => error_msg = Some(e),
        }
    }

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(dashboard_page(collections, active_col, documents, error_msg).into_string())
}

pub async fn create_collection(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    form: web::Form<CreateCollectionForm>,
) -> impl Responder {
    if !is_authenticated(&req) {
        return HttpResponse::SeeOther().insert_header((header::LOCATION, "/login")).finish();
    }

    let mut db_write = db.write().await;
    db_write.create_collection(form.name.clone());

    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, format!("/?collection={}", form.name)))
        .finish()
}

pub async fn create_document(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    path: web::Path<String>,
    form: web::Form<CreateDocumentForm>,
) -> impl Responder {
    if !is_authenticated(&req) {
        return HttpResponse::SeeOther().insert_header((header::LOCATION, "/login")).finish();
    }

    let collection_name = path.into_inner();
    let mut db_write = db.write().await;

    match serde_json::from_str::<Document>(&form.json) {
        Ok(doc) => {
            if let Err(e) = db_write.create_document(&collection_name, form.doc_id.clone(), doc) {
                return HttpResponse::SeeOther()
                    .insert_header((header::LOCATION, format!("/?collection={}&error={}", collection_name, e)))
                    .finish();
            }
        }
        Err(e) => {
            return HttpResponse::SeeOther()
                .insert_header((header::LOCATION, format!("/?collection={}&error=JSON Inválido: {}", collection_name, e)))
                .finish();
        }
    }

    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, format!("/?collection={}", collection_name)))
        .finish()
}

pub async fn update_document(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    path: web::Path<(String, String)>,
    form: web::Form<UpdateDocumentForm>,
) -> impl Responder {
    if !is_authenticated(&req) {
        return HttpResponse::SeeOther().insert_header((header::LOCATION, "/login")).finish();
    }

    let (collection_name, doc_id) = path.into_inner();
    let mut db_write = db.write().await;

    match serde_json::from_str::<Document>(&form.json) {
        Ok(doc) => {
            if let Err(e) = db_write.update_document(&collection_name, &doc_id, doc.fields) {
                return HttpResponse::SeeOther()
                    .insert_header((header::LOCATION, format!("/?collection={}&error={}", collection_name, e)))
                    .finish();
            }
        }
        Err(e) => {
            return HttpResponse::SeeOther()
                .insert_header((header::LOCATION, format!("/?collection={}&error=JSON Inválido: {}", collection_name, e)))
                .finish();
        }
    }

    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, format!("/?collection={}", collection_name)))
        .finish()
}

pub async fn delete_document(
    req: HttpRequest,
    db: web::Data<Arc<RwLock<Database>>>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    if !is_authenticated(&req) {
        return HttpResponse::SeeOther().insert_header((header::LOCATION, "/login")).finish();
    }

    let (collection_name, doc_id) = path.into_inner();
    let mut db_write = db.write().await;

    if let Err(e) = db_write.delete_document(&collection_name, &doc_id) {
        return HttpResponse::SeeOther()
            .insert_header((header::LOCATION, format!("/?collection={}&error={}", collection_name, e)))
            .finish();
    }

    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, format!("/?collection={}", collection_name)))
        .finish()
}
