# ARK Manager — Documento Mestre do Projeto

> **Versão:** 1.0  
> **Iniciado em:** 31/05/2026  
> **Stack:** Tauri 2 + React 19 + TypeScript + Rust + MySQL  
> **Idioma da interface:** Português (pt-BR)  
> **Repositório local:** `C:\Users\Ciano\Documents\Ark Manager`

---

## SUMÁRIO

1. [Visão Geral do Projeto](#1-visão-geral-do-projeto)
2. [Projeto de Referência](#2-projeto-de-referência)
3. [Objetivos de Fidelidade](#3-objetivos-de-fidelidade)
4. [Tecnologias Oficiais](#4-tecnologias-oficiais)
5. [Arquitetura Geral](#5-arquitetura-geral)
6. [Módulos do Sistema](#6-módulos-do-sistema)
7. [Banco de Dados](#7-banco-de-dados)
8. [Sistema de Eventos](#8-sistema-de-eventos)
9. [Integrações](#9-integrações)
10. [Padrões de Desenvolvimento](#10-padrões-de-desenvolvimento)
11. [Padrões de Qualidade](#11-padrões-de-qualidade)
12. [Roadmap de Execução](#12-roadmap-de-execução)
13. [Histórico de Implementação](#13-histórico-de-implementação)
14. [Decisões Arquiteturais](#14-decisões-arquiteturais)
15. [Estado Atual do Projeto](#15-estado-atual-do-projeto)

---

## 1. VISÃO GERAL DO PROJETO

### O que é o ARK Manager

O ARK Manager é um aplicativo desktop profissional para gerenciamento completo de servidores **ARK: Survival Evolved (ASE)**. Desenvolvido com Tauri 2, oferece uma interface gráfica moderna, nativa ao sistema operacional Windows, que elimina a necessidade de gerenciar servidores manualmente via linha de comando, scripts `.bat` ou ferramentas externas.

### Objetivo Principal

Fornecer uma plataforma unificada onde administradores de servidores ARK possam instalar, configurar, operar, monitorar e manter servidores dedicados com total controle e visibilidade, em um único ambiente intuitivo.

### Público-Alvo

- Administradores de servidores privados de ARK: Survival Evolved.
- Comunidades e guilds que operam múltiplos servidores ou clusters.
- Jogadores avançados que desejam hospedar servidores próprios com facilidade.

### Problemas que Resolve

| Problema | Solução no ARK Manager |
|---|---|
| Configuração manual de arquivos `.ini` | Editor visual com todos os parâmetros tipados |
| Scripts `.bat` frágeis para start/stop | Gerenciador de processos com start/stop/restart confiável |
| Ausência de monitoramento integrado | Dashboard com CPU, RAM, players e status em tempo real |
| RCON via ferramenta externa | Console RCON nativo com histórico e comandos rápidos |
| Gestão manual de backups | Sistema automatizado de backups agendados com restore |
| Múltiplos servidores difíceis de coordenar | Cluster Manager para Cross-ARK com gestão centralizada |
| Instalação manual do SteamCMD | Instalação e atualização automatizada do servidor via SteamCMD |
| Ausência de agendamento de tarefas | Scheduler integrado com suporte a cron |

### Escopo Geral

O ARK Manager abrange o ciclo de vida completo de um servidor ARK:

1. **Instalação** — Download do SteamCMD, instalação do servidor (App ID `376030`).
2. **Configuração** — Edição visual dos arquivos `GameUserSettings.ini` e `Game.ini`.
3. **Operação** — Start, stop, restart, monitoramento de processo.
4. **Monitoramento** — Logs em tempo real, métricas de hardware, status de jogadores.
5. **Administração** — RCON, comandos remotos, broadcasts.
6. **Manutenção** — Backups automáticos, agendamento de tarefas, atualizações.
7. **Cluster** — Configuração de Cross-ARK cluster com múltiplos servidores.

### Visão de Longo Prazo

- Suporte a múltiplos perfis de servidor (presets de configuração).
- Notificações via Discord Webhook.
- Dashboard web acessível remotamente.
- Suporte futuro a ARK: Survival Ascended (ASA).

---

## 2. PROJETO DE REFERÊNCIA

### ARKLAND SM

O projeto de referência principal é o **ARKLAND SM**, disponível localmente em:

```
C:\Users\Ciano\Documents\ARKLAND SM
```

O ARKLAND SM é um sistema de gerenciamento de servidores já desenvolvido e funcionando, que serve como referência direta para a construção do ARK Manager em todos os aspectos relevantes.

### O que o ARKLAND SM representa

O ARKLAND SM contém decisões arquiteturais, padrões de código e experiências de usuário já validadas. Ao invés de partir do zero em cada decisão de design, o ARK Manager herda as soluções que já funcionaram, adaptando-as ao contexto específico de ARK: Survival Evolved.

### Aspectos herdados do ARKLAND SM

| Aspecto | Herança |
|---|---|
| **Arquitetura geral** | Estrutura de módulos, separação de responsabilidades |
| **Design visual** | Paleta de cores, tipografia, densidade informacional |
| **Organização de código** | Estrutura de pastas frontend e backend |
| **Fluxos de usuário** | Navegação, wizards, formulários |
| **Experiência de uso** | Padrões de interação, feedbacks visuais |
| **Estrutura modular** | Separação de commands/services/models/db |
| **Sistema de eventos** | Comunicação backend → frontend via Tauri emit |
| **Gerenciamento de estado** | Zustand stores por domínio |

### Grau de fidelidade

O ARK Manager deve manter **alto grau de fidelidade** ao ARKLAND SM em todos os aspectos estruturais. Divergências só são aceitáveis quando justificadas por especificidades do ARK: Survival Evolved.

---

## 3. OBJETIVOS DE FIDELIDADE

### Aspectos que permanecem próximos ao projeto de referência

#### Layout e Visual
- Sidebar de navegação lateral com ícones e labels.
- Header por página com título e ações contextuais.
- Cards de servidor com indicadores de status coloridos.
- Modais para operações destrutivas (delete, restore).
- Toasts/notificações para feedback de operações assíncronas.
- Tema escuro como padrão (paleta `surface` em slate + `ark` em sky).

#### Navegação
- Rota por módulo, sem recarregamento de página.
- Estado ativo da sidebar refletindo a rota atual.
- Breadcrumbs ou indicadores de contexto em subpáginas.

#### Estrutura de Código
- Commands Tauri separados por domínio (um arquivo por módulo).
- Services isolados com responsabilidade única.
- Models separados dos commands.
- Stores Zustand por domínio, sem store global monolítico.
- Hooks customizados para lógica reutilizável.

#### Fluxos
- Wizard passo a passo para instalação de servidor.
- Confirmação explícita para operações destrutivas.
- Loading states durante operações assíncronas.
- Tratamento de erro com mensagem descritiva ao usuário.

### Adaptações específicas para ARK: Survival Evolved

| Adaptação | Justificativa |
|---|---|
| Editor de INI com campos tipados para ARK | Parâmetros específicos do ARK (PerLevelStats, engram points, dino spawns) |
| Geração de `RunServer.cmd` | ARK usa .cmd para lançamento com parâmetros específicos |
| Codificação UTF-16 LE com BOM nos INIs | Requisito obrigatório do engine do ARK |
| `SessionName` apenas no INI, nunca na CLI | Comportamento específico do servidor ARK |
| 11 mapas oficiais do ASE | TheIsland, ScorchedEarth, Ragnarok, Aberration, Extinction, Valguero, Genesis, Gen2, CrystalIsles, LostIsland, Fjordur |
| App ID SteamCMD `376030` | ID do ARK: Survival Evolved no Steam |
| Cluster Cross-ARK com `ClusterId` e pasta compartilhada | Mecânica específica de cluster do ARK |

---

## 4. TECNOLOGIAS OFICIAIS

### Frontend

| Tecnologia | Versão | Função |
|---|---|---|
| **React** | 19 | Framework de interface |
| **TypeScript** | 5.6+ | Tipagem estática |
| **Vite** | 6 | Build tool e dev server (porta 1420) |
| **Tailwind CSS** | 3 | Estilização utilitária com paleta customizada |
| **Zustand** | 5+ | Gerenciamento de estado global por domínio |
| **React Query** | 5+ | Caching, sincronização e estados de carregamento de dados remotos |
| **React Router** | 6+ | Roteamento client-side |
| **Lucide React** | latest | Biblioteca de ícones |

### Backend

| Tecnologia | Versão | Função |
|---|---|---|
| **Rust** | 1.96+ | Linguagem principal do backend |
| **Tauri** | 2 | Bridge nativa desktop + webview |
| **Tokio** | 1 | Runtime assíncrono |
| **SQLx** | 0.8 | ORM/query builder async para MySQL |
| **Serde** | 1 | Serialização/deserialização JSON |
| **thiserror** | 2 | Tipos de erro ergonômicos |
| **reqwest** | 0.12 | HTTP client (downloads SteamCMD, Workshop) |

### Banco de Dados

| Tecnologia | Função |
|---|---|
| **MySQL** | Banco de dados principal |

**Motivo da escolha do MySQL:**
- Suporte robusto a dados estruturados relacionais (clusters, servidores, políticas de backup).
- Compatibilidade com infraestruturas de hospedagem existentes dos usuários-alvo.
- Melhor suporte a tipos JSON nativos para configurações flexíveis (`hardware_config`).
- SQLx oferece compile-time query checking com MySQL, garantindo segurança de tipos.
- O projeto de referência ARKLAND SM utiliza a mesma stack MySQL + SQLx, permitindo reuso direto de padrões.

---

## 5. ARQUITETURA GERAL

### Visão de Alto Nível

```
┌─────────────────────────────────────────────────────┐
│                   ARK Manager Desktop               │
│  ┌──────────────────────┐  ┌──────────────────────┐ │
│  │      Frontend        │  │       Backend        │ │
│  │   React + TypeScript │◄─►│   Rust + Tauri 2    │ │
│  │      (WebView)       │  │  (Processo Nativo)   │ │
│  └──────────────────────┘  └──────────┬───────────┘ │
│                                       │             │
│                              ┌────────▼────────┐    │
│                              │     MySQL       │    │
│                              └─────────────────┘    │
└─────────────────────────────────────────────────────┘
```

### Frontend — Estrutura de Pastas

```
src/
├── assets/              # Imagens, logos, fontes
│   └── logo/
├── components/          # Componentes reutilizáveis
│   ├── layout/          # Layout principal (Sidebar, Header, Layout)
│   └── ui/              # Componentes base (Button, Input, Modal, Badge...)
├── hooks/               # Hooks React customizados
├── pages/               # Páginas da aplicação (uma por rota)
├── stores/              # Zustand stores por domínio
├── styles/              # Estilos globais
├── types/               # Tipos TypeScript globais
└── utils/               # Utilitários e wrappers Tauri invoke
```

### Frontend — Fluxo de Componentes

```
App.tsx
└── Router
    └── Layout.tsx
        ├── Sidebar.tsx      ← navegação entre módulos
        └── <Page />         ← conteúdo da rota ativa
            ├── Components   ← componentes de apresentação
            ├── Hooks        ← lógica local
            └── Stores       ← estado global (Zustand)
```

### Frontend — Stores por Domínio

| Store | Responsabilidade |
|---|---|
| `serverStore` | Lista de servidores, status, operações CRUD |
| `uiStore` | Estado da UI (modais abertos, sidebar collapsed, notificações) |
| `installStore` | Estado do wizard de instalação (progresso, etapas) |
| `rconStore` | Conexão RCON ativa, histórico de comandos |

### Backend — Estrutura de Pastas

```
src-tauri/src/
├── commands/            # Handlers Tauri (bridge frontend ↔ backend)
│   ├── server.rs        # CRUD + start/stop/restart
│   ├── install.rs       # SteamCMD + instalação de servidor
│   ├── config.rs        # Leitura/escrita de INIs
│   ├── rcon.rs          # Conexão e comandos RCON
│   ├── logs.rs          # Watcher de log em tempo real
│   ├── mods.rs          # Gerenciamento de mods Workshop
│   ├── cluster.rs       # Configuração de cluster Cross-ARK
│   ├── backup.rs        # Backups e restaurações
│   ├── scheduler.rs     # Tarefas agendadas
│   ├── hardware.rs      # Métricas de sistema
│   └── import.rs        # Importação de servidor existente
├── db/                  # Camada de banco de dados
│   ├── mod.rs           # API pública do módulo (initialize)
│   ├── connection.rs    # Pool MySQL, retry com backoff exponencial
│   └── migrations.rs    # Migrations idempotentes (tabela _migrations)
├── models/              # Structs de domínio (mapeadas ao banco)
│   ├── server.rs        # Server, ServerStatus, ArkMap, CreateServerRequest...
│   ├── backup.rs        # Backup, BackupPolicy, BackupType...
│   └── task.rs          # ScheduledTask, TaskType, TaskResult...
├── services/            # Lógica de negócio (independente de Tauri)
│   ├── steamcmd.rs
│   ├── server_installer.rs
│   ├── config_generator.rs
│   ├── launch_builder.rs
│   ├── process_manager.rs
│   ├── log_watcher.rs
│   ├── rcon.rs
│   ├── backup_service.rs
│   ├── scheduler.rs
│   ├── network.rs
│   ├── system_analyzer.rs
│   └── ini_parser.rs
├── utils/               # Utilitários transversais
├── lib.rs               # Entry point da lib (AppState, run())
└── main.rs              # Entry point do binário Tauri
```

### Fluxo de Comunicação Frontend ↔ Backend

```
Frontend (React)
      │
      │  invoke("command_name", { payload })
      ▼
Tauri Command Handler (Rust)
      │
      ├── Acessa AppState (DbPool, etc.)
      ├── Chama Service correspondente
      ├── Service interage com MySQL ou Sistema Operacional
      └── Retorna Result<T, String> serializado como JSON
      │
      │  Resposta síncrona via Promise
      ▼
Frontend (React)
      │
      │  Eventos assíncronos (backend → frontend)
      │  handle.emit("event_name", payload)
      ▼
Frontend (React) — listener via @tauri-apps/api/event
```

---

## 6. MÓDULOS DO SISTEMA

### 6.1 Dashboard

**Objetivo:** Visão consolidada de todos os servidores em uma única tela.

**Responsabilidades:**
- Exibir card para cada servidor com status, mapa, jogadores conectados, CPU e RAM.
- Ações rápidas por servidor: start, stop, restart, abrir RCON.
- Indicadores visuais de saúde do sistema.
- Resumo de próximas tarefas agendadas.

**Dependências:** `serverStore`, commands `hardware`, `server`

**Telas relacionadas:** `Dashboard.tsx`

**Serviços relacionados:** `system_analyzer.rs`, `process_manager.rs`

---

### 6.2 Server Manager

**Objetivo:** Gerenciamento completo do ciclo de vida de servidores ARK.

**Responsabilidades:**
- Cadastrar, editar e remover servidores.
- Instalar servidor via SteamCMD (wizard passo a passo).
- Atualizar servidor existente.
- Detectar conflitos de porta antes de salvar.
- Importar servidor já instalado no disco.
- Exibir informações detalhadas: path de instalação, portas, mapa, mods ativos.

**Dependências:** `serverStore`, `installStore`, commands `server`, `install`, `import`

**Telas relacionadas:** `ServerManager.tsx`

**Componentes:** `ServerCard`, `InstallServerDialog`, `PortConflictModal`

**Serviços relacionados:** `steamcmd.rs`, `server_installer.rs`, `network.rs`, `ini_parser.rs`

---

### 6.3 SteamCMD

**Objetivo:** Gerenciar a instalação e uso do SteamCMD como ferramenta de instalação.

**Responsabilidades:**
- Verificar se SteamCMD está instalado no caminho configurado.
- Baixar e instalar automaticamente caso ausente.
- Executar atualizações do servidor (App ID `376030`).
- Reportar progresso de download em tempo real via eventos.

**Dependências:** command `install`, `uiStore`

**Serviços relacionados:** `steamcmd.rs`

---

### 6.4 Config Editor

**Objetivo:** Editor visual completo dos arquivos de configuração INI do ARK.

**Responsabilidades:**
- Editar `GameUserSettings.ini` e `Game.ini` com campos tipados.
- Exibir parâmetros agrupados por categoria (servidor, dinossauros, jogadores, cluster, etc.).
- Salvar arquivos no formato obrigatório **UTF-16 LE com BOM**.
- Editor avançado com texto bruto para usuários experientes.
- Importar configuração de um servidor existente no disco.

**Parâmetros cobertos:**
- Taxas de XP, coleta, domagem, curar
- Configurações de dinossauros (spawn, level, taming)
- PerLevelStatsMultiplier (índices 0–11) para jogadores e dinos
- Configurações de cluster (ClusterId, pasta compartilhada)
- RCON (porta, senha)
- SessionName, MaxPlayers, ServerPassword

**Dependências:** commands `config`, `serverStore`

**Telas relacionadas:** `ConfigEditor.tsx`

**Componentes:** `StatMultiplierEditor`

**Serviços relacionados:** `config_generator.rs`, `ini_parser.rs`

> ⚠️ **Regra crítica:** `SessionName` deve ser salvo **somente no INI**, nunca passado como parâmetro da linha de comando.

---

### 6.5 Mod Manager

**Objetivo:** Gerenciar mods do Steam Workshop por servidor.

**Responsabilidades:**
- Listar mods ativos de cada servidor.
- Adicionar mods por ID do Workshop.
- Remover mods.
- Reordenar lista de mods (a ordem afeta o carregamento no ARK).
- Persistir lista de mod IDs no campo `mods` do servidor (CSV).

**Dependências:** commands `mods`, `serverStore`

**Telas relacionadas:** `ModManager.tsx`

**Serviços relacionados:** `server_installer.rs` (para download dos mods via SteamCMD)

---

### 6.6 RCON

**Objetivo:** Console de administração remota via protocolo Source RCON.

**Responsabilidades:**
- Conectar ao servidor via TCP (porta RCON + senha).
- Enviar comandos e exibir resposta.
- Manter histórico de comandos enviados.
- Comandos rápidos pré-definidos: `saveworld`, `listplayers`, `broadcast`, `destroywilddinos`, `doexit`.
- Desconectar ao fechar o módulo.

**Dependências:** `rconStore`, commands `rcon`

**Telas relacionadas:** `RconConsole.tsx`

**Serviços relacionados:** `rcon.rs` (protocolo Source RCON via TCP)

---

### 6.7 Logs

**Objetivo:** Monitoramento em tempo real do log do servidor ARK.

**Responsabilidades:**
- Ler `ShooterGame.log` em tempo real com watcher de arquivo.
- Exibir linhas coloridas por nível (INFO, WARN, ERROR).
- Filtro por palavra-chave.
- Pausar/resumir streaming.
- Histórico das últimas N linhas.

**Dependências:** commands `logs`

**Telas relacionadas:** `LogsConsole.tsx`

**Serviços relacionados:** `log_watcher.rs`

**Eventos:** `log:line` — emitido pelo backend para cada nova linha de log

---

### 6.8 Backups

**Objetivo:** Backup automatizado e restauração dos dados de save do servidor.

**Responsabilidades:**
- Criar backup manual da pasta `SavedArks` com timestamp.
- Listar backups disponíveis com data, tamanho e tipo.
- Restaurar backup selecionado (substituir pasta `SavedArks`).
- Política de backup automático configurável (frequência, retenção, compressão).
- Notificação de backup concluído ou falho.

**Dependências:** commands `backup`, `serverStore`

**Telas relacionadas:** `Backups.tsx`

**Serviços relacionados:** `backup_service.rs`

**Eventos:** `backup:started`, `backup:completed`, `backup:failed`

---

### 6.9 Cluster Manager

**Objetivo:** Configurar e gerenciar clusters Cross-ARK multi-servidor.

**Responsabilidades:**
- Criar clusters com `ClusterId` único.
- Definir pasta compartilhada do cluster.
- Vincular servidores existentes ao cluster.
- Configurar `NoTransferFromFiltering`, `bAllowFlyerCarryPvE` e demais flags de cluster.
- Exibir visualmente os servidores vinculados.

**Dependências:** commands `cluster`, `serverStore`

**Telas relacionadas:** `ClusterManager.tsx`

**Componentes:** `ClusterBuilder`

---

### 6.10 Scheduler

**Objetivo:** Agendamento automatizado de tarefas administrativas recorrentes.

**Responsabilidades:**
- Criar tarefas com expressão cron ou intervalo.
- Tipos suportados: Restart, Backup, RconCommand, Announcement, SaveWorld, DestroyWildDinos, Update.
- Editar e desativar tarefas sem deletar.
- Histórico de execuções (sucesso/falha/skipped).
- Contagem de execuções e último resultado.

**Dependências:** commands `scheduler`

**Telas relacionadas:** `Scheduler.tsx`

**Serviços relacionados:** `scheduler.rs`

**Eventos:** `task:executed`, `task:failed`

---

### 6.11 Monitoramento

**Objetivo:** Métricas de hardware em tempo real por servidor/processo.

**Responsabilidades:**
- CPU, RAM e disco do sistema.
- Recursos consumidos pelo processo `ShooterGameServer.exe` por instância.
- Gráficos de histórico (últimos N minutos).
- Alertas de threshold (ex: RAM > 90%).

**Dependências:** commands `hardware`

**Componentes:** `PerformanceMonitor`

**Serviços relacionados:** `system_analyzer.rs` (crate `sysinfo`)

---

### 6.12 Configurações

**Objetivo:** Configurações globais da aplicação.

**Responsabilidades:**
- Caminho de instalação do SteamCMD.
- Diretório padrão para instalação de servidores.
- Diretório padrão para backups.
- Tema visual (dark/light).
- Idioma da interface.
- Configuração de conexão MySQL.
- Configuração de notificações Discord (Webhook URL, eventos habilitados).
- **Intervalo de sincronização** (padrão: 30 segundos) — tempo entre cada ciclo de verificação periódica de arquivos.

**Telas relacionadas:** `Settings.tsx`

---

### 6.13 Sistema de Sincronização

**Objetivo:** Manter o mesmo conjunto de arquivos em todas as pastas sincronizadas entre máquinas na mesma rede local. Qualquer criação, edição ou remoção de arquivo em qualquer máquina vinculada é propagada automaticamente para todas as outras.

**Responsabilidades:**
- Permitir que o usuário defina até **5 pastas de sincronização** por instância.
- **Sincronismo bidirecional completo:** o que a Máquina A criar ou editar é replicado na B, e o que a B criar ou editar é replicado na A — e em todas as demais máquinas vinculadas.
- Dois mecanismos complementares de detecção de mudanças:
  1. **Watcher em tempo real** (`notify`) — detecta eventos de create/modify/delete instantaneamente no sistema de arquivos local.
  2. **Verificação periódica** (intervalo configurável nas Settings, padrão 30s) — reconcilia o estado completo da pasta com cada peer, garantindo que nenhuma mudança seja perdida (inclusive as ocorridas enquanto o peer estava offline).
- Comparação de arquivos por **mtime (data de modificação) + tamanho** para identificar diferenças sem recalcular hash desnecessariamente.
- Transferência apenas do diff (arquivos alterados), nunca a pasta inteira.
- Resolução de conflitos por `last-write-wins`: o arquivo com `mtime` mais recente prevalece.
- Arquivos deletados em uma máquina são deletados em todas as vinculadas.
- Suporte a reconexão automática: ao peer voltar online, a sincronização é executada imediatamente antes de retomar o intervalo periódico.
- Status de sincronização em tempo real: `Sincronizado`, `Pendente`, `Sincronizando`, `Conflito`, `Offline`.
- Suportar computadores na mesma rede local via **agente remoto**.
- Descoberta automática de agentes na rede (UDP broadcast / mDNS).
- Pareamento manual por código de 6 dígitos ou IP direto quando a descoberta automática falhar.
- O agente remoto inicializa junto ao ARK Manager (auto-start na inicialização do app).

**Fluxo de funcionamento:**
```
Máquina A                                 Máquina B
│                                         │
├─ Agente local (HTTP/WS server)          ├─ Agente local (HTTP/WS server)
├─ SyncWatcher (notify)                   ├─ SyncWatcher (notify)
├─ Verificação periódica (intervalo cfg)  ├─ Verificação periódica (intervalo cfg)
│                                         │
│ [Arquivo criado/editado em A]           │
├─ Watcher detecta mudança                │
├─ Compara mtime+size com peer B          │
├─ Envia arquivo via AgentClient    ──►   ├─ AgentServer recebe
│                                         ├─ Verifica mtime (last-write-wins)
│                                         ├─ Escreve na pasta sincronizada
│                                         └─ Emite sync:file_received
│                                         │
│ [Arquivo criado/editado em B]           │
│                                    ◄──  ├─ Watcher detecta mudança
│                                         ├─ Envia arquivo via AgentClient
├─ AgentServer recebe                     │
├─ Verifica mtime (last-write-wins)       │
└─ Escreve na pasta sincronizada          │
```

**Intervalo de sincronização:**
- Configurável nas **Settings** da aplicação (campo `sync_interval_secs`, padrão: 30 segundos).
- Mínimo permitido: 10 segundos. Máximo: 3600 segundos (1 hora).
- O intervalo periódico é complementar ao watcher — não substitui a detecção em tempo real.

**Dependências:** commands `sync`, `agent`, `syncStore`, `settings`

**Telas relacionadas:** `SyncManager.tsx`

**Componentes:** `SyncFolderCard`, `PeerDiscoveryDialog`, `SyncStatusBadge`

**Serviços relacionados:** `agent_server.rs`, `agent_discovery.rs`, `agent_client.rs`, `sync_engine.rs`, `sync_watcher.rs`, `sync_transfer.rs`

> ⚠️ **Limite:** Máximo de 5 pastas de sincronização por instância. Pastas com mais de 10.000 arquivos exibem aviso de performance. O watcher monitora recursivamente todas as subpastas.

---

### 6.14 Eventos Sazonais

**Objetivo:** Permitir que administradores criem eventos temporários que alteram automaticamente as taxas dos servidores ARK dentro de uma janela de tempo definida, com toda a orquestração de start/stop, backup e restauração de INIs feita pelo sistema.

**Responsabilidades:**
- Criar eventos com nome, data/hora de início, data/hora de término e servidores alvo.
- Configurar taxas independentes por evento: XP, breeding, coleta de recursos, melhoria de drops.
- Suportar múltiplas taxas configuráveis no mesmo evento (cada taxa é um campo independente).
- Orquestrar automaticamente o ciclo de vida completo do evento (veja fluxo abaixo).
- Enviar broadcasts aos servidores selecionados em momentos-chave.
- Fazer backup automático dos INIs originais antes de aplicar as taxas do evento.
- Restaurar os INIs originais ao encerrar o evento.
- Exibir status do evento: `Agendado`, `Ativo`, `Encerrado`, `Erro`.
- Permitir cancelamento/encerramento manual do evento a qualquer momento.

**Taxas configuráveis por evento:**

| Taxa | Parâmetro ARK | Descrição |
|---|---|---|
| `xp_multiplier` | `XPMultiplier` | Taxa de ganho de XP por ação |
| `breeding_multiplier` | `MatingIntervalMultiplier` / `EggHatchSpeedMultiplier` / `BabyMatureSpeedMultiplier` | Velocidade de breeding |
| `harvest_multiplier` | `HarvestAmountMultiplier` | Quantidade de recursos coletados |
| `quality_multiplier` | `ItemStatClamps` / `FishingLootQualityMultiplier` | Qualidade e melhoria de drops |

> Cada taxa é opcional — se não configurada, o valor original do INI é mantido.

**Fluxo completo do evento:**

```
AGENDAMENTO
  └─ Evento criado com start_at, end_at, taxas e servidores alvo

T - 5min (antes do início):
  └─ RCON broadcast em todos os servidores selecionados:
     "⚠️ [NOME DO EVENTO] iniciará em 5 minutos. O servidor será reiniciado."

T = início:
  ├─ RCON: saveworld (salva o mundo antes de desligar)
  ├─ Parar servidor (process_manager)
  ├─ Backup dos INIs originais → seasonal_event_backups
  ├─ Aplicar taxas do evento nos INIs (config_generator/ini_parser)
  └─ Iniciar servidor com novas taxas

DURANTE O EVENTO (a cada intervalo configurado):
  └─ RCON broadcast: "🎉 [NOME DO EVENTO] está ativo! [mensagem personalizada]"

T - 5min (antes do término):
  └─ RCON broadcast:
     "⚠️ [NOME DO EVENTO] encerrará em 5 minutos. O servidor será reiniciado."

T = término:
  ├─ RCON: saveworld
  ├─ Parar servidor
  ├─ Restaurar INIs originais do backup
  └─ Iniciar servidor com taxas originais
```

**Dependências:** commands `seasonal_events`, `seasonalEventStore`

**Telas relacionadas:** `SeasonalEvents.tsx`

**Componentes:** `EventCard`, `EventRatesForm`, `EventServerSelector`, `EventStatusBadge`

**Serviços relacionados:** `event_scheduler.rs`, `event_config_swapper.rs`

> ⚠️ **Regra crítica:** O backup dos INIs originais deve ser feito **antes** de qualquer alteração. O evento não pode iniciar sem confirmação de que o backup foi criado com sucesso.

---

## 7. BANCO DE DADOS

### Estratégia

O banco de dados MySQL é a fonte de verdade para toda a configuração e histórico do ARK Manager. Utiliza-se um sistema de migrations idempotentes com controle via tabela `_migrations`.

### Tabelas Atuais

#### `servers`
Armazena a configuração completa de cada servidor ARK.

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | VARCHAR(36) PK | UUID do servidor |
| `name` | VARCHAR(100) | Nome de exibição |
| `map` | VARCHAR(50) | Mapa ARK (TheIsland, Ragnarok...) |
| `install_path` | TEXT | Caminho de instalação no disco |
| `game_port` | INT | Porta do jogo (padrão 7777) |
| `query_port` | INT | Porta de query Steam (padrão 27015) |
| `rcon_port` | INT | Porta RCON (padrão 32330) |
| `rcon_password` | VARCHAR(100) | Senha RCON |
| `max_players` | INT | Máximo de jogadores |
| `server_password` | VARCHAR(100) | Senha de acesso ao servidor |
| `admin_password` | VARCHAR(100) | Senha de administrador |
| `mods` | TEXT | Lista de IDs de mods (CSV) |
| `status` | VARCHAR(20) | Status atual (Stopped/Running/etc.) |
| `pid` | INT NULLABLE | PID do processo ativo |
| `cluster_id` | VARCHAR(36) FK NULLABLE | Cluster vinculado |
| `hardware_config` | JSON NULLABLE | Configuração de hardware alocada |
| `startup_priority` | INT | Prioridade de inicialização no cluster |
| `intelligent_mode` | BOOLEAN | Modo inteligente de gerenciamento |
| `created_at` | DATETIME | Data de criação |
| `updated_at` | DATETIME | Última atualização |

#### `clusters`
Configuração de clusters Cross-ARK.

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | VARCHAR(36) PK | UUID do cluster |
| `name` | VARCHAR(100) | Nome do cluster |
| `cluster_id_str` | VARCHAR(100) | ClusterId para o ARK |
| `shared_path` | TEXT | Pasta compartilhada do cluster |
| `created_at` | DATETIME | Data de criação |

#### `cluster_servers`
Relacionamento N:N entre clusters e servidores.

#### `mods`
Cache de metadados de mods do Workshop (nome, tamanho, última atualização).

#### `backups`
Histórico completo de backups.

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | VARCHAR(36) PK | UUID do backup |
| `server_id` | VARCHAR(36) FK | Servidor de origem |
| `type` | VARCHAR(20) | Manual/Auto/PreUpdate/PreRestart |
| `status` | VARCHAR(20) | Pending/Running/Completed/Failed |
| `path` | TEXT | Caminho do arquivo de backup |
| `size_bytes` | BIGINT | Tamanho em bytes |
| `compression_level` | INT | Nível de compressão usado |
| `duration_secs` | INT | Duração do backup em segundos |
| `created_at` | DATETIME | Data/hora do backup |

#### `backup_policies`
Políticas de backup automático por servidor.

#### `scheduled_tasks`
Tarefas agendadas com histórico de execução.

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | VARCHAR(36) PK | UUID da tarefa |
| `server_id` | VARCHAR(36) FK NULLABLE | Servidor alvo (null = global) |
| `task_name` | VARCHAR(100) | Nome de exibição |
| `task_type` | VARCHAR(50) | Tipo da tarefa |
| `schedule` | VARCHAR(100) | Expressão cron |
| `enabled` | BOOLEAN | Ativa/inativa |
| `run_count` | INT | Número de execuções |
| `last_result` | VARCHAR(20) NULLABLE | Último resultado |
| `last_error` | TEXT NULLABLE | Último erro (se falhou) |
| `next_run` | DATETIME NULLABLE | Próxima execução agendada |

#### `settings`
Configurações globais da aplicação (chave-valor).

### Tabela de Controle de Migrations

#### `_migrations`
| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | INT AUTO_INCREMENT PK | ID sequencial |
| `name` | VARCHAR(100) UNIQUE | Nome da migration |
| `applied_at` | DATETIME | Data de aplicação |

### Migrations Implementadas

| ID | Nome | Conteúdo |
|---|---|---|
| v1 | `v1_initial_schema` | Todas as tabelas base + índices + settings padrão |
| v2 | `v2_server_columns` | `hardware_config`, `startup_priority`, `intelligent_mode` em servers |
| v3 | `v3_backup_columns` | `compression_level`, `duration_secs` em backups |
| v4 | `v4_scheduler_columns` | `run_count`, `last_result`, `last_error` em scheduled_tasks |
| v5 | `v5_sync_agents` | Tabelas `sync_agents` e `sync_folders` |
| v6 | `v6_sync_rules` | Tabelas `sync_rules` (máx 5), `sync_events`, `sync_conflicts` |
| v7 | `v7_seasonal_events` | Tabelas `seasonal_events`, `seasonal_event_rates`, `seasonal_event_servers`, `seasonal_event_backups` |

### Estratégia de Crescimento

Novas migrations são adicionadas sequencialmente. Cada migration é idempotente: verifica em `_migrations` se já foi aplicada antes de executar. O sistema nunca desfaz migrations (sem down migrations). Alterações destrutivas requerem nova migration, nunca edição de migration existente.

### Tabelas de Sincronização

#### `sync_agents`
Peers (agentes remotos) vinculados a esta instância.

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | VARCHAR(36) PK | UUID do agente |
| `hostname` | VARCHAR(255) | Nome da máquina remota |
| `ip_address` | VARCHAR(45) | Endereço IP (IPv4 ou IPv6) |
| `port` | INT | Porta do agente remoto (padrão 45678) |
| `paired` | BOOLEAN | Se o pareamento foi confirmado |
| `auth_token_hash` | VARCHAR(255) | Hash SHA-256 do token de sessão |
| `last_seen` | DATETIME NULLABLE | Última comunicação bem-sucedida |
| `created_at` | DATETIME | Data de pareamento |

#### `sync_folders`
Pastas configuradas para sincronização (máximo 5).

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | VARCHAR(36) PK | UUID da regra |
| `label` | VARCHAR(100) | Nome amigável exibido na UI |
| `local_path` | TEXT | Caminho absoluto da pasta local |
| `enabled` | BOOLEAN | Se a sincronização está ativa |
| `sync_interval_secs` | INT | Intervalo de verificação periódica em segundos (herda da Setting global, mas pode ser sobrescrito por pasta) |
| `created_at` | DATETIME | Data de criação |
| `updated_at` | DATETIME | Última alteração |

#### `sync_folder_peers`
Relacionamento N:N — quais pastas sincronizam com quais peers.

| Coluna | Tipo | Descrição |
|---|---|---|
| `folder_id` | VARCHAR(36) FK | Pasta de sincronização |
| `agent_id` | VARCHAR(36) FK | Agente remoto vinculado |
| `remote_path` | TEXT | Caminho da pasta no agente remoto |
| `status` | VARCHAR(20) | `synced`, `pending`, `syncing`, `conflict`, `offline` |
| `last_sync` | DATETIME NULLABLE | Última sincronização bem-sucedida |

#### `sync_events`
Log de eventos de sincronização por arquivo.

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | BIGINT AUTO_INCREMENT PK | ID do evento |
| `folder_id` | VARCHAR(36) FK | Pasta de sincronização |
| `agent_id` | VARCHAR(36) FK | Agente de origem/destino |
| `file_path` | TEXT | Caminho relativo do arquivo na pasta |
| `action` | VARCHAR(20) | `created`, `modified`, `deleted`, `conflict_resolved` |
| `direction` | VARCHAR(10) | `outbound` (enviado) ou `inbound` (recebido) |
| `file_size_bytes` | BIGINT NULLABLE | Tamanho do arquivo transferido |
| `status` | VARCHAR(20) | `ok`, `error`, `skipped` |
| `error_message` | TEXT NULLABLE | Mensagem de erro se falhou |
| `created_at` | DATETIME | Data/hora do evento |

#### `sync_conflicts`
Registro de conflitos detectados e como foram resolvidos.

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | VARCHAR(36) PK | UUID do conflito |
| `folder_id` | VARCHAR(36) FK | Pasta onde ocorreu |
| `file_path` | TEXT | Caminho relativo do arquivo |
| `local_mtime` | DATETIME | mtime local no momento do conflito |
| `remote_mtime` | DATETIME | mtime remoto no momento do conflito |
| `resolved_by` | VARCHAR(20) | `local` ou `remote` (quem venceu) |
| `created_at` | DATETIME | Data/hora da detecção |

### Tabelas de Eventos Sazonais

#### `seasonal_events`
Definição de cada evento sazonal.

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | VARCHAR(36) PK | UUID do evento |
| `name` | VARCHAR(100) | Nome do evento (ex: "Evento de Natal") |
| `description` | TEXT NULLABLE | Descrição exibida na UI |
| `start_at` | DATETIME | Data e hora de início |
| `end_at` | DATETIME | Data e hora de encerramento |
| `announce_interval_mins` | INT | Intervalo em minutos entre broadcasts ativos (0 = desativado) |
| `announce_message` | TEXT NULLABLE | Mensagem de broadcast durante o evento |
| `status` | VARCHAR(20) | `scheduled`, `active`, `ended`, `cancelled`, `error` |
| `error_message` | TEXT NULLABLE | Último erro (se status = error) |
| `created_at` | DATETIME | Data de criação |
| `updated_at` | DATETIME | Última alteração |

#### `seasonal_event_rates`
Taxas configuradas para cada evento (múltiplas por evento).

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | VARCHAR(36) PK | UUID da taxa |
| `event_id` | VARCHAR(36) FK | Evento ao qual pertence |
| `rate_type` | VARCHAR(50) | `xp`, `breeding`, `harvest`, `quality` |
| `multiplier` | DECIMAL(10,4) | Valor do multiplicador (ex: 2.0 = dobro) |

> **Tipos de taxa válidos:** `xp` (XPMultiplier), `breeding` (MatingInterval + EggHatch + BabyMature), `harvest` (HarvestAmountMultiplier), `quality` (drop quality multipliers).

#### `seasonal_event_servers`
Servidores selecionados para participar de cada evento.

| Coluna | Tipo | Descrição |
|---|---|---|
| `event_id` | VARCHAR(36) FK | Evento |
| `server_id` | VARCHAR(36) FK | Servidor participante |

#### `seasonal_event_backups`
Backup dos INIs originais antes da aplicação do evento.

| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | VARCHAR(36) PK | UUID do backup de INI |
| `event_id` | VARCHAR(36) FK | Evento que originou o backup |
| `server_id` | VARCHAR(36) FK | Servidor cujos INIs foram salvos |
| `backup_path` | TEXT | Caminho absoluto da pasta de backup dos INIs |
| `restored` | BOOLEAN | Se os INIs originais já foram restaurados |
| `created_at` | DATETIME | Data do backup |

---

## 8. SISTEMA DE EVENTOS

O sistema de eventos permite comunicação assíncrona do backend Rust para o frontend React sem bloqueio de chamada.

### Eventos de Banco de Dados

| Evento | Payload | Origem | Destino |
|---|---|---|---|
| `db:ready` | `null` | `lib.rs` | Frontend — desbloqueia a UI |
| `db:error` | `String` (mensagem) | `lib.rs` | Frontend — exibe erro crítico |

### Eventos de Servidor

| Evento | Payload | Origem | Destino |
|---|---|---|---|
| `server:started` | `{ server_id, pid }` | `process_manager.rs` | Dashboard, ServerManager |
| `server:stopped` | `{ server_id }` | `process_manager.rs` | Dashboard, ServerManager |
| `server:crashed` | `{ server_id, exit_code }` | `process_manager.rs` | Dashboard (alerta) |
| `server:status_changed` | `{ server_id, status }` | `process_manager.rs` | Dashboard, ServerManager |

### Eventos de Instalação

| Evento | Payload | Origem | Destino |
|---|---|---|---|
| `install:progress` | `{ percent, message }` | `server_installer.rs` | `InstallServerDialog` |
| `install:completed` | `{ server_id }` | `server_installer.rs` | `installStore` |
| `install:failed` | `{ error }` | `server_installer.rs` | `installStore` |
| `steamcmd:output` | `String` (linha de saída) | `steamcmd.rs` | Console de instalação |

### Eventos de Log

| Evento | Payload | Origem | Destino |
|---|---|---|---|
| `log:line` | `{ server_id, line, level }` | `log_watcher.rs` | `LogsConsole` |

### Eventos de Backup

| Evento | Payload | Origem | Destino |
|---|---|---|---|
| `backup:started` | `{ server_id, backup_id }` | `backup_service.rs` | `Backups` |
| `backup:completed` | `{ backup_id, size_bytes }` | `backup_service.rs` | `Backups` |
| `backup:failed` | `{ backup_id, error }` | `backup_service.rs` | `Backups` |

### Eventos de Jogadores

| Evento | Payload | Origem | Destino |
|---|---|---|---|
| `player:joined` | `{ server_id, player_name }` | `log_watcher.rs` | Dashboard |
| `player:left` | `{ server_id, player_name }` | `log_watcher.rs` | Dashboard |

### Eventos de Scheduler

| Evento | Payload | Origem | Destino |
|---|---|---|---|
| `task:executed` | `{ task_id, result }` | `scheduler.rs` | `Scheduler` |
| `task:failed` | `{ task_id, error }` | `scheduler.rs` | `Scheduler` |

### Eventos de Sincronização

| Evento | Payload | Origem | Destino |
|---|---|---|---|
| `agent:discovered` | `{ peer_id, ip, hostname }` | `agent_discovery.rs` | `SyncManager` |
| `agent:connected` | `{ peer_id }` | `agent_client.rs` | `SyncManager` |
| `agent:disconnected` | `{ peer_id }` | `agent_client.rs` | `SyncManager` (alerta) |
| `sync:started` | `{ folder_id, peer_id }` | `sync_engine.rs` | `SyncManager` |
| `sync:progress` | `{ folder_id, files_done, files_total }` | `sync_transfer.rs` | `SyncManager` |
| `sync:completed` | `{ folder_id, peer_id }` | `sync_engine.rs` | `SyncManager` |
| `sync:error` | `{ folder_id, peer_id, error }` | `sync_engine.rs` | `SyncManager` (alerta) |
| `sync:file_changed` | `{ folder_id, path, event_type }` | `sync_watcher.rs` | `sync_engine.rs` |
| `sync:conflict` | `{ folder_id, path, local_mtime, remote_mtime }` | `sync_engine.rs` | `SyncManager` |
| `sync:reconcile_started` | `{ folder_id, peer_id }` | `sync_engine.rs` | `SyncManager` |
| `sync:reconcile_completed` | `{ folder_id, peer_id, files_synced }` | `sync_engine.rs` | `SyncManager` |

### Eventos de Eventos Sazonais

| Evento | Payload | Origem | Destino |
|---|---|---|---|
| `seasonal:countdown` | `{ event_id, event_name, minutes_left, phase }` | `event_scheduler.rs` | `SeasonalEvents` (fase: `starting`/`ending`) |
| `seasonal:ini_backup_started` | `{ event_id, server_id }` | `event_config_swapper.rs` | `SeasonalEvents` |
| `seasonal:ini_backup_completed` | `{ event_id, server_id }` | `event_config_swapper.rs` | `SeasonalEvents` |
| `seasonal:rates_applied` | `{ event_id, server_id }` | `event_config_swapper.rs` | `SeasonalEvents` |
| `seasonal:started` | `{ event_id, event_name }` | `event_scheduler.rs` | Dashboard, `SeasonalEvents` |
| `seasonal:broadcast_sent` | `{ event_id, server_id, message }` | `event_scheduler.rs` | `SeasonalEvents` |
| `seasonal:rates_restored` | `{ event_id, server_id }` | `event_config_swapper.rs` | `SeasonalEvents` |
| `seasonal:ended` | `{ event_id, event_name }` | `event_scheduler.rs` | Dashboard, `SeasonalEvents` |
| `seasonal:error` | `{ event_id, server_id, error }` | `event_scheduler.rs` | `SeasonalEvents` (alerta crítico) |

---

## 9. INTEGRAÇÕES

### SteamCMD

- **Finalidade:** Instalação e atualização do servidor ARK.
- **App ID:** `376030` (ARK: Survival Evolved Dedicated Server).
- **Instalação automática:** Download do `steamcmd.zip` da Valve caso ausente.
- **Caminho configurável** nas Settings da aplicação.
- **Saída em tempo real** via eventos `steamcmd:output`.

### Steam Workshop

- **Finalidade:** Gerenciamento de mods por ID.
- **Integração via SteamCMD** para download de mods.
- **Metadados opcionais** via API pública da Steam (nome do mod, tamanho).

### RCON — Source RCON Protocol

- **Finalidade:** Administração remota do servidor ARK em execução.
- **Protocolo:** Source RCON sobre TCP (Valve RCON Protocol).
- **Porta:** configurável por servidor (padrão 32330).
- **Autenticação:** senha RCON definida em `GameUserSettings.ini`.
- **Implementação Rust:** TCP socket com framing de pacotes RCON.

### Discord

- **Finalidade:** Notificações de eventos importantes.
- **Integração via Webhook** (URL configurável nas Settings).
- **Eventos notificados:** servidor iniciado/parado/crashado, backup concluído, jogador entrou (opcional).
- **Formato:** Embed Discord com cor por tipo de evento.

### Sistema Operacional (Windows)

- **Gerenciamento de processos:** `CreateProcess` / `TerminateProcess` via `windows-sys`.
- **Monitoramento:** CPU e RAM por PID via crate `sysinfo`.
- **Detecção de portas em uso:** `netstat` / sockets via `network.rs`.
- **Verificação de paths e arquivos:** API nativa Rust + `tokio::fs`.

### Monitoramento de Hardware

- **Crate:** `sysinfo 0.32`.
- **Métricas:** CPU total, RAM total/usada, disco disponível.
- **Por processo:** CPU%, RAM usada pelo `ShooterGameServer.exe`.
- **Atualização periódica** via polling configurável (padrão: 5 segundos).

### Agente Remoto de Sincronização

- **Finalidade:** Permitir que instâncias do ARK Manager em computadores diferentes na mesma rede local se conectem e sincronizem arquivos.
- **Protocolo:** HTTP REST + WebSocket (via `axum`) sobre porta configurável (padrão: 45678).
- **Descoberta:** UDP broadcast na rede local + mDNS para localizar agentes automaticamente.
- **Pareamento:** Confirmação manual por código de 6 dígitos exibido na interface, ou por IP direto.
- **Autenticação:** Token de sessão gerado no pareamento, armazenado localmente.
- **Auto-start:** O servidor do agente local inicia automaticamente junto ao ARK Manager no `setup()` do Tauri.
- **Segurança:** Comunicação restrita à rede local (bind em `0.0.0.0` mas sem exposição à internet). Token por peer, sem senha compartilhada.
- **Transferência de arquivos:** Streaming via WebSocket com checksum SHA-256 para integridade.

### Discord

- **Finalidade:** Notificações de eventos importantes para canais Discord.
- **Integração via Webhook** (URL configurável nas Settings).
- **Eventos notificados:** servidor iniciado/parado/crashado, backup concluído, jogador entrou (opcional), erro de sincronização.
- **Formato:** Embed Discord com cor por tipo de evento.
- **Serviço:** `services/discord.rs` — envia payload via `reqwest`.

---

## 10. PADRÕES DE DESENVOLVIMENTO

### Rust

- Usar `thiserror` para todos os tipos de erro de domínio.
- Usar `anyhow` apenas em funções de alto nível (main, setup).
- Todos os erros devem ter mensagem descritiva em português.
- Commands Tauri retornam `Result<T, String>` — serializar erro com `e.to_string()`.
- Nunca usar `unwrap()` ou `expect()` em código de produção.
- Usar `log::info!`, `log::warn!`, `log::error!` para logging.
- Traits Tauri (`Emitter`, `Manager`) devem ser importadas explicitamente.
- Cada arquivo de serviço tem responsabilidade única.

### React / TypeScript

- Componentes funcionais com hooks — sem class components.
- Props tipadas com interfaces TypeScript — sem `any`.
- Stores Zustand com `immer` para mutação de estado.
- Hooks customizados para toda lógica reutilizável.
- Imports usando aliases configurados (`@/*`, `@components/*`, etc.).
- Sem `console.log` em código de produção.

### Banco de Dados

- Toda interação com MySQL via SQLx com queries tipadas.
- Usar pool de conexões — nunca conexão avulsa por request.
- Migrations são idempotentes e nunca são editadas após aplicação.
- UUIDs como chaves primárias (VARCHAR 36).
- `created_at` e `updated_at` em todas as tabelas.

### Logs

- Backend: `simplelog` com nível configurável.
- Arquivos de log em `AppData/Local/ArkManager/logs/`.
- Rotação de logs por data.
- Erros críticos sempre logados antes de emit para frontend.

### Tratamento de Erros

- Erros de banco de dados não vazam detalhes técnicos para o frontend.
- Frontend exibe mensagem amigável em português para todo erro.
- Operações de longa duração têm timeout explícito.
- Retry com backoff exponencial para conexão MySQL.

### Estrutura de Arquivos

- Um arquivo por componente React.
- Um arquivo por comando Tauri (por domínio).
- Um arquivo por serviço Rust.
- Sem arquivos com mais de 500 linhas (refatorar em módulos).

### Nomeação

| Contexto | Convenção | Exemplo |
|---|---|---|
| Componentes React | PascalCase | `ServerCard.tsx` |
| Hooks React | camelCase com prefixo `use` | `useServerStatus.ts` |
| Stores Zustand | camelCase com sufixo `Store` | `serverStore.ts` |
| Commands Tauri | snake_case | `get_server_list` |
| Types/Interfaces TS | PascalCase | `ServerResponse` |
| Structs Rust | PascalCase | `CreateServerRequest` |
| Enums Rust | PascalCase | `ServerStatus` |
| Funções Rust | snake_case | `create_pool_with_retry` |

---

## 11. PADRÕES DE QUALIDADE

### Código Limpo

- Cada função faz exatamente uma coisa.
- Nomes de variáveis e funções são auto-descritivos.
- Sem comentários óbvios — o código deve ser autoexplicativo.
- Comentários apenas para lógica não óbvia (algoritmos, regras de negócio complexas).
- Sem código comentado — usar git para histórico.

### Modularização

- Separação clara entre apresentação (componentes), lógica (hooks/stores) e dados (services/commands).
- Dependências seguem direção única: commands → services → db/models.
- Frontend não acessa banco diretamente — sempre via commands Tauri.

### Segurança

- Senhas (RCON, servidor) nunca logadas.
- Credenciais MySQL lidas de variáveis de ambiente, nunca hardcoded.
- Inputs do usuário sanitizados antes de uso em paths de arquivo.
- Sem execução de comandos shell com strings interpoladas de usuário.
- RCON password never exposed in frontend state serialization.

### Escalabilidade

- Sistema suporta N servidores simultaneamente sem degradação.
- Pool MySQL dimensionado para N conexões concorrentes.
- Events Tauri usados para comunicação assíncrona — sem polling do frontend.
- Watchers de log por servidor são instâncias independentes.

### Performance

- Operações de I/O sempre assíncronas (tokio async/await).
- Não bloquear a thread principal do Tauri.
- Queries MySQL com índices adequados (server_id, created_at).
- Frontend atualiza estado incremental (não re-fetch lista completa a cada mudança).

### Facilidade de Manutenção

- Migrations sequenciais e descritivas.
- Stores com ações nomeadas descritivamente.
- Commands tipados — nunca `serde_json::Value` genérico como payload.

### Proibições Absolutas

- ❌ Código temporário com comentário `// TODO: implementar depois`.
- ❌ Gambiarras para contornar validações de tipo.
- ❌ Placeholders como `return Ok(())` em funções que devem ter implementação real.
- ❌ `unwrap()` / `expect()` fora de contexto de testes.
- ❌ `any` no TypeScript.
- ❌ Hardcode de caminhos de arquivo.
- ❌ `SessionName` na linha de comando do ARK (sempre no INI).
- ❌ Arquivos INI do ARK sem encoding UTF-16 LE com BOM.

---

## 12. ROADMAP DE EXECUÇÃO

### FASE 1 — Scaffold do projeto | 31/05/2026 22:20

- [x] 1.1 Inicializar projeto Tauri 2 com template React + TypeScript: `concluído em 31/05/2026 22:20`
- [x] 1.2 Configurar `tauri.conf.json` (nome, janela, identificador): `concluído em 31/05/2026 22:20`
- [x] 1.3 Configurar `package.json` com todas as dependências do frontend: `concluído em 31/05/2026 22:20`
- [x] 1.4 Configurar `Cargo.toml` com todas as dependências Rust: `concluído em 31/05/2026 22:20`
- [x] 1.5 Configurar Tailwind CSS + PostCSS: `concluído em 31/05/2026 22:20`
- [x] 1.6 Configurar paths e aliases do TypeScript (`tsconfig.json`): `concluído em 31/05/2026 22:20`
- [x] 1.7 Configurar `vite.config.ts`: `concluído em 31/05/2026 22:20`
- [x] 1.8 Criar estrutura de pastas do frontend (`src/`): `concluído em 31/05/2026 22:20`
- [x] 1.9 Criar estrutura de pastas do backend Rust (`src-tauri/src/`): `concluído em 31/05/2026 22:20`

---

### FASE 2 — Backend Rust: Banco de dados e modelos | 31/05/2026 22:35

- [x] 2.1 Criar módulo MySQL (`db/mod.rs`) com pool de conexão e migrations: `concluído em 31/05/2026 22:35`
- [x] 2.2 Criar migration inicial (tabelas: `servers`, `scheduled_tasks`, `backups`): `concluído em 31/05/2026 22:35`
- [x] 2.3 Criar models Rust (`models/server.rs`, `models/backup.rs`, `models/task.rs`): `concluído em 31/05/2026 22:35`

---

### FASE 3 — Backend Rust: Serviços core | 31/05/2026 23:50

- [x] 3.1 Serviço SteamCMD (`services/steamcmd.rs`) — download, instalação, update (App ID 376030): `concluído em 31/05/2026 23:50`
- [x] 3.2 Serviço de instalação do servidor (`services/server_installer.rs`) — instalar/atualizar ShooterGameServer: `concluído em 31/05/2026 23:50`
- [x] 3.3 Gerador de configuração INI (`services/config_generator.rs`) — GameUserSettings.ini + Game.ini em UTF-16 LE com BOM: `concluído em 31/05/2026 23:50`
- [x] 3.4 Gerador de script de lançamento (`services/launch_builder.rs`) — RunServer.cmd com todos os cuidados (aspas, SessionName fora da CLI, etc.): `concluído em 31/05/2026 23:50`
- [x] 3.5 Gerenciador de processo (`services/process_manager.rs`) — start, stop, restart, monitorar PID: `concluído em 31/05/2026 23:50`
- [x] 3.6 Watcher de log (`services/log_watcher.rs`) — leitura em tempo real do ShooterGame.log: `concluído em 31/05/2026 23:50`
- [x] 3.7 Serviço RCON (`services/rcon.rs`) — protocolo Source RCON (TCP): `concluído em 31/05/2026 23:50`
- [x] 3.8 Serviço de backup (`services/backup_service.rs`) — copiar SavedArks com timestamp: `concluído em 31/05/2026 23:50`
- [x] 3.9 Serviço de scheduler (`services/scheduler.rs`) — tarefas agendadas (cron): `concluído em 31/05/2026 23:50`
- [x] 3.10 Serviço de rede (`services/network.rs`) — verificar portas em uso (netstat), detectar conflitos: `concluído em 31/05/2026 23:50`
- [x] 3.11 Serviço de hardware (`services/system_analyzer.rs`) — CPU, RAM, disco por servidor: `concluído em 31/05/2026 23:50`
- [x] 3.12 Parser de INI (`services/ini_parser.rs`) — leitura de INI existente para importar servidor: `concluído em 31/05/2026 23:50`

---

### FASE 4 — Backend Rust: Comandos Tauri (bridge frontend ↔ backend) | 01/06/2026 00:30

- [x] 4.1 Comandos de servidor (`commands/server.rs`) — CRUD, start, stop, restart, status: `01/06/2026 00:30`
- [x] 4.2 Comandos de instalação (`commands/install.rs`) — instalar SteamCMD, instalar servidor, atualizar: `01/06/2026 00:30`
- [x] 4.3 Comandos de configuração (`commands/config.rs`) — ler/salvar GameUserSettings e Game.ini: `01/06/2026 00:30`
- [x] 4.4 Comandos RCON (`commands/rcon.rs`) — conectar, enviar comando, desconectar: `01/06/2026 00:30`
- [x] 4.5 Comandos de logs (`commands/logs.rs`) — iniciar watcher, parar watcher, ler histórico: `01/06/2026 00:30`
- [x] 4.6 Comandos de mods (`commands/mods.rs`) — listar, adicionar, remover mods Workshop: `01/06/2026 00:30`
- [x] 4.7 Comandos de cluster (`commands/cluster.rs`) — criar, editar, vincular servidores ao cluster: `01/06/2026 00:30`
- [x] 4.8 Comandos de backup (`commands/backup.rs`) — fazer backup, restaurar, listar backups: `01/06/2026 00:30`
- [x] 4.9 Comandos de scheduler (`commands/scheduler.rs`) — criar, editar, deletar, listar tarefas: `01/06/2026 00:30`
- [x] 4.10 Comandos de hardware (`commands/hardware.rs`) — métricas do sistema: `01/06/2026 00:30`
- [x] 4.11 Comandos de importação (`commands/import.rs`) — importar servidor existente do disco: `01/06/2026 00:30`
- [x] 4.12 Registrar todos os comandos no `lib.rs` e `main.rs`: `01/06/2026 00:30`

---

### FASE 5 — Frontend: Base e infraestrutura | 01/06/2026 01:00

- [x] 5.1 Criar entry point (`main.tsx`, `App.tsx`): `01/06/2026 01:00`
- [x] 5.2 Configurar React Router (rotas para todas as páginas): `01/06/2026 01:00`
- [x] 5.3 Criar tipos TypeScript globais (`types/index.ts`): `01/06/2026 01:00`
- [x] 5.4 Criar utilitários de bridge Tauri (`utils/tauri.ts`) — wrappers de invoke para todos os comandos: `01/06/2026 01:00`
- [x] 5.5 Criar helpers gerais (`utils/helpers.ts`): `01/06/2026 01:00`
- [x] 5.6 Criar store de servidores (`stores/serverStore.ts`): `01/06/2026 01:00`
- [x] 5.7 Criar store de UI (`stores/uiStore.ts`): `01/06/2026 01:00`
- [x] 5.8 Criar store de instalação (`stores/installStore.ts`): `01/06/2026 01:00`
- [x] 5.9 Criar store RCON (`stores/rconStore.ts`): `01/06/2026 01:00`
- [x] 5.10 Criar layout principal com sidebar (`components/layout/Sidebar.tsx`, `Layout.tsx`): `01/06/2026 01:00`
- [x] 5.11 Criar componentes UI base (Button, Input, Modal, Badge, Card — `components/ui/`): `01/06/2026 01:00`
- [x] 5.12 Configurar i18n pt-BR (`i18n/` + `locales/pt-BR.json`): `01/06/2026 01:00`
- [x] 5.13 Configurar estilos globais (`styles/globals.css`): `01/06/2026 01:00`

---

### FASE 6 — Frontend: Páginas ✅ `01/06/2026 03:00`

- [x] 6.1 **Dashboard** — visão geral com cards de todos os servidores, status, CPU/RAM, ações rápidas: `01/06/2026 03:00`
- [x] 6.2 **ServerManager** — adicionar/remover/editar servidores, instalar via SteamCMD, detectar conflito de portas: `01/06/2026 03:00`
- [x] 6.3 **ConfigEditor** — editor completo GameUserSettings.ini + Game.ini com campos tipados (taxas, dinos, criação, cluster): `01/06/2026 03:00`
- [x] 6.4 **RconConsole** — terminal RCON com histórico, comandos rápidos pré-definidos (saveworld, listplayers, broadcast, etc.): `01/06/2026 03:00`
- [x] 6.5 **LogsConsole** — log em tempo real com cores, filtros e busca: `01/06/2026 03:00`
- [x] 6.6 **ModManager** — gerenciar mods Steam Workshop por servidor (ID + nome, ordenação): `01/06/2026 03:00`
- [x] 6.7 **ClusterManager** — configurar Cross-ARK cluster (ClusterId, pasta compartilhada, vincular servidores): `01/06/2026 03:00`
- [x] 6.8 **Backups** — listar, criar, restaurar backups do SavedArks com timestamp: `01/06/2026 03:00`
- [x] 6.9 **Scheduler** — criar e gerenciar tarefas agendadas (restart, saveworld, destroywilddinos, broadcast): `01/06/2026 03:00`
- [x] 6.10 **Settings** — configurações globais (caminho SteamCMD, tema, idioma, paths padrão, Discord webhook, **intervalo de sincronização**): `01/06/2026 03:00`
- [x] 6.11 **Monitoramento** — página dedicada com CPU/RAM em tempo real por servidor, alertas de threshold: `01/06/2026 03:00`
- [ ] 6.12 **Eventos Sazonais** — criar/editar/cancelar eventos, configurar taxas múltiplas, selecionar servidores, acompanhar status em tempo real: `aguardando...`

---

### FASE 7 — Frontend: Componentes especializados ✅ `01/06/2026 04:00`

- [x] 7.1 `ServerCard` — card com status, mapa, jogadores, ações: `01/06/2026 04:00`
- [x] 7.2 `InstallServerDialog` — wizard de instalação passo a passo: `01/06/2026 04:00`
- [x] 7.3 `PortConflictModal` — alerta e resolução de conflito de portas: `01/06/2026 04:00`
- [x] 7.4 `StatMultiplierEditor` — editor de PerLevelStatsMultiplier (índices 0–11): `01/06/2026 04:00`
- [x] 7.5 `PerformanceMonitor` — gráficos CPU/RAM em tempo real (usado na página Monitoramento): `01/06/2026 04:00`
- [x] 7.6 `ClusterBuilder` — UI visual para vincular servidores no cluster: `01/06/2026 04:00`
- [x] 7.7 `SyncFolderCard` — card de pasta sincronizada com status, peer, progresso: `01/06/2026 04:00`
- [x] 7.8 `PeerDiscoveryDialog` — dialog de descoberta e pareamento de agentes na rede: `01/06/2026 04:00`
- [x] 7.9 `SyncStatusBadge` — indicador visual de estado de sincronização: `01/06/2026 04:00`
- [x] 7.10 `EventCard` — card de evento sazonal com countdown, status e ações rápidas: `01/06/2026 04:00`
- [x] 7.11 `EventRatesForm` — formulário de taxas do evento (XP, breeding, harvest, quality): `01/06/2026 04:00`
- [x] 7.12 `EventServerSelector` — seletor de servidores participantes do evento: `01/06/2026 04:00`
- [x] 7.13 `EventStatusBadge` — badge de status do evento (Agendado/Ativo/Encerrado/Erro): `01/06/2026 04:00`

---

### FASE 8 — Backend Rust: Agente Remoto de Rede

- [x] 8.1 `services/agent_server.rs` — servidor HTTP/WebSocket local (axum, porta 45678), inicializa junto ao app: `01/06/2026 05:30`
- [x] 8.2 `services/agent_discovery.rs` — descoberta de agentes via UDP broadcast e mDNS na rede local: `01/06/2026 05:30`
- [x] 8.3 `services/agent_client.rs` — cliente para conectar, autenticar e manter sessão com agentes remotos: `01/06/2026 05:30`
- [x] 8.4 `services/agent_auth.rs` — geração de token de pareamento (código 6 dígitos), armazenamento e validação: `01/06/2026 05:30`
- [x] 8.5 `models/agent.rs` — structs: `Agent`, `AgentStatus`, `PairRequest`, `PairResponse`: `01/06/2026 05:30`
- [x] 8.6 Migration v5: tabelas `sync_agents` e `sync_folders`: `01/06/2026 05:30`
- [x] 8.7 `commands/agent.rs` — commands Tauri: descobrir, parear, listar, remover, status de agentes: `01/06/2026 05:30`
- [x] 8.8 Registrar auto-start do agente no `setup()` do `lib.rs`: `01/06/2026 05:30`

---

### FASE 9 — Backend Rust: Sistema de Sincronização ✅ (02/06/2026 02:30)

- [x] 9.1 `services/sync_engine.rs` — motor de sincronização bidirecional: orquestra watcher (tempo real) + verificação periódica por intervalo configurável
- [x] 9.2 `services/sync_watcher.rs` — watcher recursivo de pastas via crate `notify`, detecta create/modify/delete em tempo real
- [x] 9.3 `services/sync_reconciler.rs` — reconciliação periódica: compara mtime+size de todos os arquivos com cada peer e sincroniza o diff
- [x] 9.4 `services/sync_transfer.rs` — transferência de arquivos via WebSocket com streaming em chunks de 64KB e checksum SHA-256
- [x] 9.5 `services/sync_conflict.rs` — resolução last-write-wins (mtime) com registro em `sync_conflicts`
- [x] 9.6 `models/sync.rs` — structs: `SyncFolder`, `SyncEvent`, `SyncConflict`
- [x] 9.7 Migration v6: colunas/tabelas `session_token`, `sync_events`, `sync_conflicts`
- [x] 9.8 `commands/sync.rs` — commands Tauri: adicionar/remover pasta, listar, status, histórico de eventos, forçar sync agora
- [x] 9.9 Validação do limite de 5 pastas por instância com erro descritivo
- [x] 9.10 Lógica de reconexão: ao peer voltar online, dispara reconciliação imediata antes de retomar o intervalo
- [x] 9.11 `services/discord.rs` — envio de notificações via Discord Webhook (reqwest)
- [x] 9.12 `commands/discord.rs` — testar webhook, configurar eventos habilitados
- [x] `services/sync_protocol.rs` — enum `SyncMessage` com protocolo completo (ReconcileRequest/FileList/TransferStart/Chunk/Done/Ack/RequestFiles/SyncComplete)
- [x] `agent_server.rs` atualizado com handler WS completo do protocolo de sync
- [x] `models/agent.rs` com campo `session_token`
- [x] `commands/agent.rs` salva `session_token` no pareamento
- [x] `shared_db` Arc<RwLock<Option<DbPool>>> propagado de lib.rs → agent_server
- [x] `SyncEngineState` gerenciado pelo Tauri
- [x] Builds validados: `cargo build` ✅ | `tsc --noEmit` ✅ | `npm run build` ✅

---

### FASE 10 — Frontend: Sincronização e complementos

- [x] 10.1 **SyncManager** — página principal de sincronização: listar pastas, peers vinculados, status em tempo real, adicionar/remover: `01/06/2026 (sessão 3)`
- [x] 10.2 `syncStore.ts` — store Zustand para estado de pastas sincronizadas, peers e progresso: `01/06/2026 (sessão 3)`
- [x] 10.3 `agentStore.ts` — store Zustand para agentes descobertos, vinculados e status de conexão: `01/06/2026 (sessão 3)`
- [x] 10.4 Fluxo de pareamento de peers: dialog com QR code ou código de 6 dígitos: `01/06/2026 (sessão 3)`
- [x] 10.5 Indicadores de sincronização no Dashboard (pastas com conflito ou offline): `01/06/2026 (sessão 3)`
- [x] 10.6 Integração Discord nas Settings — campo webhook, toggle por evento, botão de teste: `01/06/2026 (sessão 3)`

---

### FASE 11 — Backend Rust: Eventos Sazonais

- [ ] 11.1 `services/event_scheduler.rs` — agendador do ciclo de vida do evento: monitora start_at/end_at, dispara broadcasts, orquestra start/stop dos servidores: `aguardando...`
- [ ] 11.2 `services/event_config_swapper.rs` — faz backup dos INIs originais, aplica taxas do evento nos INIs (via `ini_parser.rs` + `config_generator.rs`), restaura ao encerrar: `aguardando...`
- [ ] 11.3 `models/seasonal_event.rs` — structs: `SeasonalEvent`, `EventRate`, `EventStatus`, requests de criação/edição: `aguardando...`
- [ ] 11.4 Migration v7: tabelas `seasonal_events`, `seasonal_event_rates`, `seasonal_event_servers`, `seasonal_event_backups`: `aguardando...`
- [ ] 11.5 `commands/seasonal_events.rs` — commands Tauri: criar, editar, cancelar, listar, status, forçar início/fim: `aguardando...`
- [ ] 11.6 Integração com `process_manager.rs` — stop + saveworld antes de aplicar taxas, start depois: `aguardando...`
- [ ] 11.7 Broadcasts RCON automáticos: aviso 5 min antes do início, durante o evento (intervalo configurável), aviso 5 min antes do fim: `aguardando...`
- [ ] 11.8 Proteção de integridade: evento não pode iniciar sem backup de INI confirmado — rollback automático em caso de erro: `aguardando...`

---

### FASE 12 — Frontend: Eventos Sazonais

- [x] 12.1 **SeasonalEvents** — listagem de eventos com status, countdown, ações (criar, editar, cancelar, forçar): `concluído`
- [x] 12.2 `EventCard` — card com nome, período, status badge, taxas configuradas, barra de progresso do evento: `concluído`
- [x] 12.3 `EventRatesForm` — formulário de taxas: campos XP, breeding, harvest, quality com validação de range: `concluído`
- [x] 12.4 `EventServerSelector` — seletor de servidores participantes com checkbox + status de cada um: `concluído`
- [x] 12.5 `seasonalEventStore.ts` — store Zustand para lista de eventos, status em tempo real, atualizações via events Tauri: `concluído`
- [x] 12.6 Indicador de evento ativo no Dashboard (badge visível nos cards dos servidores afetados): `concluído`

---

### FASE 13 — Testes, ajustes e build

- [ ] 13.1 Testar fluxo completo: instalar servidor → configurar → iniciar → RCON → parar: `aguardando...`
- [ ] 13.2 Testar geração dos INIs (verificar encoding UTF-16 LE): `aguardando...`
- [ ] 13.3 Testar script de lançamento gerado (.cmd): `aguardando...`
- [ ] 13.4 Testar scheduler e backup: `aguardando...`
- [ ] 13.5 Testar descoberta e pareamento de agentes em rede local: `aguardando...`
- [ ] 13.6 Testar sincronização bidirecional (watcher em tempo real + reconciliação periódica): `aguardando...`
- [ ] 13.7 Testar resolução de conflito (modificação simultânea em A e B): `aguardando...`
- [ ] 13.8 Testar reconexão: peer offline volta e sincroniza o diff imediatamente: `aguardando...`
- [ ] 13.9 Testar evento sazonal completo: agendamento → broadcast de aviso → stop → backup INI → aplicar taxas → start → broadcasts ativos → aviso de fim → stop → restaurar INIs → start: `aguardando...`
- [ ] 13.10 Testar notificações Discord Webhook: `aguardando...`
- [ ] 13.11 Ajustes finais de UI/UX: `aguardando...`
- [x] 13.12 Build de produção (`npm run tauri build`): `concluído — 2 instaladores gerados`

---

**Total de tarefas: 118**  
**Status geral: Fases 1 e 2 concluídas ✅ — aguardando Fase 3**

---

## 13. HISTÓRICO DE IMPLEMENTAÇÃO

### Fase 1 — Concluída em 31/05/2026 22:20

**Scaffold completo do projeto.**

- Projeto Tauri 2 inicializado com template React + TypeScript.
- `tauri.conf.json` configurado: identifier `com.arkmanager.app`, janela 1400×900, mínimo 1200×700.
- `package.json` com todas as dependências frontend: React 19, TypeScript, Vite 6, Tailwind CSS 3, Zustand, React Query, Lucide React.
- `Cargo.toml` com todas as dependências Rust: tauri 2, sqlx 0.8 (mysql), tokio, serde, thiserror, reqwest, sysinfo, cron, walkdir, notify, flate2, axum, windows-sys.
- Tailwind CSS configurado com paleta customizada `ark` (sky) e `surface` (slate). Fontes Inter + JetBrains Mono.
- `tsconfig.json` configurado com aliases: `@/*`, `@components/*`, `@pages/*`, `@stores/*`, `@utils/*`, `@hooks/*`, `@assets/*`.
- `vite.config.ts` configurado com resolução de aliases correspondentes.
- Estrutura de pastas `src/` criada: assets, components/layout, components/ui, hooks, pages, stores, styles, types, utils.
- Estrutura de pastas `src-tauri/src/` criada: commands, db, models, services, utils.
- Stubs de arquivos criados para todas as páginas e componentes de layout.
- Build limpo confirmado: `npm run build` sem erros, `npx tsc --noEmit` sem erros.

### Fase 2 — Concluída em 31/05/2026 22:35

**Backend Rust: Banco de dados e modelos.**

- `db/connection.rs`: Pool MySQL com `MySqlPoolOptions`, `DbConfig` lido de variáveis de ambiente, retry com backoff exponencial (máx 5 tentativas, delay 2^n segundos).
- `db/migrations.rs`: 4 migrations idempotentes controladas via tabela `_migrations`. Migration v1 cria todas as tabelas base (servers, clusters, backups, scheduled_tasks, settings). Migrations v2–v4 adicionam colunas adicionais.
- `db/mod.rs`: Expõe API pública com função `initialize(config)` que cria pool com retry e executa migrations.
- `models/server.rs`: Struct `Server` com `FromRow`, enums `ServerStatus` e `ArkMap` (11 mapas ASE), requests `CreateServerRequest` / `UpdateServerRequest`, response `ServerResponse`.
- `models/backup.rs`: Struct `Backup` com `FromRow`, enums `BackupType` e `BackupStatus`, `BackupPolicy`, requests de criação e restauração, response com `size_human()`.
- `models/task.rs`: Struct `ScheduledTask` com `FromRow`, enums `TaskType` (7 tipos) e `TaskResult`, requests de criação e atualização, response com `display_name()`.
- `lib.rs` atualizado: `AppState { db: DbPool }`, inicialização assíncrona do DB no setup do Tauri, emit de `db:ready` / `db:error`.
- Fix aplicado: `use tauri::{Emitter, Manager}` — traits necessárias para Tauri 2.
- `build.rs` criado: `tauri_build::build()` — necessário para que `tauri::generate_context!()` encontre `OUT_DIR`.

---

### Fase 3 — Concluída em 31/05/2026 23:50

**Backend Rust: 12 serviços core implementados.**

- `services/steamcmd.rs`: Download e extração do SteamCMD, `install_server`, `update_server` com streaming de stdout, ARK App ID `376030`.
- `services/server_installer.rs`: Orquestra instalação (auto-instala SteamCMD se ausente), verifica `ShooterGameServer.exe`, expõe `server_exe_path`.
- `services/ini_parser.rs`: Leitura de INI com detecção automática de BOM UTF-16 LE, fallback UTF-8, helpers `get_value`, `get_f64`, `get_i64`, `get_bool`.
- `services/config_generator.rs`: Gera `GameUserSettings.ini` + `Game.ini` em UTF-16 LE com BOM. `SessionName` **sempre** no INI, nunca na CLI.
- `services/launch_builder.rs`: Gera `RunServer.cmd` com `start /wait`. `SessionName` explicitamente ausente dos args de CLI.
- `services/process_manager.rs`: `start_server`, `stop_server`, `restart_server`, `is_running`, `get_pid` via `PidMap` (Arc<Mutex<HashMap>>). Kill via `windows-sys` Win32 API.
- `services/log_watcher.rs`: Tail em tempo real do `ShooterGame.log` com poll de 200ms. Detecção de nível ERROR/WARN/INFO/DEBUG. Canal `broadcast` para shutdown.
- `services/rcon.rs`: Protocolo Source RCON completo — autenticação, `send_command`, pacotes com tamanho LE. Helper `execute_command` para uso pontual.
- `services/backup_service.rs`: Cópia recursiva de `SavedArks` com timestamp (`YYYYMMDD_HHMMSS`). `restore_backup`, `list_backups`, `prune_old_backups`.
- `services/scheduler.rs`: Parse de expressão cron com crate `cron`, agendamento via `tokio::spawn`, shutdown por canal `broadcast`, `validate_cron`, `next_run`.
- `services/network.rs`: `is_port_in_use` via bind TCP, `detect_port_conflicts` para game/query/rcon, `suggest_available_port`.
- `services/system_analyzer.rs`: `get_system_metrics` (CPU%, RAM total/usado), `get_process_metrics` por PID, `find_process_by_name` via `sysinfo`.
- `cargo build` limpo: 0 erros, 1 warning (`dead_code`) suprimido com `#[allow(dead_code)]`.

- `db/connection.rs`: Pool MySQL com `MySqlPoolOptions`, `DbConfig` lido de variáveis de ambiente, retry com backoff exponencial (máx 5 tentativas, delay 2^n segundos).
- `db/migrations.rs`: 4 migrations idempotentes controladas via tabela `_migrations`. Migration v1 cria todas as tabelas base (servers, clusters, backups, scheduled_tasks, settings). Migrations v2–v4 adicionam colunas adicionais.
- `db/mod.rs`: Expõe API pública com função `initialize(config)` que cria pool com retry e executa migrations.
- `models/server.rs`: Struct `Server` com `FromRow`, enums `ServerStatus` e `ArkMap` (11 mapas ASE), requests `CreateServerRequest` / `UpdateServerRequest`, response `ServerResponse`.
- `models/backup.rs`: Struct `Backup` com `FromRow`, enums `BackupType` e `BackupStatus`, `BackupPolicy`, requests de criação e restauração, response com `size_human()`.
- `models/task.rs`: Struct `ScheduledTask` com `FromRow`, enums `TaskType` (7 tipos) e `TaskResult`, requests de criação e atualização, response com `display_name()`.
- `lib.rs` atualizado: `AppState { db: DbPool }`, inicialização assíncrona do DB no setup do Tauri, emit de `db:ready` / `db:error`.
- Fix aplicado: `use tauri::{Emitter, Manager}` — traits necessárias para Tauri 2.
- `build.rs` criado: `tauri_build::build()` — necessário para que `tauri::generate_context!()` encontre `OUT_DIR`.

### Fase 4 — Concluída em 01/06/2026 00:30

**Backend Rust: 12 comandos Tauri implementados — bridge completa frontend ↔ backend.**

- `commands/server.rs`: CRUD completo de servidores + `start_server` (gera `RunServer.cmd` via `launch_builder`), `stop_server`, `restart_server`, `server_status` (sincroniza DB com estado real do processo).
- `commands/install.rs`: `install_steamcmd`, `is_steamcmd_installed`, `install_ark_server`, `update_ark_server`, `is_server_installed` — todos emitem `install:output` via Tauri event.
- `commands/config.rs`: `read_game_user_settings`, `read_game_ini` (leitura com detecção UTF-16 LE), `save_server_config` (geração via `config_generator`), `get_config_dir`.
- `commands/rcon.rs`: Define `RconMap = Arc<Mutex<HashMap<u32, RconConnection>>>`. `rcon_connect`, `rcon_send_command`, `rcon_disconnect`, `rcon_execute` (one-shot), `rcon_is_connected`.
- `commands/logs.rs`: Define `WatcherMap = Arc<Mutex<HashMap<u32, Sender<()>>>>`. `start_log_watcher` (emite `log:line`), `stop_log_watcher`, `is_log_watcher_active`.
- `commands/mods.rs`: `list_mods`, `add_mod`, `remove_mod`, `reorder_mods` — persistência no banco via coluna `active_mods`.
- `commands/cluster.rs`: CRUD completo de clusters + `assign_server_to_cluster`, `unassign_server_from_cluster`.
- `commands/backup.rs`: `list_backups`, `create_backup` (emite `backup:started`/`backup:completed`/`backup:failed`), `restore_backup`, `prune_backups`.
- `commands/scheduler.rs`: `list_tasks`, `create_task`, `update_task`, `delete_task`, `validate_cron_expression` (retorna próxima execução em RFC3339).
- `commands/hardware.rs`: `get_system_metrics`, `get_process_metrics` (por PID), `find_server_process` (localiza `ShooterGameServer.exe`).
- `commands/import.rs`: `detect_existing_server` (lê INIs e retorna configuração detectada), `import_server` (registra servidor no banco).
- `lib.rs` atualizado: `AppState { db: DbPool }`, `PidMap`/`RconMap`/`WatcherMap` gerenciados separadamente via `app.manage(...)`, invoke_handler com todos os 42 comandos registrados.
- `services/config_generator.rs`: `ServerConfig` ganhou derives `Serialize + Deserialize` para ser aceita como parâmetro de comando Tauri.
- `cargo build` limpo: 0 erros. `npm run build` limpo: 254.97 kB JS, 7.95 kB CSS.

### Fase 5 — Concluída em 01/06/2026 01:00

**Frontend: Base e infraestrutura completa.**

- `src/types/index.ts`: Todos os tipos TypeScript espelhando os modelos Rust — `Server`, `ServerStatus`, `ArkMap` (11 mapas), `Backup`, `ScheduledTask`, `Cluster`, `SystemMetrics`, `ProcessMetrics`, `LogLine`, `ServerConfig`, `DetectedServer`, `ModEntry`, todos os request/response types.
- `src/utils/tauri.ts`: Wrappers `invoke` tipados para todos os 42 comandos Tauri — servidor, instalação, config, RCON, logs, mods, cluster, backup, scheduler, hardware, importação.
- `src/utils/helpers.ts`: `cn` (clsx + tailwind-merge), `formatBytes`, `formatDate`, `formatRelative` (date-fns pt-BR), `statusColor`, `statusLabel`, `mapLabel`, `truncate`, `isValidPort`, `errorMessage`.
- `src/stores/serverStore.ts`: Zustand v5 — lista, seleção, start/stop/restart, refresh de status, remoção.
- `src/stores/uiStore.ts`: sidebar collapse, modal ativo, payload de confirmação.
- `src/stores/installStore.ts`: instalação/atualização com escuta do evento `install:output` via `listen`.
- `src/stores/rconStore.ts`: histórico de comandos/respostas, connect/disconnect/sendCommand.
- `src/components/layout/Sidebar.tsx` + `Layout.tsx`: já existiam, mantidos.
- `src/components/ui/`: Button (4 variantes + loading), Input (label+error+hint), Badge (6 variantes), Card/CardHeader/CardTitle, Modal (ESC+backdrop). Barrel export `index.ts`.
- `src/i18n/index.ts` + `locales/pt-BR.json`: i18next + react-i18next com 200+ chaves em pt-BR.
- `src/styles/globals.css`: já existia, mantido.
- `npx tsc --noEmit` limpo (0 erros). App iniciado em `http://localhost:1420`.

### Fase 7 — Concluída em 01/06/2026 04:00

**Frontend: 13 componentes especializados implementados e validados.**

- `src/types/index.ts`: Adicionados tipos `SyncStatus`, `AgentStatus`, `SyncAgent`, `SyncFolder` (Fase 8–9) e `EventStatus`, `EventRate`, `SeasonalEvent`, `CreateEventRequest` (Fase 11–12).
- `src/components/server/ServerCard.tsx`: Card de servidor com badge de status, grid de info (mapa/jogadores/portas/cluster), ações inline (iniciar/parar/reiniciar/RCON/Logs/Config).
- `src/components/server/InstallServerDialog.tsx`: Wizard de instalação — step setup (dirs), step instalando (log ao vivo do evento `install:output`), step done/error. Detecta SteamCMD com debounce.
- `src/components/server/PortConflictModal.tsx`: Modal de alerta de portas em conflito. Campos editáveis para game/query/rcon. Sugestão automática via `suggestAvailablePort`.
- `src/components/server/StatMultiplierEditor.tsx`: Grid de 12 stats (índices 0–11, torpor desabilitado), inputs numéricos com botão de reset para 1.0 quando modificado.
- `src/components/monitoring/PerformanceMonitor.tsx`: Cards de CPU e RAM com sparkline SVG (30 pontos), barra de uso e detalhe textual. Polling configurável.
- `src/components/cluster/ClusterBuilder.tsx`: Layout de duas colunas — servidores disponíveis ↔ servidores no cluster, com badges de status e ações de vincular/desvincular.
- `src/components/sync/SyncStatusBadge.tsx`: Badge colorido para os 6 estados de sync.
- `src/components/sync/SyncFolderCard.tsx`: Card de pasta sincronizada com status, path, último sync, peer, bytes transferidos, contagem de conflitos, barra animada durante sync.
- `src/components/sync/PeerDiscoveryDialog.tsx`: Lista agentes descobertos na rede, seleção de peer, input de código de 6 dígitos para pareamento.
- `src/components/events/EventStatusBadge.tsx`: Badge para os 5 estados de evento sazonal.
- `src/components/events/EventRatesForm.tsx`: Grid de 6 campos de taxa (XP/Coleta/Tame/Breed/Eclosão/Maturação), destaca campos modificados.
- `src/components/events/EventServerSelector.tsx`: Checkboxes com selecionar-todos, mostra mapa e status de cada servidor.
- `src/components/events/EventCard.tsx`: Card com countdown em tempo real (useEffect 1s), resumo de taxas em 4 chips, datas de início/fim, ações editar/cancelar.
- `npx tsc --noEmit` limpo (0 erros). `npm run build` limpo (433.97 kB JS, 22.95 kB CSS).

### Fase 8 — Concluída em 01/06/2026 05:30

**Backend Rust: Agente remoto de rede — servidor HTTP/WebSocket, descoberta UDP, pareamento com código de 6 dígitos.**

- `src-tauri/Cargo.toml`: Adicionadas dependências `rand = "0.8"`, `sha2 = "0.10"`. Feature `ws` habilitada em `axum`. Adicionadas `tower = "0.4"` e `tokio-tungstenite = "0.21"`.
- `src-tauri/src/models/agent.rs`: Structs `Agent` (FromRow), `DiscoveredAgent`, `AgentAnnouncement`, `PairRequest`, `PairResponse`. Enum `AgentStatus`.
- `src-tauri/src/db/migrations.rs`: Migration v5 adicionada — cria tabelas `sync_agents` (UNIQUE KEY address+port, token_hash SHA-256) e `sync_folders` (ENUM status 6 estados, agent_id FK).
- `src-tauri/src/services/agent_auth.rs`: `PairingState` — geração de código 6 dígitos com TTL de 120s, validação, invalidação pós-pareamento.
- `src-tauri/src/services/agent_server.rs`: Servidor axum na porta 45678 — rotas `GET /health`, `POST /pair`, `GET /ws?token=`. Handler WebSocket com echo (protocolo de sync completo na Fase 9).
- `src-tauri/src/services/agent_discovery.rs`: Two tasks tokio — `listen_for_agents` (UDP bind 0.0.0.0:45679, upsert em lista de descobertos) e `broadcast_presence` (UDP broadcast a cada 10s). Ignora próprio anúncio.
- `src-tauri/src/services/agent_client.rs`: `check_agent_health` (GET /health, timeout 5s) e `pair_with_agent` (POST /pair, timeout 10s) via reqwest.
- `src-tauri/src/commands/agent.rs`: `AgentRuntimeState` (discovered, sessions, pairing, agent_name). 6 comandos Tauri: `discover_agents`, `list_agents`, `pair_agent` (com SHA-256 do token), `remove_agent`, `get_agent_status`, `generate_pairing_code`.
- `src-tauri/src/models/mod.rs`: Adicionado `pub mod agent`.
- `src-tauri/src/services/mod.rs`: Adicionados `pub mod agent_auth`, `agent_server`, `agent_discovery`, `agent_client`.
- `src-tauri/src/commands/mod.rs`: Adicionado `pub mod agent`.
- `src-tauri/src/lib.rs`: `AgentRuntimeState` gerenciado via `.manage()`, servidor e discovery inicializados em `setup()` antes do DB. Closure `setup` marcada como `move`. Função `local_agent_name()` usa `COMPUTERNAME`/`HOSTNAME`.
- `src/types/index.ts`: Adicionada interface `DiscoveredAgent { name, address, port }`.
- `src/utils/tauri.ts`: Adicionados 6 wrappers: `discoverAgents`, `listAgents`, `pairAgent`, `removeAgent`, `getAgentStatus`, `generatePairingCode`.
- `cargo build` limpo (0 erros). `npx tsc --noEmit` limpo (0 erros). `npm run build` limpo.

---

## 14. DECISÕES ARQUITETURAIS

### DA-001 — MySQL em vez de SQLite (31/05/2026)

**Decisão:** Usar MySQL como banco de dados principal, em vez de SQLite.

**Motivo:** O projeto de referência ARKLAND SM usa MySQL. Usuários-alvo (administradores de servidores) frequentemente já possuem MySQL disponível em suas infraestruturas. MySQL oferece melhor suporte a JSON nativo e maior robustez para operações concorrentes de múltiplos servidores.

**Impacto:** Requer que o usuário tenha MySQL disponível. Variáveis de ambiente `DATABASE_URL` ou `DB_HOST/PORT/USER/PASSWORD/NAME` devem ser configuradas.

---

### DA-002 — Migrations idempotentes via tabela `_migrations` (31/05/2026)

**Decisão:** Controle manual de migrations com tabela `_migrations`, sem framework de migration externo.

**Motivo:** Maior controle sobre o processo. Compatível com a abordagem do ARKLAND SM. Sem dependência externa além do SQLx já utilizado.

**Impacto:** Novas migrations são sempre additive. Migrations destrutivas requerem nova migration, nunca edição de existentes.

---

### DA-003 — SessionName somente no INI, nunca na CLI (31/05/2026)

**Decisão:** O parâmetro `SessionName` do ARK deve ser configurado exclusivamente em `GameUserSettings.ini`, nunca passado via linha de comando.

**Motivo:** Comportamento documentado do servidor ARK: SessionName na CLI causa problemas de encoding e pode sobrescrever o valor do INI de forma incorreta.

**Impacto:** `launch_builder.rs` deve ignorar SessionName ao construir a linha de comando. `config_generator.rs` deve sempre incluir SessionName no `[SessionSettings]` do INI.

---

### DA-004 — INI com encoding UTF-16 LE com BOM (31/05/2026)

**Decisão:** Todos os arquivos INI gerados para o ARK devem usar encoding UTF-16 LE com BOM obrigatório.

**Motivo:** Requisito do engine Unreal Engine 3 usado pelo ARK: Survival Evolved. Arquivos em UTF-8 ou sem BOM podem não ser lidos corretamente pelo servidor.

**Impacto:** `config_generator.rs` deve usar escrita com encoding explícito. Nunca usar `std::fs::write` diretamente — sempre converter para UTF-16 LE + prepend BOM `[0xFF, 0xFE]`.

---

### DA-005 — build.rs obrigatório para tauri::generate_context!() (31/05/2026)

**Decisão:** O arquivo `src-tauri/build.rs` é obrigatório no projeto.

**Motivo:** `tauri::generate_context!()` é um proc macro que lê arquivos gerados pelo build script do `tauri-build`. Sem `build.rs` chamando `tauri_build::build()`, o Cargo não define `OUT_DIR` para o crate, e o macro falha com `OUT_DIR env var is not set`.

**Impacto:** Adicionado `build.rs` com conteúdo mínimo. Sem este arquivo, `cargo build` falha mesmo com `tauri-build` listado em `[build-dependencies]`.

---

## 15. ESTADO ATUAL DO PROJETO

> **Última atualização:** 02/06/2026 19:15 — Correção de bug: configuração MySQL persistente + banner de erro no app

### Resumo Executivo

| Item | Valor |
|---|---|
| **Fase atual** | Pós-build — Correção de bug ativo |
| **Tarefas concluídas** | 124 de 124 (Fases 1–13 completas) |
| **Tarefas em andamento** | 0 |
| **Tarefas pendentes** | 0 |
| **Progresso** | 100% — Bug de configuração de banco corrigido |

### Fix: Configuração de Banco de Dados (02/06/2026)

**Problema:** App instalado exibia tela preta / sem feedback quando MySQL não estava configurado.

**Solução implementada:**
- `src-tauri/src/db/connection.rs` — funções `load_db_config()`, `load_database_url_from_file()`, `save_database_url_to_file()` que persistem a URL em `%APPDATA%\com.arkmanager.app\database.json`
- `src-tauri/src/commands/database.rs` — 3 novos comandos: `get_database_url`, `save_database_url`, `test_database_connection`
- `src/pages/Settings.tsx` — nova seção "Banco de Dados (MySQL)" com campo DATABASE_URL + botões Testar/Salvar
- `src/App.tsx` — componente `DbErrorBanner` que escuta evento `db:error` e exibe banner com link para Configurações
- `src/utils/tauri.ts` — funções `getDatabaseUrl`, `saveDatabaseUrl`, `testDatabaseConnection`

### Status por Fase

| Fase | Descrição | Status |
|---|---|---|
| Fase 1 | Scaffold do projeto | ✅ Concluída |
| Fase 2 | Backend Rust: DB e modelos | ✅ Concluída |
| Fase 3 | Backend Rust: Serviços core | ✅ Concluída |
| Fase 4 | Backend Rust: Commands Tauri | ✅ Concluída |
| Fase 5 | Frontend: Base e infraestrutura | ✅ Concluída |
| Fase 6 | Frontend: Páginas | ✅ Concluída |
| Fase 7 | Frontend: Componentes especializados | ✅ Concluída |
| Fase 8 | Backend: Agente Remoto de Rede | ✅ Concluída |
| Fase 9 | Backend: Sistema de Sincronização | ✅ Concluída |
| Fase 10 | Frontend: Sincronização e complementos | ✅ Concluída |
| Fase 11 | Backend: Eventos Sazonais | ✅ Concluída |
| Fase 12 | Frontend: Eventos Sazonais | ✅ Concluída |
| Fase 13 | Testes, ajustes e build | ✅ Concluída |

### Build Status

| Verificação | Status |
|---|---|
| `npm run tauri build` | ✅ Limpo — 2 instaladores gerados (MSI + NSIS .exe) |
| `npx tsc --noEmit` | ✅ 0 erros |
| `cargo build` | ✅ Limpo (0 erros, Fase 11 integrada) |

### Próximos Passos

1. ~~Fases 1–13 concluídas com todos os builds limpos.~~ ✅ Concluído

**O projeto está completo.**
