# 🔑 Plano de Implementação: Módulo de Autenticação e Perfis (Register, Login com Cookie e Revalidate com Rotação)

Este plano detalha a criação de um módulo de autenticação e gerenciamento de perfis de usuário com base em duas coleções distintas (`authentication` e `users`), com controle de sessão por cookie HTTP e um endpoint para revalidação de sessão (`revalidate`) com rotação automática de tokens (session rolling).

---

## 🔍 Visão Geral do Design

### 1. Separação de Dados (Coleções)
As informações dos usuários serão distribuídas em duas coleções:
* **Coleção `authentication`:** Guardará apenas as credenciais (`email` e `password_hash`).
* **Coleção `users`:** Guardará os dados do perfil, como `name`, `document` (CPF/RG), `address`, etc.

Ambas as coleções estarão vinculadas pelo mesmo ID de documento (ex: `username` ou `id` do usuário).

### 2. Controle de Sessão e Rotação de Cookies
Para gerenciar o estado da sessão de forma segura:
* O servidor manterá um mapa em memória (`sessions: RwLock<HashMap<String, String>>` mapeando um `session_token` -> `user_id`).
* O token é transmitido por meio do cookie HTTP `auth_session`.
* **Rotação de Cookie (Token Rotation):** A cada chamada de revalidação com sucesso, o servidor invalidará o token de sessão antigo, gerará um novo token de sessão associado ao mesmo `user_id` e retornará um novo cookie `auth_session` para o cliente.

---

## 📞 Endpoints de API

### A. Registro (`POST /api/v1/auth/register`)
Responsável por criar o usuário nas duas coleções:
* **Payload JSON esperado:**
  ```json
  {
    "id": "joao123",
    "email": "joao@example.com",
    "password": "senha_secreta",
    "name": "João Silva",
    "document": "123.456.789-00",
    "address": "Rua das Flores, 123"
  }
  ```
* **Fluxo:**
  1. Verifica se já existe o `id` em `authentication` ou `users`. Se sim, retorna erro.
  2. Gera o hash `bcrypt` da senha.
  3. Cria o documento em `authentication` com `{ "email": "...", "password_hash": "..." }`.
  4. Cria o documento em `users` com `{ "name": "...", "document": "...", "address": "..." }`.
  5. Retorna `201 Created`.

### B. Login (`POST /api/v1/auth/login`)
Valida as credenciais, inicia a sessão e define o cookie HTTP:
* **Payload JSON esperado:**
  ```json
  {
    "id": "joao123",
    "password": "senha_secreta"
  }
  ```
* **Headers de resposta (Sucesso):**
  ```http
  Set-Cookie: auth_session=123e4567-e89b-12d3-a456-426614174000; HttpOnly; Secure; Path=/; Max-Age=28800; SameSite=Strict
  ```
  *(Nota: A flag `Secure` será habilitada apenas em produção, se `APP_ENV=production`)*
* **Payload JSON retornado (Sucesso):**
  ```json
  {
    "authenticated": true,
    "id": "joao123",
    "email": "joao@example.com",
    "profile": {
      "name": "João Silva",
      "document": "123.456.789-00",
      "address": "Rua das Flores, 123"
    }
  }
  ```

### C. Revalidação (`GET /api/v1/auth/revalidate`)
Verifica se o cliente possui uma sessão ativa baseada no cookie recebido, rotaciona a sessão e retorna o usuário atualizado:
* **Headers da requisição:**
  ```http
  Cookie: auth_session=123e4567-e89b-12d3-a456-426614174000
  ```
* **Fluxo:**
  1. Lê o cookie `auth_session` da requisição HTTP.
  2. Verifica se o token antigo existe no mapa de sessões.
  3. Se válido:
     - Recupera o `id` do usuário associado.
     - Remove o token antigo do mapa de sessões.
     - Gera um **novo** `session_token` (UUID).
     - Insere o novo token no mapa associado ao mesmo `id`.
     - Define o novo cookie `auth_session` com o novo token nos cabeçalhos de resposta.
     - Busca os perfis nas coleções `authentication` e `users` e retorna os dados com `200 OK`.
  4. Se for inválido ou ausente, retorna `401 Unauthorized`.
* **Headers de resposta (Sucesso):**
  ```http
  Set-Cookie: auth_session=987f6543-e21b-32d3-b456-426614174999; HttpOnly; Secure; Path=/; Max-Age=28800; SameSite=Strict
  ```
* **Payload JSON retornado (Sucesso):**
  ```json
  {
    "authenticated": true,
    "id": "joao123",
    "email": "joao@example.com",
    "profile": {
      "name": "João Silva",
      "document": "123.456.789-00",
      "address": "Rua das Flores, 123"
    }
  }
  ```
* **Payload JSON retornado (Erro/Não autenticado):**
  ```json
  {
    "authenticated": false,
    "error": "Sessão inválida ou expirada"
  }
  ```

---

## 🛠️ Alterações Propostas

### 📦 [Cargo.toml](file:///e:/waterbase-rust-app/Cargo.toml)
Adicionar dependência de criptografia:
* `bcrypt = "0.15"`

### 📄 [src/lib.rs](file:///e:/waterbase-rust-app/src/lib.rs)
* Adicionar o campo `sessions: RwLock<HashMap<String, String>>` na struct `SharedDb`.
* Inicializar o mapa vazio em `SharedDb::from_database`.

### 📄 [src/handlers/api.rs](file:///e:/waterbase-rust-app/src/handlers/api.rs)
1. **Implementar `api_auth_register`:** (`POST /api/v1/auth/register`).
2. **Implementar `api_auth_login`:** (`POST /api/v1/auth/login`). Cria a sessão no `db.sessions` e define o cookie `auth_session`.
3. **Implementar `api_auth_revalidate`:** (`GET /api/v1/auth/revalidate`). Lê o cookie, busca no `db.sessions`, invalida o token antigo, gera o novo token, atualiza o mapa e retorna o novo cookie `auth_session` e os dados do usuário.
4. **Filtro em `authentication`:** Omitir `password_hash` em leituras padrão na coleção `authentication`.

### 📄 [src/main.rs](file:///e:/waterbase-rust-app/src/main.rs)
* Registrar as novas rotas da API:
  ```rust
  .route("/auth/register", web::post().to(handlers::api::api_auth_register))
  .route("/auth/login", web::post().to(handlers::api::api_auth_login))
  .route("/auth/revalidate", web::get().to(handlers::api::api_auth_revalidate))
  ```

---

## 🚦 Plano de Verificação

### Testes Automatizados
* Testar fluxos de registro e login com cookie em `src/handlers/api.rs`.
* Testar chamada de `revalidate` enviando cookie válido (deve retornar sucesso, perfil, e um novo cookie diferente no cabeçalho).
* Testar chamada de `revalidate` sem cookie ou com cookie inválido (deve retornar 401).

### Testes Manuais
1. Registrar um usuário e efetuar o login usando uma ferramenta de testes HTTP (Postman/cURL).
2. Verificar que a resposta do login incluiu o cabeçalho `Set-Cookie` com o token de sessão `auth_session` (ex: `token1`).
3. Chamar `GET /api/v1/auth/revalidate` enviando o cookie recebido (`token1`) e validar o retorno do perfil + novo cabeçalho `Set-Cookie` com um valor diferente (ex: `token2`).
4. Tentar chamar `revalidate` novamente com o token antigo (`token1`) e validar que a chamada falha devido à rotação da sessão.
5. Chamar `revalidate` com o token novo (`token2`) e validar o sucesso.
