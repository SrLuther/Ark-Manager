# Changelog — Ark Manager

Todas as mudanças notáveis neste projeto serão documentadas aqui.  
Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/).

---

## [Não lançado]

---

## [1.1.0] — 01/06/2026

### Adicionado
- Sistema de autoupdate via `tauri-plugin-updater` — verifica novas versões no GitHub Releases ao iniciar e a cada 4 horas; exibe banner flutuante com progresso de download e botão de reinicialização (`src/components/updater/UpdateNotifier.tsx`)
- Endpoint de update configurado em `tauri.conf.json` apontando para `latest.json` do GitHub Releases

### Alterado
- **Banco de dados migrado de MySQL/MariaDB para SQLite local** — zero configuração necessária; arquivo criado automaticamente em `%APPDATA%\com.arkmanager.app\ark-manager.db` na primeira execução
- `db/connection.rs` reescrito: pool SQLite via `sqlx` com `SqlitePoolOptions`, opção `create_if_missing`, `foreign_keys` habilitado
- `db/migrations.rs` reescrito: sintaxe SQLite (`INTEGER PRIMARY KEY AUTOINCREMENT`, `TEXT`, `REAL`, índices em instruções `CREATE INDEX` separadas, sem `ENGINE=InnoDB`, `ENUM`, `ON UPDATE`)
- `db/mod.rs`: `initialize()` sem parâmetros — não depende mais de configuração prévia do usuário
- `lib.rs`: removido `load_db_config`; inicialização do banco direto no startup sem retry externo
- `commands/database.rs`: substituído por stub simples — retorna o caminho do banco em uso
- Todas as queries SQL: `NOW()` → `CURRENT_TIMESTAMP`, `INSERT IGNORE` → `INSERT OR IGNORE`, `ON DUPLICATE KEY UPDATE` → `INSERT OR REPLACE`, `last_insert_id()` → `last_insert_rowid()`
- `Settings.tsx`: seção "Banco de Dados (MySQL / MariaDB)" removida — não há mais configuração de banco para o usuário
- `utils/tauri.ts`: exports `getDatabaseUrl`, `saveDatabaseUrl`, `testDatabaseConnection` removidos; mensagens de erro MySQL-específicas removidas de `traduzirErro()`

### Corrigido
- Console do Windows suprimido em release via `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
- Erros internos do Tauri/Rust traduzidos para PT-BR via wrapper `invoke()` em `utils/tauri.ts`
- Botões de seleção de pasta (`pickDir`) em Settings, ServerManager e InstallServerDialog agora exibem toast de erro em vez de falhar silenciosamente

---

## [1.0.0] — 01/06/2026

### Adicionado

#### Backend Rust
- Estrutura completa de módulos: `commands/`, `db/`, `models/`, `services/`, `utils/`
- Pool MySQL com retry exponencial (`db/connection.rs`)
- 7 migrations idempotentes via tabela `_migrations` (v1–v7): schema base, colunas extras, agentes, sync, eventos sazonais
- **12 serviços core**: SteamCMD, instalação de servidor, config INI (UTF-16 LE com BOM), gerador de `RunServer.cmd`, gerenciador de processos, watcher de log, RCON (Source RCON TCP), backup, scheduler (cron), rede, hardware (`sysinfo`), parser INI
- **17 arquivos de commands Tauri**: server, install, config, rcon, logs, mods, cluster, backup, scheduler, hardware, import, agent, sync, discord, database, seasonal_events
- `setup_database` — cria banco MySQL do zero com `utf8mb4` e executa todas as migrations
- Agente de rede local: servidor HTTP/WebSocket (`axum`, porta 45678), descoberta UDP broadcast/mDNS, pareamento por código 6 dígitos
- Motor de sincronização bidirecional: watcher em tempo real (`notify`) + reconciliação periódica por `mtime+size`, resolução `last-write-wins`, limite de 5 pastas
- Serviço de notificações Discord via Webhook (`services/discord.rs`)
- Eventos sazonais: `event_scheduler.rs` (ciclo de vida completo), `event_config_swapper.rs` (backup/restore INI), broadcasts RCON automáticos (5 min antes do início/fim)
- `commands/seasonal_events.rs`: criar, editar, cancelar, force_start, force_end, status

#### Frontend React
- Layout com sidebar colapsável, 13 rotas configuradas
- **13 páginas**: Dashboard, ServerManager, ConfigEditor, RconConsole, LogsConsole, ModManager, ClusterManager, Backups, Scheduler, Settings, Monitoring, SyncManager, SeasonalEvents
- **Settings** completamente reconstruída: campos separados para MySQL (host/porta/usuário/senha/banco), botões "Criar banco", "Importar existente", "Testar conexão", "Salvar URL"; seção SteamCMD com instalação e log inline; Discord Webhook com toggle por evento
- Componentes de eventos sazonais: `EventCard` (countdown em tempo real), `EventRatesForm`, `EventServerSelector`, `EventStatusBadge`
- Componentes de sincronização: `SyncFolderCard`, `PeerDiscoveryDialog`, `SyncStatusBadge`
- Stores Zustand: `serverStore`, `uiStore`, `installStore`, `rconStore`, `syncStore`, `agentStore`, `seasonalEventStore`
- `utils/tauri.ts` com wrapper `invoke()` traduzindo erros para PT-BR e exports para todos os 60+ comandos
- `ErrorBoundary` com exibição de stack trace e botão de recarga
- Banner de erro de banco de dados (`DbErrorBanner`) com link direto para Settings
- Logo 128×128 centralizada na sidebar com `object-contain`
- Pickers de pasta nativos via `@tauri-apps/plugin-dialog` em ServerManager e Settings

#### Infraestrutura
- Tauri 2 com plugins: `opener`, `dialog`, `shell`, `process`, `updater`
- Tailwind CSS 3 com paleta `ark` (sky) e `surface` (slate)
- Build de produção gerado: instaladores NSIS e MSI

---
