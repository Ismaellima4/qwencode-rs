# Especificações do Projeto: qwencode-rs

## 📋 Visão Geral

SDK Rust para acesso programático ao QwenCode CLI, baseado no SDK TypeScript oficial (`@qwen-code/sdk`).

### Objetivos
- Fornecer API assíncrona idiomaticamente Rust (async/await com Tokio)
- Manter paridade funcional com o SDK TypeScript
- Aproveitar o sistema de tipos forte do Rust para segurança em compile-time
- Suportar comunicação via stdin/stdout com o CLI do QwenCode
- Integrar com servidores MCP (Model Context Protocol)

## 🏗️ Arquitetura

### Estrutura de Módulos

```
src/
├── lib.rs              # API pública principal, re-exports
├── types/              # Definições de tipos e estruturas
│   ├── mod.rs
│   ├── message.rs      # Tipos de mensagens (User, Assistant, System, Result)
│   ├── config.rs       # QueryOptions e configurações
│   ├── error.rs        # Tipos de erro (AbortError, SDKError)
│   ├── permission.rs   # Modos de permissão e CanUseTool
│   └── mcp.rs          # Tipos relacionados a MCP
├── transport/          # Camada de comunicação
│   ├── mod.rs
│   ├── stdin.rs        # Comunicação via stdin/stdout
│   ├── stream.rs       # Stream de mensagens
│   └── protocol.rs     # Protocolo de comunicação
├── query/              # Lógica principal de query
│   ├── mod.rs
│   ├── session.rs      # Gerenciamento de sessão
│   ├── builder.rs      # Query builder pattern
│   └── handler.rs      # Handlers de mensagens
├── mcp/                # Suporte a MCP servers
│   ├── mod.rs
│   ├── server.rs       # MCP Server embutido
│   ├── tool.rs         # Definição de ferramentas
│   └── client.rs       # MCP Client
└── utils/              # Utilitários
    ├── mod.rs
    ├── validation.rs   # Validações
    └── helpers.rs      # Funções auxiliares
```

## 📦 Dependências Principais

### Produçāo
```toml
[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
tokio-stream = "0.1"

# Serializaçāo
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# MCP SDK
reqwest = { version = "0.11", features = ["json", "stream"] }

# Validaçāo/Schema
schemars = "0.8"  # Equivalente ao Zod em Rust

# Erros
thiserror = "1"
anyhow = "1"

# Logging/Debug
tracing = "0.1"

# UUID para session IDs
uuid = { version = "1", features = ["v4"] }

# Channels para comunicaçāo
async-channel = "2"

# Timeout
tokio-util = { version = "0.7", features = ["time"] }
```

### Desenvolvimento
```toml
[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
assert_fs = "1"
```

## 🔧 API Design

### Funçāo Principal: `query()`

```rust
use qwencode_rs::{query, QueryOptions, SDKMessage};

let options = QueryOptions::builder()
    .cwd("/path/to/project")
    .model("qwen-max")
    .permission_mode(PermissionMode::Default)
    .build()?;

let mut result = query("What files are in the current directory?", options).await?;

while let Some(message) = result.next().await {
    match message {
        SDKMessage::Assistant(msg) => println!("Assistant: {}", msg.content),
        SDKMessage::Result(msg) => println!("Result: {:?}", msg.result),
        _ => {}
    }
}
```

### Tipos de Mensagem

```rust
pub enum SDKMessage {
    User(SDKUserMessage),
    Assistant(SDKAssistantMessage),
    System(SDKSystemMessage),
    Result(SDKResultMessage),
    PartialAssistant(SDKPartialAssistantMessage),
}

// Type guards via pattern matching idiomatic Rust
```

### QueryOptions

```rust
#[derive(Debug, Clone, Builder)]
pub struct QueryOptions {
    pub cwd: Option<PathBuf>,
    pub model: Option<String>,
    pub path_to_qwen_executable: Option<String>,
    pub permission_mode: PermissionMode,
    pub can_use_tool: Option<CanUseToolCallback>,
    pub env: Option<HashMap<String, String>>,
    pub system_prompt: Option<SystemPromptConfig>,
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
    pub abort_signal: Option<tokio_util::sync::CancellationToken>,
    pub debug: bool,
    pub max_session_turns: Option<i32>,
    pub core_tools: Option<Vec<String>>,
    pub exclude_tools: Option<Vec<String>>,
    pub allowed_tools: Option<Vec<String>>,
    pub auth_type: AuthType,
    pub agents: Option<Vec<SubagentConfig>>,
    pub include_partial_messages: bool,
    pub resume: Option<String>,
    pub session_id: Option<String>,
    pub timeouts: Option<TimeoutConfig>,
}
```

### PermissionMode

```rust
pub enum PermissionMode {
    Default,    // Leitura automática, escrita requer aprovaçāo
    Plan,       // Apenas plano, bloqueia escrita
    AutoEdit,   // Aprova automaticamente ediçāo
    Yolo,       // Aprova tudo automaticamente
}
```

### Métodos do Query Handle

```rust
pub struct QueryHandle {
    // campos internos
}

impl QueryHandle {
    pub fn session_id(&self) -> &str;
    pub fn is_closed(&self) -> bool;
    pub async fn interrupt(&self) -> Result<()>;
    pub async fn set_permission_mode(&self, mode: PermissionMode) -> Result<()>;
    pub async fn set_model(&self, model: String) -> Result<()>;
    pub async fn close(self) -> Result<()>;
}
```

### MCP Tool Definition

