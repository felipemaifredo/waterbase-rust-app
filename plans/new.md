1. Vulnerabilidade Crítica de Bypass de Autenticação (Cookie Estático)
No arquivo 

src/handlers/hud.rs
, a autenticação de sessão do HUD é baseada em uma constante estática de texto:

rust
pub const SESSION_COOKIE_NAME: &str = "waterbase_session";
pub const SESSION_COOKIE_VALUE: &str = "authenticated_admin";
Qualquer atacante que saiba o nome e o valor constante do cookie pode criar manualmente o cookie waterbase_session=authenticated_admin no navegador e ignorar completamente a tela de login, ganhando acesso de escrita total às coleções e aos documentos.

Recomendação: A sessão deve utilizar identificadores de sessão aleatórios gerados em memória no login (armazenados temporariamente no SharedDb::sessions) ou cookies criptograficamente assinados/cifrados.