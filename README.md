<div align="center">
  <img src="src/assets/logo/logo-128.png" alt="Ark Manager Logo" width="96" />
  <h1>Ark Manager</h1>
  <p><strong>Gerenciador completo de servidores ARK: Survival Evolved</strong></p>

  ![Versão](https://img.shields.io/badge/versão-1.0.0-blue?style=flat-square)
  ![Plataforma](https://img.shields.io/badge/plataforma-Windows-blue?style=flat-square&logo=windows)
  ![Tauri](https://img.shields.io/badge/Tauri-2-24C8D8?style=flat-square&logo=tauri)
  ![React](https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react)
  ![Rust](https://img.shields.io/badge/Rust-1.96-orange?style=flat-square&logo=rust)
  ![Licença](https://img.shields.io/badge/licença-MIT-green?style=flat-square)
</div>

---

## O que é

O **Ark Manager** é um aplicativo desktop para Windows que centraliza o gerenciamento completo de servidores **ARK: Survival Evolved (ASE)**. Substitui scripts `.bat`, ferramentas externas de RCON e configuração manual de arquivos `.ini` por uma interface gráfica moderna e intuitiva.

---

## Funcionalidades

| Módulo | O que faz |
|---|---|
| **Dashboard** | Visão geral com status, CPU/RAM e ações rápidas para todos os servidores |
| **Servidores** | Cadastrar, editar, instalar (via SteamCMD), iniciar/parar/reiniciar |
| **Config Editor** | Editor visual de `GameUserSettings.ini` e `Game.ini` com encoding correto (UTF-16 LE BOM) |
| **RCON Console** | Terminal RCON com histórico e comandos rápidos (saveworld, destroywilddinos…) |
| **Logs** | Visualização em tempo real do `ShooterGame.log` com filtros e cores por nível |
| **Mods** | Gerenciar mods do Steam Workshop por servidor |
| **Cluster** | Configurar Cross-ARK cluster com `ClusterId` e pasta compartilhada |
| **Backups** | Backup automático/manual do `SavedArks` com restauração por timestamp |
| **Agendador** | Tarefas cron: restart, backup, RCON, saveworld, destroywilddinos, update |
| **Monitoramento** | CPU, RAM e disco por servidor/processo em tempo real |
| **Sincronização** | Sync bidirecional de pastas entre máquinas na rede local via agente HTTP/WS |
| **Eventos Sazonais** | Eventos temporários com taxas especiais, broadcasts automáticos e backup/restore de INIs |
| **Configurações** | MySQL, SteamCMD, backups, Discord Webhook, intervalo de sync |

---

## Requisitos

- **Windows 10/11** (x64)
- **MySQL 8** ou **MariaDB 10.6+**
- **SteamCMD** (instalável pelo próprio app)
- WebView2 Runtime (incluído no instalador)

---

## Instalação

1. Baixe o instalador na [página de releases](https://github.com/SrLuther/Ark-Manager/releases/latest)
2. Execute o `.exe` e siga o assistente
3. Na primeira execução, acesse **Configurações → Banco de Dados**
4. Preencha os dados do MySQL e clique em **Criar banco** para configurar o schema automaticamente
5. Acesse **Configurações → SteamCMD** e clique em **Instalar** se ainda não tiver o SteamCMD

---

## Stack técnica

| Camada | Tecnologia |
|---|---|
| Interface | React 19 + TypeScript 5.6 + Tailwind CSS 3 |
| Desktop | Tauri 2 |
| Backend | Rust 1.96 + Tokio |
| Banco de dados | MySQL / MariaDB via SQLx 0.8 |
| Estado | Zustand 5 |
| Build | Vite 6 |

---

## Desenvolvimento

### Pré-requisitos

- Node.js 20+
- Rust 1.96+ (`rustup`)
- MySQL / MariaDB rodando localmente

### Configurar o banco

Crie um arquivo `%APPDATA%\com.arkmanager.app\database.json`:

```json
{
  "url": "mysql://root:senha@localhost:3306/ark_manager"
}
```

Ou use a tela de **Configurações** do app em modo de desenvolvimento.

### Rodar em modo dev

```bash
# Instalar dependências
npm install

# Iniciar em modo desenvolvimento (abre janela Tauri)
npm run tauri dev
```

### Build de produção

```bash
npm run tauri build
```

Os instaladores são gerados em `src-tauri/target/release/bundle/`.

---

## Autoupdate

O app verifica automaticamente novas versões no GitHub Releases ao iniciar e a cada 4 horas. Quando disponível, exibe um banner discreto no canto inferior direito com opção de instalar e reiniciar.

Para publicar uma atualização, crie uma release no GitHub com:
- O instalador `.nsis.zip` assinado
- O arquivo `latest.json` com metadados da versão

---

## Estrutura do projeto

```
Ark Manager/
├── src/                    # Frontend React
│   ├── components/         # Componentes reutilizáveis (UI, layout, eventos, sync…)
│   ├── pages/              # Páginas da aplicação (uma por rota)
│   ├── stores/             # Zustand stores por domínio
│   └── utils/              # Wrappers Tauri invoke + helpers
├── src-tauri/
│   └── src/
│       ├── commands/       # Handlers Tauri (bridge frontend ↔ backend)
│       ├── db/             # Pool MySQL + migrations
│       ├── models/         # Structs de domínio
│       └── services/       # Lógica de negócio (steamcmd, rcon, sync, eventos…)
└── PLANO.md                # Documento de arquitetura e roadmap detalhado
```

---

## Licença

MIT © 2026 SrLuther
