//Libs
use actix_web::{web, App, HttpServer};
use std::sync::{Arc, RwLock};
use dotenvy::dotenv;

//Imports
use waterbase_rust_app::{Database, Document, Value};

//Modules
mod ui;
mod handlers;

//Main
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Carrega variáveis do arquivo .env
    dotenv().ok();

    // Cria ou carrega o banco de dados persistido no disco
    let database = Arc::new(RwLock::new(
        Database::new_with_storage("data".to_string()).expect("Falha ao inicializar armazenamento no disco")
    ));

    // Seed de dados iniciais apenas se o banco estiver completamente vazio
    {
        let mut db = database.write().unwrap();
        if db.collections.is_empty() {
            db.create_collection("users".to_string());
            let mut fields = std::collections::HashMap::new();
            fields.insert("nome".to_string(), Value::String("Felipe".to_string()));
            fields.insert("idade".to_string(), Value::Number(29.0));
            fields.insert("ativo".to_string(), Value::Boolean(true));
            let doc = Document::new(fields);
            let _ = db.create_document("users", "felipe_id".to_string(), doc);
        }
    }

    let shared_db = web::Data::new(database);
    let port_str = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port = port_str.parse::<u16>().unwrap_or(8080);
    println!("Servidor rodando em http://localhost:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(shared_db.clone())
            // HUD Dashboard Routes
            .route("/", web::get().to(handlers::hud::index))
            .route("/login", web::get().to(handlers::hud::login_get))
            .route("/login", web::post().to(handlers::hud::login_post))
            .route("/logout", web::get().to(handlers::hud::logout))
            .route("/docs", web::get().to(handlers::hud::docs_get))
            .route("/collections", web::post().to(handlers::hud::create_collection))
            .route("/collections/{col}/documents", web::post().to(handlers::hud::create_document))
            .route("/collections/{col}/documents/{doc}/update", web::post().to(handlers::hud::update_document))
            .route("/collections/{col}/documents/{doc}/delete", web::post().to(handlers::hud::delete_document))
            
            // External API Routes (REST v1)
            .service(
                web::scope("/api/v1")
                    .route("/collections", web::get().to(handlers::api::api_list_collections))
                    .route("/{col}", web::get().to(handlers::api::api_list_documents))
                    .route("/{col}/query", web::post().to(handlers::api::api_query_documents))
                    .route("/{col}/{doc}", web::get().to(handlers::api::api_get_document))
                    .route("/{col}/{doc}", web::post().to(handlers::api::api_create_document))
                    .route("/{col}/{doc}", web::put().to(handlers::api::api_update_document))
                    .route("/{col}/{doc}", web::delete().to(handlers::api::api_delete_document))
            )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
