# 🚀 Guia de Deploy - Waterbase

Este documento contém a lista de modificações necessárias no código e as configurações recomendadas para colocar a aplicação Waterbase em produção.

---

## 📋 Checklist de Alterações no Código

### 1. Permitir Tráfego Externo (IP Binding)
Por padrão, o servidor escuta apenas no localhost (`127.0.0.1`). É necessário alterar para `0.0.0.0` para que ele responda a requisições de fora da máquina/container.

*   **Onde alterar:** [src/main.rs:L69](file:///Users/felipemaifredo/Dev/waterbase-rust-app/src/main.rs#L69)
*   **Como está:**
    ```rust
    .bind(("127.0.0.1", port))?
    ```
*   **Como deve ficar (carregando dinamicamente):**
    ```rust
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    // ...
    .bind((host.as_str(), port))?
    ```

---

### 2. Configurar o Caminho de Persistência (Diretório `/data`)
Se o deploy for em uma plataforma *stateless* (como Render livre, Railway ou Heroku), os dados serão perdidos a cada deploy ou reinicialização se não houver um volume persistente montado.

*   **Onde alterar:** [src/main.rs:L21](file:///Users/felipemaifredo/Dev/waterbase-rust-app/src/main.rs#L21)
*   **Como está:**
    ```rust
    Database::new_with_storage("data".to_string())
    ```
*   **Como deve ficar (parametrizável):**
    ```rust
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data".to_string());
    // ...
    Database::new_with_storage(db_path)
    ```
*   **Ação na Hospedagem:** Crie e monte um **Persistent Volume** apontando para o diretório configurado (ex: `/data` ou o valor de `DATABASE_PATH`).

---

### 3. Cookies de Sessão Seguros (HTTPS)
Se a aplicação rodar sob HTTPS em produção, é de extrema importância habilitar a flag `secure(true)` nos cookies para impedir que eles sejam interceptados.

*   **Onde alterar:** [src/handlers/hud.rs:L66-L69](file:///Users/felipemaifredo/Dev/waterbase-rust-app/src/handlers/hud.rs#L66-L69)
*   **Como está:**
    ```rust
    let cookie = Cookie::build(SESSION_COOKIE_NAME, SESSION_COOKIE_VALUE)
        .path("/")
        .http_only(true)
        .finish();
    ```
*   **Como deve ficar:**
    ```rust
    let is_prod = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()) == "production";

    let cookie = Cookie::build(SESSION_COOKIE_NAME, SESSION_COOKIE_VALUE)
        .path("/")
        .http_only(true)
        .secure(is_prod) // Habilita secure apenas em produção
        .finish();
    ```

---

## ⚙️ Variáveis de Ambiente Necessárias (Produção)

Configure essas variáveis de ambiente no painel administrativo da sua plataforma de hospedagem (Ex: Railway, Render, Fly.io, etc.):

| Variável | Valor Recomendado / Descrição |
| :--- | :--- |
| `PORT` | Porta onde o app vai rodar (injetada automaticamente pelas PaaS, ex: `8080`) |
| `HOST` | `0.0.0.0` |
| `APP_ENV` | `production` |
| `ADMIN_USER` | Nome de usuário forte para acessar o HUD (Evite usar `admin`) |
| `ADMIN_PASSWORD` | Senha forte para o painel administrativo |
| `API_KEY` | Bearer Token forte para requisições na API REST |
| `DATABASE_PATH` | Caminho do volume persistente (ex: `/data` ou `/mnt/volume/data`) |

---

## 📦 Compilação e Execução

Para rodar em ambiente de produção, compile utilizando a flag `--release` para habilitar as otimizações do compilador do Rust.

1.  **Compilar:**
    ```bash
    cargo build --release
    ```
2.  **Iniciar o Servidor:**
    ```bash
    ./target/release/waterbase-rust-app
    ```
