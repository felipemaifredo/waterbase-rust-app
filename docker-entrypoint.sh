#!/bin/sh
# Garante que o diretório de dados existe e pertence ao usuário da aplicação
# Necessário no Render: o Persistent Disk é montado como root após o build
if [ -n "$DATABASE_PATH" ]; then
  mkdir -p "$DATABASE_PATH"
  chown -R waterbase:waterbase "$DATABASE_PATH" 2>/dev/null || true
fi

# Inicia a aplicação como usuário não-root
exec gosu waterbase /app/waterbase-rust-app
