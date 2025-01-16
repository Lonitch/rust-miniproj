# Model Context Protocol (MCP) Tutorial

## Introduction to MCP

The Model Context Protocol (MCP) is a standardized protocol that enables seamless integration of AI capabilities across applications while maintaining clear security boundaries and isolating concerns. Built on JSON-RPC, MCP provides a stateful session protocol focused on context exchange and sampling coordination between clients and servers.

## Core Architecture

### Client-Host-Server Architecture

MCP follows a client-host-server architecture where each host can run multiple client instances. Here's a breakdown of the key components:

#### Host Process
The host acts as the container and coordinator with the following responsibilities:
- Creates and manages multiple client instances
- Controls client connection permissions and lifecycle
- Enforces security policies and consent requirements
- Handles user authorization decisions
- Coordinates AI/LLM integration and sampling
- Manages context aggregation across clients

```rust
// Example host implementation in Rust
pub struct McpHost {
    clients: HashMap<ClientId, Client>,
    security_policy: SecurityPolicy,
    auth_manager: AuthorizationManager,
}

impl McpHost {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            security_policy: SecurityPolicy::default(),
            auth_manager: AuthorizationManager::new(),
        }
    }

    pub fn create_client(&mut self) -> Result<ClientId, Error> {
        let client = Client::new();
        let client_id = ClientId::generate();
        self.clients.insert(client_id, client);
        Ok(client_id)
    }

    pub fn authorize_connection(&self, client_id: ClientId, server: &Server) -> Result<(), Error> {
        self.auth_manager.check_permissions(client_id, server)?;
        Ok(())
    }
}
```

#### Clients
Each client maintains an isolated server connection with these responsibilities:
- Establishes one stateful session per server
- Handles protocol negotiation and capability exchange
- Routes protocol messages bidirectionally
- Manages subscriptions and notifications
- Maintains security boundaries between servers

```rust
pub struct Client {
    server_connection: Option<ServerConnection>,
    capabilities: ClientCapabilities,
    session_state: SessionState,
}

impl Client {
    pub fn connect_to_server(&mut self, server: Server) -> Result<(), Error> {
        let connection = ServerConnection::establish(server)?;
        self.negotiate_capabilities(&connection)?;
        self.server_connection = Some(connection);
        Ok(())
    }

    fn negotiate_capabilities(&mut self, conn: &ServerConnection) -> Result<(), Error> {
        let server_caps = conn.get_capabilities()?;
        self.capabilities.negotiate(server_caps)?;
        Ok(())
    }
}
```

#### Servers
Servers provide specialized context and capabilities:
- Expose resources, tools and prompts via MCP primitives
- Operate independently with focused responsibilities
- Request sampling through client interfaces
- Must respect security constraints
- Can be local processes or remote services

```rust
pub struct Server {
    capabilities: ServerCapabilities,
    resources: ResourceManager,
    tools: ToolManager,
    prompts: PromptManager,
}

impl Server {
    pub fn new() -> Self {
        Self {
            capabilities: ServerCapabilities::default(),
            resources: ResourceManager::new(),
            tools: ToolManager::new(),
            prompts: PromptManager::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Tool) {
        self.tools.register(tool);
    }

    pub fn expose_resource(&mut self, resource: Resource) {
        self.resources.add(resource);
    }
}
```

## Server Features

MCP provides three fundamental primitives that servers can expose to enable rich interactions between clients and language models:

### 1. Prompts
Pre-defined templates or instructions that guide language model interactions:
- User-controlled through UI elements like slash commands
- Support parameterization for customization
- Can include text and images
- May reference other MCP resources

```rust
pub struct Prompt {
    name: String,
    description: Option<String>,
    arguments: Vec<PromptArgument>,
    messages: Vec<PromptMessage>,
}

pub struct PromptArgument {
    name: String,
    description: Option<String>,
    required: bool,
}

pub enum PromptMessage {
    Text {
        role: Role,
        content: String,
    },
    Image {
        role: Role,
        data: Vec<u8>,
        mime_type: String,
    },
    Resource {
        role: Role,
        uri: String,
    },
}

impl Prompt {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            arguments: Vec::new(),
            messages: Vec::new(),
        }
    }

    pub fn add_argument(&mut self, name: &str, required: bool) {
        self.arguments.push(PromptArgument {
            name: name.to_string(),
            description: None,
            required,
        });
    }
}
```

