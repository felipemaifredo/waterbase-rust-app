# 🚀 Guia de Deploy - Waterbase

Este documento contém as configurações recomendadas e instruções para colocar a aplicação Waterbase em produção de forma segura.

---

## 🔄 Local vs Produção — Como Conciliar

Todas as configurações de produção são controladas por variáveis de ambiente. Isso significa que o mesmo binário roda normalmente em local e se adapta automaticamente em produção — **sem precisar mudar código**.

| Configuração | Local (`.env`) | Produção (PaaS/Docker) |
|---|---|---|
| **Cookie Secure** | `APP_ENV=development` → flag desativada | `APP_ENV=production` → flag ativada |
| **CORS** | `ALLOWED_ORIGIN` ausente → libera qualquer origem | `ALLOWED_ORIGIN=https://seu-dominio.com` → restrito |
| **Health check** | Funciona igual nos dois ambientes | Funciona igual |
| **Docker** | Não usado — rode com `cargo run` | `docker compose up -d` |

### Arquivo `.env` para desenvolvimento local

Copie o [.env.example](file:///e:/waterbase-rust-app/.env.example) e ajuste:

```env
APP_ENV=development
HOST=127.0.0.1
PORT=8080
DATABASE_PATH=data
ADMIN_USER=admin
ADMIN_PASSWORD=admin
API_KEY=waterbase_secret_token_123
# ALLOWED_ORIGIN não precisa ser definido em local
```

Em produção, as mesmas variáveis são configuradas no painel da PaaS com valores fortes — o código lê via `std::env::var()` e se adapta sozinho.

---

## ⚙️ Variáveis de Ambiente Necessárias (Produção)

Configure essas variáveis no painel administrativo da sua plataforma de hospedagem (Railway, Render, Fly.io, etc.):

| Variável | Valor Recomendado / Descrição |
| :--- | :--- |
| `PORT` | Porta onde o app vai rodar (injetada automaticamente pelas PaaS, ex: `8080`) |
| `HOST` | `0.0.0.0` |
| `APP_ENV` | `production` |
| `ADMIN_USER` | Nome de usuário forte para acessar o HUD (evite `admin`) |
| `ADMIN_PASSWORD` | Senha forte para o painel administrativo |
| `API_KEY` | Bearer Token forte para requisições na API REST |
| `DATABASE_PATH` | Caminho do volume persistente (ex: `/data` ou `/mnt/volume/data`) |
| `ALLOWED_ORIGIN` | Origem permitida para CORS (ex: `https://seu-dominio.com`) |

> **Segurança:** Nunca comite o arquivo `.env` no repositório. Use `.env.example` como referência pública.

---

## 🐳 Docker

### Executando com Docker

```bash
# Build da imagem
docker build -t waterbase .

# Rodando o container com volume persistente e variáveis de ambiente
docker run -d \
  --name waterbase \
  -p 8080:8080 \
  -v waterbase_data:/app/data \
  -e APP_ENV=production \
  -e ADMIN_USER=seu_usuario \
  -e ADMIN_PASSWORD=sua_senha_forte \
  -e API_KEY=seu_token_secreto \
  waterbase
```

---

### Docker Compose (Recomendado para Self-Hosting)

Crie um arquivo `docker-compose.yml` na raiz do projeto:

```yaml
version: "3.9"

services:
  waterbase:
    build: .
    container_name: waterbase
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - APP_ENV=production
      - HOST=0.0.0.0
      - PORT=8080
      - DATABASE_PATH=/app/data
      - ADMIN_USER=${ADMIN_USER}
      - ADMIN_PASSWORD=${ADMIN_PASSWORD}
      - API_KEY=${API_KEY}
      - ALLOWED_ORIGIN=${ALLOWED_ORIGIN}
    volumes:
      - waterbase_data:/app/data

volumes:
  waterbase_data:
```

Suba o serviço com:
```bash
docker compose up -d
```



## 📦 Compilação e Execução (Sem Docker)

Para rodar diretamente no servidor, compile com a flag `--release` para habilitar as otimizações do Rust.

1. **Compilar:**
   ```bash
   cargo build --release
   ```
2. **Iniciar o Servidor:**
   ```bash
   ./target/release/waterbase-rust-app
   ```

Para manter o processo rodando em background, use `systemd`, `supervisord`, ou um gerenciador de processos equivalente.

---

## ☁️ Plataformas PaaS Recomendadas

| Plataforma | Notas |
| :--- | :--- |
| **[Railway](https://railway.app)** | Detecta Dockerfile automaticamente. Configure as variáveis de ambiente no painel e associe um volume persistente a `/app/data`. |
| **[Render](https://render.com)** | Use "Web Service" com Docker. O disco persistente deve ser montado em `/app/data`. |
| **[Fly.io](https://fly.io)** | Use `fly launch` + `fly volumes create`. Monte o volume em `/app/data` no `fly.toml`. |

### Checklist pré-deploy

- [ ] `APP_ENV=production` configurado
- [ ] `ADMIN_PASSWORD` e `API_KEY` são valores fortes e únicos
- [ ] Cookie `secure(true)` habilitado
- [ ] CORS restrito à origem correta via `ALLOWED_ORIGIN`
- [ ] Volume persistente montado em `DATABASE_PATH`
- [ ] Health check `/health` respondendo corretamente
- [ ] `.env` não está no repositório (`.gitignore` configurado)