```rust
use qwencode_rs::{tool, create_sdk_mcp_server};

let calc_tool = tool!(
    "calculate_sum",
    "Add two numbers",
    |args: CalcArgs| async move {
        MCPToolResult {
            content: vec![ToolContent::Text {
                text: format!("{}", args.a + args.b),
            }],
        }
    }
);

let server = create_sdk_mcp_server("calculator", vec![calc_tool]);
```

### Abort com CancellationToken

```rust
use tokio_util::sync::CancellationToken;

let cancel_token = CancellationToken::new();
let cancel_token_clone = cancel_token.clone();

tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(5)).await;
    cancel_token_clone.cancel();
});

let result = query("Long task...", options).await;
// CancellationToken::cancelled() indica abort
```

## 🎯 Diferenciaçāo TypeScript → Rust

| TypeScript | Rust |
|------------|------|
| `AbortController` | `tokio_util::sync::CancellationToken` |
| `AsyncIterable<T>` | `impl Stream<Item = T>` ou `AsyncIterator` |
| `Promise<T>` | `Future<Output = T>` |
| `zod` schemas | `schemars` + validaçāo manual |
| Type guards | Pattern matching (`match`) |
| `Record<string, T>` | `HashMap<String, T>` |
| Callbacks | Closures + `Box<dyn Fn...>` |
| `export { ... }` | `pub use ...` |

## ⏱️ Timeouts Padrāo

| Timeout | Padrāo | Descriçāo |
|---------|--------|-----------|
| `can_use_tool` | 60s | Tempo para callback de permissāo |
| `mcp_request` | 60s | Chamadas de ferramentas MCP |
| `control_request` | 60s | `initialize()`, `set_model()`, etc |
| `stream_close` | 15s | Fechar stdin em modo multi-turn |

## 🔐 Modos de Permissāo

### Cadeia de Prioridade
```
exclude_tools/deny 
  > ask 
  > plan 
  > yolo 
  > allowed_tools/allow 
  > can_use_tool callback 
  > comportamento padrão
```

## 🚀 Exemplos de Uso

### Exemplo 1: Single-turn simples
```rust
let result = query("Create a hello.txt file", QueryOptions::default()).await?;
while let Some(msg) = result.next().await {
    if let SDKMessage::Assistant(a) = msg {
        println!("{}", a.content);
    }
}
```

### Exemplo 2: Multi-turn com Stream
```rust
async fn message_stream() -> impl Stream<Item = SDKUserMessage> {
    let (tx, rx) = async_channel::unbounded();
    
    tx.send(SDKUserMessage { content: "Create hello.txt".into(), ..Default::default() }).await.unwrap();
    // ... mais mensagens
    
    rx
}

let mut result = query_stream(message_stream().await, options).await?;
```

### Exemplo 3: Custom Tool Handler
```rust
async fn can_use_tool(
    tool_name: &str,
    input: &serde_json::Value,
) -> ToolPermissionResult {
    if tool_name.starts_with("read_") {
        return ToolPermissionResult::Allow { input: input.clone() };
    }
    
    // Lógica customizada de aprovaçāo
    ToolPermissionResult::Deny { message: "Denied".into() }
}

let options = QueryOptions::builder()
    .can_use_tool(can_use_tool)
    .build()?;
```

### Exemplo 4: MCP Server Embutido
```rust
let server = create_sdk_mcp_server(McpServerOptions {
    name: "calculator".into(),
    tools: vec![calc_tool],
});

let options = QueryOptions::builder()
    .permission_mode(PermissionMode::Yolo)
    .mcp_server("calculator", server)
    .build()?;
```

## 🧪 Estratégia de Testes

### Testes Unitários
- Validaçāo de tipos e configuraçāo
- Protocolo de comunicaçāo
- MCP tool definitions
- Permission handling logic

### Testes de Integraçāo
- Comunicaçāo stdin/stdout com CLI mock
- MCP server integraçāo
- Session management
- Abort/cancellation

### Testes de Exemplo
- Todos os exemplos na docs devem compilar e executar
- Usar `#[doc_test]` ou `doctest`

## 📚 Documentaçāo

### README.md deve incluir:
1. Instalaçāo (Cargo.toml dependency)
2. Quick Start
3. Referencia da API
4. Exemplos práticos
5. Tratamento de erros
6. FAQ

### Rustdoc
- Documentar todos os tipos e funçāo públicas
- Incluir exemplos em cada tipo/funçāo
- Links para documentaçāo relacionada

## 🔄 Compatibilidade

### Versōes Mínimas
- Rust: 1.75+ (async features estilizadas)
- Tokio: 1.x
- MSRV (Minimum Supported Rust Version): a definir

### Plataformas
- Linux
- macOS
- Windows (msvc/gnu)

## 📊 Métricas de Sucesso

- [ ] Paridade funcional com SDK TypeScript v0.1.6
- [ ] Todos os exemplos do TypeScript convertidos para Rust
- [ ] Cobertura de testes > 80%
- [ ] Documentaçāo completa com exemplos
- [ ] Build sem warnings em clippy
- [ ] Benchmark de performance vs TypeScript

## 🗓️ Próximos Passos

1. ✅ Especificaçāo do projeto (este documento)
2. Configurar estrutura do Cargo.toml com dependências
3. Criar estrutura de módulos (types, transport, query, mcp, utils)
4. Implementar tipos principais
5. Implementar camada de transporte
6. Implementar query engine
7. Implementar MCP support
8. Adicionar testes
9. Criar documentaçāo
10. Publicar no crates.io