### 2. Resources
Structured data or content that provides context to the model:
- Application-controlled data sources
- Can be static or dynamic
- Support subscriptions for updates
- Identified by URIs

```rust
pub struct Resource {
    uri: String,
    name: String,
    description: Option<String>,
    mime_type: Option<String>,
    content: ResourceContent,
}

pub enum ResourceContent {
    Text(String),
    Binary(Vec<u8>),
}

pub struct ResourceManager {
    resources: HashMap<String, Resource>,
    subscribers: HashMap<String, Vec<Subscriber>>,
}

impl ResourceManager {
    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.insert(resource.uri.clone(), resource);
    }

    pub fn subscribe(&mut self, uri: &str, subscriber: Subscriber) -> Result<(), Error> {
        self.subscribers
            .entry(uri.to_string())
            .or_default()
            .push(subscriber);
        Ok(())
    }

    pub fn update_resource(&mut self, uri: &str, content: ResourceContent) -> Result<(), Error> {
        if let Some(resource) = self.resources.get_mut(uri) {
            resource.content = content;
            self.notify_subscribers(uri)?;
        }
        Ok(())
    }
}
```

### 3. Tools
Executable functions that allow models to perform actions:
- Model-controlled through function calling
- Support input validation via JSON Schema
- Can return multiple content types
- May require user confirmation

```rust
pub struct Tool {
    name: String,
    description: Option<String>,
    input_schema: Schema,
    handler: Box<dyn ToolHandler>,
}

pub trait ToolHandler: Send + Sync {
    fn handle(&self, args: Value) -> Result<ToolResult, Error>;
}

pub struct ToolResult {
    content: Vec<ToolContent>,
    is_error: bool,
}

pub enum ToolContent {
    Text(String),
    Image {
        data: Vec<u8>,
        mime_type: String,
    },
    Resource {
        uri: String,
        content: ResourceContent,
    },
}

impl Tool {
    pub fn new(name: &str, schema: Schema, handler: Box<dyn ToolHandler>) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            input_schema: schema,
            handler,
        }
    }

    pub async fn call(&self, args: Value) -> Result<ToolResult, Error> {
        // Validate arguments against schema
        self.input_schema.validate(&args)?;
        
        // Execute tool handler
        self.handler.handle(args)
    }
}
```

Each primitive serves a specific purpose in the MCP ecosystem:
- Prompts: User-controlled templates for model interaction
- Resources: Application-controlled context providers
- Tools: Model-controlled function execution

This separation of concerns allows for clear boundaries while enabling rich interactions between clients, servers, and language models.

## Client Features

MCP defines two main client-side features that enable filesystem access and language model interaction:

### 1. Roots
Filesystem "roots" that define server access boundaries:
- Exposes workspace/project directories
- Controls server filesystem access
- Supports dynamic updates
- Uses file:// URIs

```rust
pub struct Root {
    uri: String,
    name: Option<String>,
}

pub struct RootManager {
    roots: Vec<Root>,
    subscribers: Vec<RootSubscriber>,
}

impl RootManager {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            subscribers: Vec::new(),
        }
    }

    pub fn add_root(&mut self, uri: &str, name: Option<String>) -> Result<(), Error> {
        // Validate URI format
        if !uri.starts_with("file://") {
            return Err(Error::InvalidUri);
        }

        self.roots.push(Root {
            uri: uri.to_string(),
            name,
        });

        // Notify subscribers
        self.notify_root_change()?;
        Ok(())
    }

    pub fn validate_path(&self, path: &Path) -> Result<(), Error> {
        // Check if path is within any root
        for root in &self.roots {
            if path.starts_with(root.uri.strip_prefix("file://").unwrap()) {
                return Ok(());
            }
        }
        Err(Error::PathOutsideRoots)
    }
}
```

### 2. Sampling
Language model interaction capabilities:
- Handles model selection
- Manages sampling parameters
- Supports text and image inputs
- Enables user review/approval

