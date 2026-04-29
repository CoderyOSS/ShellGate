## ADDED Requirements

### Requirement: Web dashboard for approval management
The web UI SHALL display pending approval requests with real-time updates. When a new approval request is created, it SHALL appear in the dashboard immediately without page refresh.

#### Scenario: View pending approvals
- **WHEN** user navigates to the dashboard
- **THEN** pending approval requests are displayed showing: command, action, repo, timestamp, agent
- **AND** each request has Approve and Reject buttons

#### Scenario: Real-time update on new request
- **WHEN** a new approval request is created while the dashboard is open
- **THEN** the new request appears in the list without page refresh (via SSE at `/api/events`)
- **AND** SSE events include: `approval.new`, `approval.resolved`, `grant.created`, `grant.expired`
- **AND** reconnection uses `Last-Event-Id` header to resume from last received event
- **AND** a 30-second keepalive heartbeat prevents proxy timeout

#### Scenario: Approve from dashboard
- **WHEN** user clicks "Approve" on a pending request
- **THEN** the request status changes to approved
- **AND** the UI updates to show "Approved" status
- **AND** the waiting patched zsh receives the allow response

### Requirement: Pre-approval management
The web UI SHALL provide a form for creating new pre-approval grants and a list of active grants with the ability to revoke them.

#### Scenario: Create new pre-approval
- **WHEN** user fills in the grant form with action pattern, repo pattern, TTL, and reason
- **AND** clicks "Create"
- **THEN** the grant is created and appears in the active grants list
- **AND** matching commands are auto-approved until expiry

#### Scenario: View active grants
- **WHEN** user navigates to the grants section
- **THEN** active grants are displayed with: action, repo, TTL remaining, use count, reason

#### Scenario: Revoke a grant
- **WHEN** user clicks "Revoke" on an active grant
- **THEN** the grant is immediately disabled
- **AND** subsequent matching commands require fresh approval

### Requirement: Audit log viewer
The web UI SHALL display the audit log with filtering and search capabilities.

#### Scenario: View audit log
- **WHEN** user navigates to the audit section
- **THEN** recent audit entries are displayed showing: timestamp, command, action, repo, granted_by, exit_code, operator_id, operator_type

#### Scenario: Filter audit log
- **WHEN** user filters by action type (e.g., `git:push`)
- **THEN** only entries matching that action are shown

#### Scenario: Search audit log
- **WHEN** user searches for text in commands or repos
- **THEN** matching entries are displayed

### Requirement: Permission configuration editor
The web UI SHALL provide an interface for viewing and modifying default permission states.

#### Scenario: View permission defaults
- **WHEN** user navigates to the permissions section
- **THEN** all default permissions are displayed with their current state (ON/OFF) and TTL

#### Scenario: Toggle a permission
- **WHEN** user toggles `pr:create` from OFF to ON
- **THEN** the default state is updated
- **AND** subsequent `pr:create` commands are auto-approved

#### Scenario: Reset to defaults
- **WHEN** user clicks "Reset to Defaults"
- **THEN** all permissions are restored to their initial seed values
- **AND** a confirmation prompt appears before executing

### Requirement: Static build with Astro and React
The web UI SHALL be built as a static site using Astro with React components. The built files SHALL be served by gate-server at `/` on port 3000.

#### Scenario: Build and deploy
- **WHEN** the Astro project is built (`npm run build`)
- **THEN** static HTML/CSS/JS files are generated in a `dist/` directory
- **AND** these files can be served by gate-server's static file handler
