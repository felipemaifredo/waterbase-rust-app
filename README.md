# Waterbase 💧

Waterbase é um banco de dados NoSQL leve, orientado a documentos, desenvolvido em Rust. Ele combina armazenamento em memória com persistência automática no disco, oferecendo um painel administrativo web integrado (HUD) e uma API REST segura para integrações externas.

---

## ✨ Recursos

*   **Estrutura de Documentos:** Armazenamento em coleções (`Collections`) contendo documentos (`Documents`) representados como dados chave-valor JSON.
*   **Persistência em Disco:** Sincronização em tempo real das alterações em memória para arquivos locais `.json` dentro do diretório `/data`.
*   **Query Engine:** Motor de consultas flexível que suporta operadores de comparação (`==`, `!=`, `>`, `>=`, `<`, `<=`), ordenação (`asc`, `desc`) e limite de resultados.
*   **Waterbase HUD:** Painel Web moderno e responsivo (estilo Linear/Vercel) para gerenciar coleções e editar dados JSON diretamente no navegador.
*   **API REST Segura:** Endpoints HTTP protegidos por Bearer Token para que sistemas externos possam ler, gravar, atualizar e consultar dados.
*   **Segurança integrada:** Páginas de gerenciamento protegidas por sessão (cookie) e rotas de API protegidas por chave configurável.

---

## 🛠️ Tecnologias Utilizadas

*   [Rust](https://www.rust-lang.org/) (Edição 2024)
*   [Actix-web](https://actix.rs/) (Framework Web assíncrono para o HUD e API)
*   [Maud](https://maud.lambda.xyz/) (Templates HTML rápidos e tipados direto no código Rust)
*   [Serde](https://serde.rs/) & [serde_json](https://github.com/serde-rs/json) (Serialização e desserialização de JSON de alta performance)
*   [Dotenvy](https://github.com/allan2/dotenvy) (Carregamento de variáveis de ambiente)
*   [Uuid](https://github.com/uuid-rs/uuid) (Geração de identificadores únicos universais)

---

## 🚀 Configuração e Execução

### 1. Requisitos
Certifique-se de possuir a ferramenta de compilação do Rust configurada:
*   [Rust/Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (1.75+)

### 2. Configurando o Ambiente
Crie um arquivo chamado `.env` na raiz do projeto (use o `.env.example` como base):

```bash
cp .env.example .env
```

Edite o arquivo `.env` com suas credenciais:

```env
ADMIN_USER=admin
ADMIN_PASSWORD=admin_seguro_123
API_KEY=waterbase_secret_token_123
```

### 3. Executando o Servidor
Para compilar e iniciar o servidor administrativo e a API, execute:

```bash
cargo run
```

O servidor estará rodando em [http://localhost:8080](http://localhost:8080).

*   Acesse o endereço no seu navegador para entrar no **Waterbase HUD** utilizando o `ADMIN_USER` e `ADMIN_PASSWORD` configurados.

---

## 📖 Referência da API REST (`/api/v1`)

Todas as requisições à API exigem o cabeçalho HTTP:
`Authorization: Bearer <API_KEY>`

| Método | Endpoint | Descrição |
| :--- | :--- | :--- |
| **GET** | `/api/v1/collections` | Lista todas as coleções do banco de dados |
| **GET** | `/api/v1/{collection}` | Retorna todos os documentos de uma coleção (com o ID injetado no JSON) |
| **GET** | `/api/v1/{collection}/{document_id}` | Obtém o conteúdo JSON de um documento específico |
| **POST** | `/api/v1/{collection}/{document_id}` | Cria ou substitui um documento. (Envie `new_id` no lugar do `document_id` para gerar um UUID v4 automaticamente) |
| **PUT** | `/api/v1/{collection}/{document_id}` | Atualização parcial (realiza o merge de campos) no documento especificado |
| **DELETE** | `/api/v1/{collection}/{document_id}` | Exclui permanentemente um documento da memória e do disco |
| **POST** | `/api/v1/{collection}/query` | Executa filtros complexos, ordenação e limite de busca no banco |

### Exemplo de Consulta (Query POST)

**Endpoint:** `/api/v1/users/query`  
**Corpo da Requisição (JSON):**
```json
{
  "where": [
    { "field": "idade", "op": ">", "value": 22 }
  ],
  "order_by": { "field": "idade", "direction": "desc" },
  "limit": 5
}
```

---

## 🧪 Testes Unitários

O projeto possui cobertura de testes para validar o motor de consultas, o armazenamento físico no disco, fluxo de CRUD e integridade da serialização customizada.

Para executar os testes:

```bash
cargo test
```