```rust
pub struct SamplingManager {
    model_registry: ModelRegistry,
    user_consent: Box<dyn UserConsentProvider>,
}

pub struct SamplingRequest {
    messages: Vec<Message>,
    model_preferences: ModelPreferences,
    system_prompt: Option<String>,
    max_tokens: Option<u32>,
}

pub struct ModelPreferences {
    hints: Vec<ModelHint>,
    cost_priority: f32,
    speed_priority: f32,
    intelligence_priority: f32,
}

impl SamplingManager {
    pub async fn create_message(&self, request: SamplingRequest) -> Result<Message, Error> {
        // Select appropriate model
        let model = self.select_model(&request.model_preferences)?;

        // Get user consent if needed
        if self.user_consent.requires_approval(&request) {
            self.user_consent.request_approval(&request).await?;
        }

        // Generate response
        let response = model.generate(&request).await?;

        // Allow user to review if needed
        if self.user_consent.requires_review(&response) {
            self.user_consent.request_review(&response).await?;
        }

        Ok(response)
    }

    fn select_model(&self, prefs: &ModelPreferences) -> Result<Box<dyn Model>, Error> {
        // Try model hints first
        for hint in &prefs.hints {
            if let Some(model) = self.model_registry.find_by_hint(hint) {
                return Ok(model);
            }
        }

        // Fall back to capability-based selection
        self.model_registry.find_by_capabilities(
            prefs.cost_priority,
            prefs.speed_priority,
            prefs.intelligence_priority,
        )
    }
}
```

## Design Principles

MCP is built on several key design principles:

1. **Easy Server Implementation**
   - Host applications handle complex orchestration
   - Servers focus on specific capabilities
   - Simple interfaces minimize overhead
   - Clear separation enables maintainability

2. **High Composability**
   - Each server provides focused functionality
   - Multiple servers can be combined seamlessly
   - Shared protocol enables interoperability
   - Modular design supports extensibility

3. **Security Isolation**
   - Servers receive only necessary context
   - Full conversation history stays with host
   - Each server connection maintains isolation
   - Cross-server interactions controlled by host
   - Host process enforces security boundaries

4. **Progressive Enhancement**
   - Core protocol provides minimal functionality
   - Additional capabilities negotiated as needed
   - Servers and clients evolve independently
   - Protocol designed for future extensibility
   - Backwards compatibility maintained

## Message Types

MCP defines three core message types based on JSON-RPC 2.0:

### 1. Requests
Bidirectional messages expecting a response:
```rust
#[derive(Serialize, Deserialize)]
pub struct Request {
    jsonrpc: String,  // Must be "2.0"
    id: RequestId,    // String or number, must not be null
    method: String,
    params: Option<Value>,
}
```

### 2. Responses
Replies to specific requests:
```rust
#[derive(Serialize, Deserialize)]
pub struct Response {
    jsonrpc: String,
    id: RequestId,
    result: Option<Value>,
    error: Option<ErrorResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    code: i32,
    message: String,
    data: Option<Value>,
}
```

### 3. Notifications
One-way messages requiring no response:
```rust
#[derive(Serialize, Deserialize)]
pub struct Notification {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
}
```

## Capability Negotiation

MCP uses capability-based negotiation where clients and servers declare supported features during initialization:

```rust
pub struct Capabilities {
    resources: ResourceCapabilities,
    tools: ToolCapabilities,
    prompts: PromptCapabilities,
}

impl Capabilities {
    pub fn negotiate(&self, other: &Capabilities) -> NegotiatedCapabilities {
        // Determine intersection of supported features
        NegotiatedCapabilities {
            resources: self.resources.negotiate(&other.resources),
            tools: self.tools.negotiate(&other.tools),
            prompts: self.prompts.negotiate(&other.prompts),
        }
    }
}
```

This capability system ensures clients and servers have a clear understanding of supported functionality while maintaining protocol extensibility.

## Transport Mechanisms

MCP supports multiple transport mechanisms for client-server communication:

### 1. stdio Transport
Communication over standard input and output streams:
- Client launches server as subprocess
- Server receives JSON-RPC messages on stdin
- Server writes responses to stdout
- Messages are delimited by newlines
- Optional logging on stderr

```rust
pub struct StdioTransport {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    stderr: BufReader<ChildStderr>,
}

impl StdioTransport {
    pub fn new(command: &str, args: &[String]) -> Result<Self, Error> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or(Error::NoStdin)?;
        let stdout = BufReader::new(child.stdout.take().ok_or(Error::NoStdout)?);
        let stderr = BufReader::new(child.stderr.take().ok_or(Error::NoStderr)?);

        Ok(Self {
            process: child,
            stdin,
            stdout,
            stderr,
        })
    }

    pub async fn send_message(&mut self, message: &str) -> Result<(), Error> {
        self.stdin.write_all(message.as_bytes())?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;
        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<String, Error> {
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        Ok(line)
    }
}
```

