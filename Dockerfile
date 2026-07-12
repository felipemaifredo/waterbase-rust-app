# Estágio de build
FROM rust:1.88-slim AS builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Estágio de execução
FROM debian:bookworm-slim

# Instala certificados SSL e gosu (para drop de privilégios no entrypoint)
RUN apt-get update && apt-get install -y ca-certificates gosu && rm -rf /var/lib/apt/lists/*

# Cria usuário não-root para maior segurança
RUN useradd -ms /bin/bash waterbase

WORKDIR /app

# Copia o executável gerado e o entrypoint
COPY --from=builder /usr/src/app/target/release/waterbase-rust-app /app/waterbase-rust-app
COPY docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

# ---------------------------------------------------------------------------
# Labels de recomendação de recursos
# ---------------------------------------------------------------------------
LABEL com.waterbase.resources.memory.limit="256m" \
      com.waterbase.resources.memory.swap="512m" \
      com.waterbase.resources.cpu.limit="0.5" \
      com.waterbase.resources.cpu.shares="512"

# Pré-cria o diretório de dados
RUN mkdir -p /app/data && chown -R waterbase:waterbase /app /app/data

# Diretório padrão para o banco de dados persistente
VOLUME ["/app/data"]

ENV HOST=0.0.0.0
ENV PORT=8080
ENV DATABASE_PATH=/app/data
ENV APP_ENV=prod

EXPOSE 8080

# O entrypoint roda como root, corrige permissões no disco montado, depois dropa para waterbase
ENTRYPOINT ["/app/docker-entrypoint.sh"]
