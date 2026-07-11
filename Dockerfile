# Estágio de build
FROM rust:1.88-slim AS builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Estágio de execução
FROM debian:bookworm-slim

# Instala certificados SSL e bibliotecas úteis (caso necessário)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copia o executável gerado
COPY --from=builder /usr/src/app/target/release/waterbase-rust-app /app/waterbase-rust-app

# ---------------------------------------------------------------------------
# Labels de recomendação de recursos
# Os limites reais de CPU e RAM são aplicados em tempo de execução via:
#   docker run --memory="256m" --memory-swap="512m" --cpus="0.5" ...
# Ou configurados no docker-compose.yml em deploy.resources.limits
# ---------------------------------------------------------------------------
LABEL com.waterbase.resources.memory.limit="256m" \
      com.waterbase.resources.memory.swap="512m" \
      com.waterbase.resources.cpu.limit="0.5" \
      com.waterbase.resources.cpu.shares="512"

# Diretório padrão para o banco de dados persistente
VOLUME ["/app/data"]

ENV HOST=0.0.0.0
ENV PORT=8080
ENV DATABASE_PATH=/app/data
ENV APP_ENV=production

EXPOSE 8080

CMD ["./waterbase-rust-app"]
