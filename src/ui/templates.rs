//Libs
use maud::{html, Markup, DOCTYPE};
use waterbase_rust_app::Document;

//Imports
use crate::ui::css::get_css;

//Main
pub fn login_page(error_msg: Option<String>) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap";
                title { "Waterbase - Login" }
                style { (get_css()) }
            }
            body class="login-body" {
                div class="login-card" {
                    h1 { "Waterbase" }
                    p class="subtitle" { "Painel Administrativo do Banco de Dados" }
                    
                    form method="POST" action="/login" {
                        div class="input-group" {
                            label for="username" { "Usuário" }
                            input type="text" id="username" name="username" placeholder="Digite o usuário" required;
                        }
                        div class="input-group" {
                            label for="password" { "Senha" }
                            input type="password" id="password" name="password" placeholder="Digite a senha" required;
                        }
                        
                        @if let Some(err) = error_msg {
                            p class="error-msg" { (err) }
                        }
                        
                        button type="submit" class="btn-primary" { "Entrar" }
                    }
                }
            }
        }
    }
}

pub fn docs_page() -> Markup {
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "waterbase_secret_token_123".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let base_url = format!("http://localhost:{}", port);
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap";
                title { "Waterbase API Docs" }
                style { (get_css()) }
            }
            body {
                div class="app-container" {
                    div class="sidebar" {
                        div class="sidebar-header" {
                            h1 { "Waterbase HUD" }
                        }
                        h2 { "Navegação" }
                        div class="sidebar-menu" {
                            a class="sidebar-item" href="/" {
                                span { "🖥️ Voltar ao HUD" }
                            }
                            a class="sidebar-item active" href="/docs" {
                                span { "📖 API Docs" }
                            }
                        }
                    }

                    div class="main-area" {
                        div class="header" {
                            h1 { "Documentação da API REST" }
                            div class="header-actions" {
                                a class="logout-btn" href="/" style="color: var(--primary); border-color: var(--primary); margin-right: 10px;" { "🖥️ Voltar ao HUD" }
                                a class="logout-btn" href="/logout" { "Sair" }
                            }
                        }

                        div class="content-body" {
                            div style="margin-bottom: 30px; border-bottom: 1px solid var(--border); padding-bottom: 20px;" {
                                h2 style="margin: 0 0 10px 0; font-size: 22px;" { "Integração Externa" }
                                p style="color: var(--text-secondary); line-height: 1.6;" {
                                    "A API REST do Waterbase permite que qualquer aplicação externa se integre e manipule dados do seu banco de dados."
                                    " Todas as requisições para a API exigem o cabeçalho de autenticação Bearer Token contendo a sua "
                                    code { "API_KEY" } " configurada no arquivo " code { ".env" } "."
                                }
                                div style="background-color: var(--surface); border: 1px solid var(--border); padding: 15px; border-radius: 8px; margin-top: 15px;" {
                                    strong { "Header de Autenticação Obrigatório:" }
                                    pre class="code-block" style="margin-top: 8px;" {
                                        "Authorization: Bearer " (api_key)
                                    }
                                }
                            }

                            div class="api-endpoint" {
                                div class="api-header" {
                                    span class="badge badge-get" { "GET" }
                                    span class="api-path" { "/api/v1/collections" }
                                }
                                p class="api-description" { "Retorna a lista de todas as coleções cadastradas em disco." }
                                div class="api-details" {
                                    div class="details-box" {
                                        h4 { "Comando Curl" }
                                        pre class="code-block" {
                                            "curl -X GET " (base_url) "/api/v1/collections \\\n     -H \"Authorization: Bearer " (api_key) "\""
                                        }
                                    }
                                    div class="details-box" {
                                        h4 { "Resposta de Exemplo" }
                                        pre class="code-block" {
                                            "[\n  \"users\",\n  \"products\"\n]"
                                        }
                                    }
                                }
                            }

                            div class="api-endpoint" {
                                div class="api-header" {
                                    span class="badge badge-get" { "GET" }
                                    span class="api-path" { "/api/v1/{collection}" }
                                }
                                p class="api-description" { "Retorna uma lista com todos os documentos contidos na coleção especificada, em formato de array de objetos com o id injetado." }
                                div class="api-details" {
                                    div class="details-box" {
                                        h4 { "Comando Curl" }
                                        pre class="code-block" {
                                            "curl -X GET " (base_url) "/api/v1/users \\\n     -H \"Authorization: Bearer " (api_key) "\""
                                        }
                                    }
                                    div class="details-box" {
                                        h4 { "Resposta de Exemplo" }
                                        pre class="code-block" {
                                            "[\n  {\n    \"id\": \"felipe_id\",\n    \"nome\": \"Felipe\",\n    \"idade\": 29.0,\n    \"ativo\": true\n  }\n]"
                                        }
                                    }
                                }
                            }

                            div class="api-endpoint" {
                                div class="api-header" {
                                    span class="badge badge-get" { "GET" }
                                    span class="api-path" { "/api/v1/{collection}/{document_id}" }
                                }
                                p class="api-description" { "Busca e retorna as propriedades de um documento específico pelo ID." }
                                div class="api-details" {
                                    div class="details-box" {
                                        h4 { "Comando Curl" }
                                        pre class="code-block" {
                                            "curl -X GET " (base_url) "/api/v1/users/felipe_id \\\n     -H \"Authorization: Bearer " (api_key) "\""
                                        }
                                    }
                                    div class="details-box" {
                                        h4 { "Resposta de Exemplo" }
                                        pre class="code-block" {
                                            "{\n  \"nome\": \"Felipe\",\n  \"idade\": 29.0,\n  \"ativo\": true\n}"
                                        }
                                    }
                                }
                            }

                            div class="api-endpoint" {
                                div class="api-header" {
                                    span class="badge badge-post" { "POST" }
                                    span class="api-path" { "/api/v1/{collection}/{document_id}" }
                                }
                                p class="api-description" { "Cria ou substitui completamente um documento. Se o ID enviado for 'new_id', o banco gerará um UUID v4 dinamicamente." }
                                div class="api-details" {
                                    div class="details-box" {
                                        h4 { "Comando Curl" }
                                        pre class="code-block" {
                                            "curl -X POST " (base_url) "/api/v1/users/new_id \\\n     -H \"Authorization: Bearer " (api_key) " \\\n     -H \"Content-Type: application/json\" \\\n     -d '{\"nome\": \"Maria\", \"idade\": 25}'"
                                        }
                                    }
                                    div class="details-box" {
                                        h4 { "Resposta de Exemplo (Auto-gerado)" }
                                        pre class="code-block" {
                                            "{\n  \"status\": \"success\",\n  \"id\": \"f81d4fae-7dec-11d0-a765-00a0c91e6bf6\"\n}"
                                        }
                                    }
                                }
                            }

                            div class="api-endpoint" {
                                div class="api-header" {
                                    span class="badge badge-put" { "PUT" }
                                    span class="api-path" { "/api/v1/{collection}/{document_id}" }
                                }
                                p class="api-description" { "Faz a mesclagem (update parcial) de dados de um documento existente, substituindo apenas os campos enviados." }
                                div class="api-details" {
                                    div class="details-box" {
                                        h4 { "Comando Curl" }
                                        pre class="code-block" {
                                            "curl -X PUT " (base_url) "/api/v1/users/felipe_id \\\n     -H \"Authorization: Bearer " (api_key) " \\\n     -H \"Content-Type: application/json\" \\\n     -d '{\"idade\": 30, \"email\": \"felipe@example.com\"}'"
                                        }
                                    }
                                    div class="details-box" {
                                        h4 { "Resposta de Exemplo" }
                                        pre class="code-block" {
                                            "{\n  \"status\": \"success\"\n}"
                                        }
                                    }
                                }
                            }

                            div class="api-endpoint" {
                                div class="api-header" {
                                    span class="badge badge-post" { "POST" }
                                    span class="api-path" { "/api/v1/{collection}/query" }
                                }
                                p class="api-description" { "Executa uma consulta complexa retornando apenas os documentos que atendem aos filtros, ordenação e limite de resultados." }
                                div class="api-details" {
                                    div class="details-box" {
                                        h4 { "Comando Curl" }
                                        pre class="code-block" {
                                            "curl -X POST " (base_url) "/api/v1/users/query \\\n     -H \"Authorization: Bearer " (api_key) " \\\n     -H \"Content-Type: application/json\" \\\n     -d '{\n          \"where\": [{\"field\": \"idade\", \"op\": \">\", \"value\": 22}],\n          \"order_by\": {\"field\": \"idade\", \"direction\": \"desc\"},\n          \"limit\": 5\n         }'"
                                        }
                                    }
                                    div class="details-box" {
                                        h4 { "Resposta de Exemplo" }
                                        pre class="code-block" {
                                            "[\n  {\n    \"id\": \"carlos_id\",\n    \"nome\": \"Carlos\",\n    \"idade\": 30.0,\n    \"ativo\": true\n  }\n]"
                                        }
                                    }
                                }
                            }

                            div class="api-endpoint" {
                                div class="api-header" {
                                    span class="badge badge-delete" { "DELETE" }
                                    span class="api-path" { "/api/v1/{collection}/{document_id}" }
                                }
                                p class="api-description" { "Exclui permanentemente o documento correspondente da memória e do disco." }
                                div class="api-details" {
                                    div class="details-box" {
                                        h4 { "Comando Curl" }
                                        pre class="code-block" {
                                            "curl -X DELETE " (base_url) "/api/v1/users/felipe_id \\\n     -H \"Authorization: Bearer " (api_key) "\""
                                        }
                                    }
                                    div class="details-box" {
                                        h4 { "Resposta de Exemplo" }
                                        pre class="code-block" {
                                            "{\n  \"status\": \"success\"\n}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn dashboard_page(
    collections: Vec<String>,
    active_collection: Option<String>,
    documents: Vec<(String, Document)>,
    error_msg: Option<String>,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap";
                title { "Waterbase HUD" }
                style { (get_css()) }
            }
            body {
                div class="app-container" {
                    // Sidebar
                    div class="sidebar" {
                        div class="sidebar-header" {
                            h1 { "Waterbase HUD" }
                        }
                        
                        h2 { "Coleções" }
                        div class="sidebar-menu" {
                            @for col in &collections {
                                @let active_class = if Some(col.clone()) == active_collection { "sidebar-item active" } else { "sidebar-item" };
                                a class=(active_class) href=(format!("/?collection={}", col)) {
                                    span { "📂 " (col) }
                                }
                            }
                        }
                        
                        form class="new-col-form" method="POST" action="/collections" {
                            input type="text" name="name" placeholder="Nova coleção..." required;
                            button type="submit" { "+" }
                        }
                    }
                    
                    // Main Area
                    div class="main-area" {
                        div class="header" {
                            div {
                                @if let Some(ref col) = active_collection {
                                    h1 { "Coleção ativa: " span style="color: var(--primary)" { (col) } }
                                } @else {
                                    h1 { "Nenhuma coleção selecionada" }
                                }
                            }
                            div class="header-actions" {
                                a class="logout-btn" href="/docs" style="color: var(--primary); border-color: var(--primary); margin-right: 10px;" { "📖 API Docs" }
                                a class="logout-btn" href="/logout" { "Sair" }
                            }
                        }
                        
                        div class="content-body" {
                            @if let Some(err) = error_msg {
                                div class="error-msg" style="margin-bottom: 20px; border: 1px solid var(--danger); padding: 12px; border-radius: 6px;" {
                                    strong { "Erro: " } (err)
                                }
                            }
                            
                            @if let Some(ref col) = active_collection {
                                // Form to Create Document
                                div class="create-doc-card" {
                                    h3 { "➕ Adicionar Novo Documento" }
                                    form class="inline-fields" method="POST" action=(format!("/collections/{}/documents", col)) {
                                        div class="input-group" {
                                            label { "Document ID" }
                                            input type="text" name="doc_id" placeholder="Ex: user_123" required;
                                        }
                                        div class="input-group" {
                                            label { "JSON Data" }
                                            textarea class="textarea-json" name="json" style="height: 100px;" required {
                                                "{\n  \"nome\": \"Exemplo\",\n  \"ativo\": true\n}"
                                            }
                                        }
                                        button class="btn-primary" type="submit" style="width: auto; align-self: flex-start; padding: 8px 16px;" { "Criar Documento" }
                                    }
                                }
                                
                                // Documents Grid
                                @if documents.is_empty() {
                                    div class="welcome-container" {
                                        p { "Nenhum documento encontrado nesta coleção." }
                                    }
                                } @else {
                                    div class="doc-grid" {
                                        @for (doc_id, doc) in &documents {
                                            @let pretty_json = serde_json::to_string_pretty(&doc).unwrap_or_default();
                                            div class="doc-card" {
                                                div class="doc-header" {
                                                    span class="doc-id" { "🆔 " (doc_id) }
                                                    form method="POST" action=(format!("/collections/{}/documents/{}/delete", col, doc_id)) {
                                                        button class="btn-icon" type="submit" title="Excluir documento" { "🗑️" }
                                                    }
                                                }
                                                form method="POST" action=(format!("/collections/{}/documents/{}/update", col, doc_id)) {
                                                    textarea class="textarea-json" name="json" { (pretty_json) }
                                                    div class="card-actions" {
                                                        button class="btn-save" type="submit" { "Salvar" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } @else {
                                div class="welcome-container" {
                                    h2 { "Bem-vindo ao Waterbase!" }
                                    p { "Selecione ou crie uma coleção na barra lateral esquerda para gerenciar os documentos do seu banco de dados." }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
