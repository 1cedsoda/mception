# MCePtion
MCePtion is an MCP (Model Context Protocol) hotplugging system for developing distributed agents and MCP management.

## Vision
There can be an **Architect MCePtion Agent**, which has access to the **MCePtion Admin MCP** and can modify the global MCP configuration and add new MCPs. If it also has access to a coding agent and a Docker agent, it can create new agents and this bootraps the agentic, extensible AI system.

## MCePtion Server
The MCePtion server is the central component.

### Features:
- **MCP Management**: Manages MCP configurations.
- **Remote MCP Configuration**: Allows MCePtion Agents to download their remote MCP configurations.
- **MCePtion Agents**: Connected distributed agents which pull remote MCP configs.
- **MCePtion Admin MCP**: An included agent that allows other MCePtion agents to modify the global MCP configuration.
- **MCePtion Admin API**: An API to manage MCPs and MCePtion Agents which is synonymous with the MCePtion Admin MCP.
- **MCP Query Forwarding**: Forwards MCP query requests to other MCP servers.
- **Audit Logs**: A log of modifications done via the MCePtion Admin MCP.

### MCP Management
The MCePtion server manages

- A list of MCPs.
- A list of MCePtion Agents.
  - A list of allowed MCPs for each MCePtion Agent.

- It's important to understand that MCePtion Agents are also MCPs themselves.

This is persisted inside a JSON file, but could be extended to versioned git repositories or other storage systems.

### Remote MCP Configuration
Via the `/agent/<agent_id>/get_mcp_config` endpoint, MCePtion Agents can download their remote MCP configuration. This configuration is a JSON object that contains the MCPs and their configurations that the agent is allowed to use.

### MCePtion Agents
MCePtion agents are servers that can pull their remote MCP configuration from the MCePtion server. There is the MCePtion SDK which allows for remote MCP configuration download and MCP query forwarding via WebSockets.

### MCP Query Forwarding
Via the WebSocket URL `/agent/<agent_id>/mcp_query_forwarding_ws`, MCePtion agents can expose their MCP interface easily via the MCePtion server which simplifies the deployment of distributed agents, because this simplifies SSL certificate and URL management, because they are defined on just the MCePtion server.

## MCePtion Agent & SDK
The MCePtion Agent is a server which implements the MCePtion SDK/API. It usually contains a reasoning engine which can use certain (remote) non-agentic MCPs to accomplish a specialized task.

## Audit Logs

# MCePtion Admin MCP
This MCP is included in the MCePtion server and can be given to selected MCePtion Agents.
It's a way to CRUD (Create, Read, Update, Delete) MCPs and MCePtion Agents via the MCePtion server.

## Tools
### Create MCP
Add a new MCP configuration.

**Parameters:**
- `id`: The key of the MCP.
- `config`: The configuration of the MCP, which is a JSON object.
- `reason`: The reason for creating the MCP. This is important for logging and auditing purposes.
- `should_create`: (Has to be true) LLM safeguard variable.

### Read MCP
Read an existing MCP configuration.

**Parameters:**
- `id`: The key of the MCP to read.

**Response:**
- `config`: The configuration of the MCP, which is a JSON object.

### Read MCP Tools
Forwards the exposed tools by the MCP.

**Parameters:**
- `id`: The key of the MCP to read the tools from.

**Response:**
- `tools`: A list of tools that the MCP provides. Each tool is a JSON object with the following fields:
  - `name`: The name of the tool.
  - `description`: A description of the tool.
  - `parameters`: A JSON schema that describes the parameters of the tool.

### Update MCP
Update an existing MCP configuration.

**Parameters:**
- `id`: The key of the MCP to update.
- `config`: The new configuration of the MCP, which is a JSON object. Can also be a partial update, so only the fields that should be updated need to be provided.
- `reason`: The reason for reading the MCP. This is important for logging and auditing purposes.
- `should_update`: (Has to be true) LLM safeguard variable.

### Delete MCP
Delete an existing MCP configuration. This will also delete the ability of Mception Agents to use this MCP.

**Parameters:**
- `id`: The key of the MCP to delete.
- `reason`: The reason for deleting the MCP. This is important for logging and auditing purposes.
- `should_delete_mcp`: (Has to be true) LLM safeguard variable.

### Create MCePtion Agent
Adds a new MCePtion Agent.

**Parameters:**
- `agent_id`: The ID of the MCePtion Agent.
- `allowed_mcp_ids`: A list of MCP IDs that the MCePtion Agent is allowed to use.
- `should_create`: (Has to be true) LLM safeguard variable.

### Read MCePtion Agent
Read an existing MCePtion Agent configuration.

**Parameters:**
- `agent_id`: The ID of the MCePtion Agent to read.

**Response:**
- `allowed_mcp_ids`: A list of MCP capabilities that the MCePtion Agent is allowed to use.

### Read MCePtion Agent Tools
Forwards the exposed tools by the MCePtion Agent.

**Parameters:**
- `agent_id`: The ID of the MCePtion Agent to read the tools from.

**Response:**
- `tools`: A list of tools that the MCePtion Agent provides. Each tool is a JSON object with the following fields:
  - `name`: The name of the tool.
  - `description`: A description of the tool.
  - `parameters`: A JSON schema that describes the parameters of the tool.

### Update MCePtion Agent
Update an existing MCePtion Agent configuration.

**Parameters:**
- `agent_id`: The ID of the MCePtion Agent to update.
- `config`: The new configuration of the MCePtion Agent, which is a JSON object. Can also be a partial update, so only the fields that should be updated need to be provided.
- `reason`: The reason for updating the MCePtion Agent. This is important for logging and auditing purposes.
- `should_update`: (Has to be true) LLM safeguard variable.

### Add MCePtion Agent Allowed MCPs

**Parameters:**
- `agent_id`: The ID of the MCePtion Agent to update.
- `mcp_id`: The ID of the MCP (or MCePtion Agent) to add to the allowed MCPs list.
- `reason`: The reason for updating the allowed MCPs. This is important for logging and auditing purposes.
- `should_add_mcp_id`: (Has to be true) LLM safeguard variable.

### Remove MCePtion Agent Allowed MCPs

**Parameters:**
- `agent_id`: The ID of the MCePtion Agent to update.
- `mcp_id`: The ID of the MCP (or MCePtion Agent) to add to the allowed MCPs list.
- `reason`: The reason for updating the allowed MCPs. This is important for logging and auditing purposes.
- `should_remove_mcp_id`: (Has to be true) LLM safeguard variable.

### Delete MCePtion Agent
Delete an existing MCePtion Agent configuration. This will also delete the ability of the MCePtion Agent to use any MCPs.

**Parameters:**
- `agent_id`: The ID of the MCePtion Agent to delete.
- `reason`: The reason for deleting the MCePtion Agent. This is important for logging and auditing purposes.
- `should_delete_mcp`: (Has to be true) LLM safeguard variable.

# MCePtion Admin API
The MCePtion Admin API is a REST API that allows you to manage the MCP and MCePtion Agent configurations. It is synonymous with the MCePtion Admin MCP and provides the same functionality.

The `reason` and `should_*` parameters are omitted.
