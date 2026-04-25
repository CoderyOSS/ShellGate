## ADDED Requirements

### Requirement: MCP server tools for agent pre-approval requests
The gate-server SHALL expose MCP tools that AI agents can invoke to request pre-approvals and check approval status. These tools allow agents to proactively request the permissions they will need for upcoming work.

#### Scenario: Agent requests a pre-approval
- **WHEN** agent calls the `request_pre_approval` MCP tool with `{actions: ["pr:create", "comment:write"], repos: ["rm-rf-etc/Willow"], ttl: "2h", reason: "Reviewing PRs for deploy"}`
- **THEN** gate-server creates a pre-approval request (itself requiring human approval)
- **AND** sends a Telegram notification to the user
- **AND** returns `{status: "pending", request_id: "uuid"}` to the agent

#### Scenario: Pre-approval request approved by human
- **WHEN** the human approves the pre-approval request via Telegram or web UI
- **THEN** a grant is created for the requested actions/repos with the requested TTL
- **AND** subsequent commands matching the grant are auto-approved

#### Scenario: Pre-approval request rejected by human
- **WHEN** the human rejects the pre-approval request
- **THEN** no grant is created
- **AND** the agent receives `{status: "rejected", reason: "..."}` on next status check

### Requirement: Check approval status tool
Agents SHALL be able to check the status of their approval and pre-approval requests.

#### Scenario: Check pending approval status
- **WHEN** agent calls `get_approval_status` with `{id: "uuid"}`
- **THEN** returns `{status: "pending"}` or `{status: "approved"}` or `{status: "rejected"}`

### Requirement: List active grants tool
Agents SHALL be able to list currently active pre-approval grants that apply to them.

#### Scenario: List grants
- **WHEN** agent calls `list_grants`
- **THEN** returns a list of active grants with action patterns, repo patterns, TTL remaining, and use counts

### Requirement: Explain blocked commands tool
Agents SHALL be able to ask which commands are currently blocked and why.

#### Scenario: Explain what is blocked
- **WHEN** agent calls `explain_blocked`
- **THEN** returns a list of permission categories that are currently OFF (not covered by any grant)
- **AND** for each, shows whether a grant exists that partially covers it

### Requirement: MCP server runs as part of gate-server
The MCP server SHALL be integrated into the gate-server binary as a subcommand or built-in capability. It SHALL communicate over stdio (standard MCP protocol).

#### Scenario: Start MCP server
- **WHEN** gate-server is started with `--mcp` flag or `gate-server mcp` subcommand
- **THEN** it speaks the MCP protocol over stdio
- **AND** agents can connect via their MCP client configuration
