//Libs
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use actix_web::cookie::{Cookie, SameSite};
use std::collections::HashMap;

//Imports
use waterbase_rust_app::{Document, SharedDb, Value};
use super::is_api_authenticated;

//Types
#[derive(serde::Deserialize)]
pub struct RegisterPayload {
    pub email: String,
    pub password: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(serde::Deserialize)]
pub struct LoginPayload {
    pub id: String,
    pub password: String,
}

//Funcs
pub async fn api_auth_register(
    req: HttpRequest,
    db: web::Data<SharedDb>,
    body: web::Json<RegisterPayload>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let payload = body.into_inner();
    
    let hash_key = match std::env::var("AUTH_HASH_KEY") {
        Ok(val) => val,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Erro interno: AUTH_HASH_KEY não configurada"
            }));
        }
    };
    
    let combined_password = format!("{}{}", payload.password, hash_key);
    let password_hash = match tokio::task::spawn_blocking(move || {
        bcrypt::hash(&combined_password, bcrypt::DEFAULT_COST)
    }).await {
        Ok(Ok(h)) => h,
        Ok(Err(e)) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Erro ao gerar hash da senha: {}", e)
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Erro interno ao processar senha"
            }));
        }
    };
    
    let mut auth_fields = HashMap::new();
    auth_fields.insert("email".to_string(), Value::String(payload.email));
    auth_fields.insert("password_hash".to_string(), Value::String(password_hash));
    let auth_doc = Document { fields: auth_fields };
    
    let user_id = uuid::Uuid::new_v4().to_string();
    if let Err(_) = db.create_document("authentication", user_id.clone(), auth_doc).await {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": "Usuário já existe"
        }));
    }
    
    let user_doc = Document { fields: payload.extra };
    
    if let Err(e) = db.create_document("users", user_id.clone(), user_doc).await {
        let _ = db.delete_document("authentication", &user_id).await;
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Erro ao criar perfil de usuário: {}", e)
        }));
    }
    
    HttpResponse::Created().json(serde_json::json!({
        "id": user_id
    }))
}

pub async fn api_auth_login(
    req: HttpRequest,
    db: web::Data<SharedDb>,
    body: web::Json<LoginPayload>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let payload = body.into_inner();
    
    let auth_doc = match db.get_document("authentication", &payload.id).await {
        Some(doc) => doc,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "authenticated": false,
                "error": "Credenciais inválidas"
            }));
        }
    };
    
    let password_hash_val = match auth_doc.fields.get("password_hash") {
        Some(Value::String(s)) => s,
        _ => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Dados de autenticação corrompidos no banco"
            }));
        }
    };
    
    let hash_key = match std::env::var("AUTH_HASH_KEY") {
        Ok(val) => val,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Erro interno: AUTH_HASH_KEY não configurada"
            }));
        }
    };
    
    let combined_password = format!("{}{}", payload.password, hash_key);
    let hash_to_verify = password_hash_val.clone();
    let verified = tokio::task::spawn_blocking(move || {
        bcrypt::verify(&combined_password, &hash_to_verify)
    }).await;
    match verified {
        Ok(Ok(true)) => {}
        _ => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "authenticated": false,
                "error": "Credenciais inválidas"
            }));
        }
    }
    
    let session_token = uuid::Uuid::new_v4().to_string();
    {
        let mut sessions = db.sessions.write().await;
        sessions.insert(session_token.clone(), payload.id.clone());
    }
    
    let is_prod = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string()) == "prod";
    let cookie = Cookie::build("auth_session", session_token)
        .path("/")
        .http_only(true)
        .secure(is_prod)
        .max_age(actix_web::cookie::time::Duration::hours(8))
        .same_site(SameSite::Strict)
        .finish();
        
    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({
            "id": payload.id
        }))
}