### 2. HTTP with Server-Sent Events (SSE)
A transport mechanism where:
- Server operates as independent process
- Handles multiple client connections
- Uses SSE for server-to-client messages
- Uses HTTP POST for client-to-server messages

```rust
pub struct HttpTransport {
    sse_client: EventSource,
    http_client: Client,
    endpoint: String,
}

impl HttpTransport {
    pub async fn new(server_url: &str) -> Result<Self, Error> {
        // Connect to SSE endpoint
        let sse_client = EventSource::new(server_url)?;
        
        // Wait for endpoint event with POST URL
        let endpoint = Self::wait_for_endpoint(&sse_client).await?;
        
        Ok(Self {
            sse_client,
            http_client: Client::new(),
            endpoint,
        })
    }

    pub async fn send_message(&self, message: &str) -> Result<(), Error> {
        self.http_client
            .post(&self.endpoint)
            .json(&message)
            .send()
            .await?;
        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<String, Error> {
        while let Some(event) = self.sse_client.next().await {
            match event {
                Ok(Event::Message(message)) => return Ok(message.data),
                Ok(_) => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Err(Error::ConnectionClosed)
    }
}
```

### 3. Custom Transports
The protocol supports custom transport implementations:

```rust
pub trait Transport {
    fn transport_type(&self) -> TransportType;
    async fn send_message(&mut self, message: &str) -> Result<(), Error>;
    async fn receive_message(&mut self) -> Result<String, Error>;
    async fn close(&mut self) -> Result<(), Error>;
}

// Example custom transport
pub struct CustomTransport {
    // Custom implementation details
}

impl Transport for CustomTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::Custom("my-transport")
    }

    async fn send_message(&mut self, message: &str) -> Result<(), Error> {
        // Custom send implementation
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<String, Error> {
        // Custom receive implementation
        Ok(String::new())
    }

    async fn close(&mut self) -> Result<(), Error> {
        // Custom cleanup
        Ok(())
    }
}
```

All transport implementations must:
- Preserve JSON-RPC message format
- Handle connection establishment
- Support message exchange patterns
- Implement proper cleanup on close

## Protocol Versioning

MCP uses a date-based versioning scheme with format `YYYY-MM-DD`, indicating when backwards incompatible changes were last made. The current version is **2024-11-05**.

Key versioning concepts:

1. Version Negotiation
- Happens during initialization
- Clients and servers can support multiple versions
- Must agree on single version for session

```rust
pub struct VersionNegotiator {
    supported_versions: Vec<String>,
}

impl VersionNegotiator {
    pub fn new(versions: Vec<String>) -> Self {
        Self {
            supported_versions: versions,
        }
    }

    pub fn negotiate(&self, client_version: &str) -> Result<String, VersionError> {
        if self.supported_versions.contains(&client_version.to_string()) {
            // Server supports client's requested version
            Ok(client_version.to_string())
        } else {
            // Return server's latest supported version
            self.supported_versions
                .last()
                .cloned()
                .ok_or(VersionError::NoVersionsSupported)
        }
    }
}

// Example version negotiation
let negotiator = VersionNegotiator::new(vec![
    "2024-11-05".to_string(),
    "2024-01-01".to_string(),
]);

// Client requests latest version
match negotiator.negotiate("2024-11-05") {
    Ok(version) => println!("Using protocol version: {}", version),
    Err(e) => println!("Version negotiation failed: {:?}", e),
}

// Client requests unsupported version
match negotiator.negotiate("1.0.0") {
    Ok(version) => println!("Using fallback version: {}", version),
    Err(e) => println!("Version negotiation failed: {:?}", e),
}
```

2. Version Compatibility
- Protocol version only increments for breaking changes
- Backwards compatible changes don't require version bump
- Clients should use latest supported version
- Servers should support multiple versions when possible

```rust
#[derive(Debug)]
pub enum VersionError {
    NoVersionsSupported,
    IncompatibleVersion {
        requested: String,
        supported: Vec<String>,
    },
}

impl Server {
    pub fn check_version_compatibility(&self, client_version: &str) -> Result<(), VersionError> {
        // Check if requested version is supported
        if !self.supported_versions.contains(&client_version.to_string()) {
            return Err(VersionError::IncompatibleVersion {
                requested: client_version.to_string(),
                supported: self.supported_versions.clone(),
            });
        }
        Ok(())
    }
}
```
