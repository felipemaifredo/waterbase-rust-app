# 🛡️ Plano de Implementação: Modo Criptografado (Data-at-Rest Encryption)

Este plano descreve a adição de um modo opcional de criptografia de banco de dados baseado em uma variável de ambiente (`DATABASE_ENCRYPTION_KEY`). Se ativado, todos os documentos serão salvos de forma cifrada no disco usando AES-256-GCM.

---

## 🔍 Visão Geral do Design

### 1. Formato de Armazenamento Criptografado
Os documentos do banco de dados (atualmente serializados em MessagePack pelo `rmp-serde`) passarão a ter o seguinte formato em disco quando a criptografia estiver ativa:

```
| 'W' | 'E' | 'N' | 'C' | Nonce / IV (12 bytes) | Ciphertext + Tag (n bytes) |
```
* **Header Mágico (`b"WENC"`):** 4 bytes para autodetectar de forma confiável que o arquivo está criptografado.
* **Nonce (12 bytes):** Vetor de inicialização gerado de forma aleatória e única para cada gravação (usando o `OsRng` do sistema).
* **Ciphertext:** Bytes do MessagePack criptografados com AES-256-GCM (inclui a tag de autenticação de 16 bytes para integridade dos dados).

### 2. Compatibilidade e Migração
Para não quebrar bancos de dados existentes:
* **Leitura (Load):**
  * Se o arquivo iniciar com os bytes `b"WENC"`, o sistema exige a chave de criptografia (`DATABASE_ENCRYPTION_KEY`). Se a chave estiver ausente, a inicialização falha. Se a chave estiver presente, descriptografa e desserializa.
  * Se o arquivo **não** iniciar com `b"WENC"`, o sistema assume que o arquivo é plaintext (retrocompatibilidade) e realiza a desserialização normal diretamente.
* **Escrita (Save):**
  * Se a chave estiver ativa, salva criptografado.
  * Se a chave não estiver ativa, salva em texto puro.

---

## 🛠️ Alterações Propostas

### 📦 [Cargo.toml](file:///e:/waterbase-rust-app/Cargo.toml)
Adicionar dependências de criptografia:
* `aes-gcm = "0.10"` (para cifragem AES-256-GCM).
* `sha2 = "0.10"` (para fazer a derivação da chave a partir de qualquer frase secreta informada).

### 📄 [src/lib.rs](file:///e:/waterbase-rust-app/src/lib.rs)
1. **Adicionar estruturas de suporte:**
   * Modificar `Database` e `SharedDb` para incluir o campo `encryption_key: Option<[u8; 32]>`.
2. **Métodos Auxiliares de Criptografia:**
   * Implementar `encrypt_data(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String>`
   * Implementar `decrypt_data(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String>`
3. **Carregamento (`new_with_storage`):**
   * Ajustar a leitura para identificar o cabeçalho `b"WENC"`, descriptografar se necessário, e repassar os bytes corretos para a desserialização do `rmp_serde`.
4. **Sincronismo (`sync_document`):**
   * Ajustar o método de escrita de `Database` e `SharedDb` para criptografar os dados antes de gravar em arquivo caso `encryption_key` seja `Some`.

### 📄 [src/main.rs](file:///e:/waterbase-rust-app/src/main.rs)
1. **Leitura da Variável de Ambiente:**
   * Adicionar a leitura de `DATABASE_ENCRYPTION_KEY`.
   * Se presente e não vazia, aplicar SHA-256 na string fornecida para obter uma chave de 32 bytes segura.
   * Repassar essa chave opcional para `Database::new_with_storage`.

---

## 🚦 Plano de Verificação

### Testes Automatizados
* Adicionar testes unitários em `src/lib.rs` para testar os seguintes cenários:
  1. Escrita e leitura com criptografia ativa.
  2. Leitura de arquivos plaintext legados quando a criptografia é ativada posteriormente (migração transparente).
  3. Tentativa de ler arquivo criptografado sem chave de criptografia ativa (deve falhar de forma limpa com erro amigável).
  4. Criptografia e descriptografia de payloads vazios ou inválidos.

### Testes Manuais
1. Iniciar o app sem `DATABASE_ENCRYPTION_KEY` e criar dados de teste.
2. Ativar `DATABASE_ENCRYPTION_KEY` com um valor secreto, reiniciar o app e validar que ele leu com sucesso os dados criados anteriormente (modo plaintext legado).
3. Criar novos dados com o modo criptografado ativo.
4. Validar inspecionando os arquivos `.bin` gerados dentro de `data/` que eles começam com `WENC` e que o conteúdo não é legível como MessagePack puro.
5. Reiniciar o app sem a chave secreta e garantir que ele exibe um erro amigável impedindo a inicialização devido à presença de coleções criptografadas inacessíveis.