pub async fn api_auth_revalidate(
    req: HttpRequest,
    db: web::Data<SharedDb>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    let session_cookie = match req.cookie("auth_session") {
        Some(c) => c,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "authenticated": false,
                "error": "Sessão inválida ou expirada"
            }));
        }
    };
    
    let old_token = session_cookie.value();
    
    let user_id = {
        let sessions = db.sessions.read().await;
        match sessions.get(old_token) {
            Some(id) => id.clone(),
            None => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "authenticated": false,
                    "error": "Sessão inválida ou expirada"
                }));
            }
        }
    };
    
    let new_token = uuid::Uuid::new_v4().to_string();
    {
        let mut sessions = db.sessions.write().await;
        sessions.remove(old_token);
        sessions.insert(new_token.clone(), user_id.clone());
    }
    
    let is_prod = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string()) == "prod";
    let cookie = Cookie::build("auth_session", new_token)
        .path("/")
        .http_only(true)
        .secure(is_prod)
        .max_age(actix_web::cookie::time::Duration::hours(8))
        .same_site(SameSite::Strict)
        .finish();
        
    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({
            "id": user_id
        }))
}

pub async fn api_auth_logout(
    req: HttpRequest,
    db: web::Data<SharedDb>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }
    if let Some(cookie) = req.cookie("auth_session") {
        let mut sessions = db.sessions.write().await;
        sessions.remove(cookie.value());
    }
    
    let cookie = Cookie::build("auth_session", "")
        .path("/")
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::ZERO)
        .same_site(SameSite::Strict)
        .finish();
        
    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({
            "success": true,
            "message": "Logout efetuado com sucesso"
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use waterbase_rust_app::Database;

    #[actix_web::test]
    async fn test_auth_cycle() {
        unsafe {
            std::env::set_var("AUTH_HASH_KEY", "test_hash_key_123");
            std::env::set_var("API_KEY", "test_api_key");
        }
        
        let db = Database::new();
        let shared_db = SharedDb::from_database(db);
        let app_data = web::Data::new(shared_db);
        
        let app = actix_web::test::init_service(
            actix_web::App::new()
                .app_data(app_data.clone())
                .route("/auth/register", web::post().to(api_auth_register))
                .route("/auth/login", web::post().to(api_auth_login))
                .route("/auth/revalidate", web::get().to(api_auth_revalidate))
                .route("/auth/logout", web::post().to(api_auth_logout))
        ).await;
        
        // 1. Test Register
        let register_payload = serde_json::json!({
            "email": "user@example.com",
            "password": "my_password",
            "name": "Jane Doe",
            "role": "admin"
        });
        
        let req = actix_web::test::TestRequest::post()
            .uri("/auth/register")
            .insert_header(("Authorization", "Bearer test_api_key"))
            .set_json(&register_payload)
            .to_request();
            
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);
        
        let body: serde_json::Value = actix_web::test::read_body_json(resp).await;
        let user_id = body.get("id").unwrap().as_str().unwrap().to_string();
        assert!(!user_id.is_empty());
        
        // 2. Test Login
        let login_payload = serde_json::json!({
            "id": user_id,
            "password": "my_password"
        });
        
        let req = actix_web::test::TestRequest::post()
            .uri("/auth/login")
            .insert_header(("Authorization", "Bearer test_api_key"))
            .set_json(&login_payload)
            .to_request();
            
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
        
        // Check session cookie is set and extract session token before consuming resp
        let session_token = {
            let cookies = resp.headers().get(actix_web::http::header::SET_COOKIE).unwrap();
            let cookie_str = cookies.to_str().unwrap();
            assert!(cookie_str.contains("auth_session="));
            cookie_str
                .split(';')
                .next()
                .unwrap()
                .split('=')
                .nth(1)
                .unwrap()
                .to_string()
        };
        
        let body: serde_json::Value = actix_web::test::read_body_json(resp).await;
        assert_eq!(body.get("id").unwrap().as_str().unwrap(), user_id);
            
        // 3. Test Revalidate
        let req = actix_web::test::TestRequest::get()
            .uri("/auth/revalidate")
            .insert_header(("Authorization", "Bearer test_api_key"))
            .insert_header((actix_web::http::header::COOKIE, format!("auth_session={}", session_token)))
            .to_request();
            
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
        
        let body: serde_json::Value = actix_web::test::read_body_json(resp).await;
        assert_eq!(body.get("id").unwrap().as_str().unwrap(), user_id);
        
        // 4. Test Logout
        let req = actix_web::test::TestRequest::post()
            .uri("/auth/logout")
            .insert_header(("Authorization", "Bearer test_api_key"))
            .insert_header((actix_web::http::header::COOKIE, format!("auth_session={}", session_token)))
            .to_request();
            
        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}
