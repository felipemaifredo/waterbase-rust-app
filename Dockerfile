# Estágio de build
FROM rust:1.75-slim AS builder

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

# Diretório padrão para o banco de dados persistente
VOLUME ["/app/data"]

ENV HOST=0.0.0.0
ENV PORT=8080
ENV DATABASE_PATH=/app/data
ENV APP_ENV=production

EXPOSE 8080

CMD ["./waterbase-rust-app"]
