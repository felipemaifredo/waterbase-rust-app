//Libs
use actix_web::{web, HttpRequest, HttpResponse, Responder};
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
    pub email: String,
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
        bcrypt::hash(&combined_password, 10)
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

    // Busca o documento de autenticação pelo email
    let query = waterbase_rust_app::Query {
        r#where: Some(vec![waterbase_rust_app::WhereFilter {
            field: "email".to_string(),
            op: waterbase_rust_app::WhereOp::Equal,
            value: Value::String(payload.email.clone()),
        }]),
        order_by: None,
        limit: Some(1),
        offset: None,
    };

    let results = match db.execute_query("authentication", query).await {
        Ok(docs) if !docs.is_empty() => docs,
        _ => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "authenticated": false,
                "error": "Credenciais inválidas"
            }));
        }
    };

    let (user_id, auth_doc) = results.into_iter().next().unwrap();

    let password_hash_val = match auth_doc.fields.get("password_hash") {
        Some(Value::String(s)) => s.clone(),
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
    let verified = tokio::task::spawn_blocking(move || {
        bcrypt::verify(&combined_password, &password_hash_val)
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
        sessions.insert(session_token.clone(), user_id.clone());
    }

    HttpResponse::Ok().json(serde_json::json!({
        "id": user_id,
        "token": session_token
    }))
}

pub async fn api_auth_revalidate(
    req: HttpRequest,
    db: web::Data<SharedDb>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }

    let old_token = match req.headers().get("X-Session-Token") {
        Some(v) => match v.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return HttpResponse::Unauthorized().json(serde_json::json!({
                    "authenticated": false,
                    "error": "Token de sessão inválido"
                }));
            }
        },
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "authenticated": false,
                "error": "Sessão inválida ou expirada"
            }));
        }
    };

    let user_id = {
        let sessions = db.sessions.read().await;
        match sessions.get(&old_token) {
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
        sessions.remove(&old_token);
        sessions.insert(new_token.clone(), user_id.clone());
    }

    HttpResponse::Ok().json(serde_json::json!({
        "id": user_id,
        "token": new_token
    }))
}

pub async fn api_auth_logout(
    req: HttpRequest,
    db: web::Data<SharedDb>,
) -> impl Responder {
    if !is_api_authenticated(&req) {
        return HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Unauthorized" }));
    }

    if let Some(token_header) = req.headers().get("X-Session-Token") {
        if let Ok(token) = token_header.to_str() {
            let mut sessions = db.sessions.write().await;
            sessions.remove(token);
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
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
            "email": "user@example.com",
            "password": "my_password"
        });

        let req = actix_web::test::TestRequest::post()
            .uri("/auth/login")
            .insert_header(("Authorization", "Bearer test_api_key"))
            .set_json(&login_payload)
            .to_request();

        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let body: serde_json::Value = actix_web::test::read_body_json(resp).await;
        assert_eq!(body.get("id").unwrap().as_str().unwrap(), user_id);
        let session_token = body.get("token").unwrap().as_str().unwrap().to_string();
        assert!(!session_token.is_empty());

        // 3. Test Revalidate
        let req = actix_web::test::TestRequest::get()
            .uri("/auth/revalidate")
            .insert_header(("Authorization", "Bearer test_api_key"))
            .insert_header(("X-Session-Token", session_token.clone()))
            .to_request();

        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let body: serde_json::Value = actix_web::test::read_body_json(resp).await;
        assert_eq!(body.get("id").unwrap().as_str().unwrap(), user_id);
        let new_token = body.get("token").unwrap().as_str().unwrap().to_string();
        assert!(!new_token.is_empty());
        assert_ne!(new_token, session_token);

        // 4. Test Logout
        let req = actix_web::test::TestRequest::post()
            .uri("/auth/logout")
            .insert_header(("Authorization", "Bearer test_api_key"))
            .insert_header(("X-Session-Token", new_token))
            .to_request();

        let resp = actix_web::test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}
