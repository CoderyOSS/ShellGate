## ADDED Requirements

### Requirement: Default permission states
Every permission SHALL have a default state (ON or OFF) and a default TTL. The system SHALL seed the following defaults on first initialization:

| Permission | Default State | Default TTL |
|------------|--------------|-------------|
| repo:read | ON | indefinite |
| pr:read | ON | indefinite |
| pr:create | OFF | - |
| pr:update | OFF | - |
| pr:merge | OFF | - |
| issue:read | ON | indefinite |
| issue:create | OFF | - |
| issue:update | OFF | - |
| comment:write | OFF | - |
| workflow:read | ON | indefinite |
| workflow:run | OFF | - |
| git:read | ON | indefinite |
| git:push | OFF | - |
| api:read | ON | indefinite |
| api:write | OFF | - |
| repo:create | OFF | - |
| repo:delete | OFF | - |

The `auth:*` permission SHALL be permanently BLOCKED and unchangeable — commands like `gh auth login` SHALL never be allowed.

#### Scenario: Default read permission active
- **WHEN** a `git fetch` command is checked
- **AND** no grant exists for `git:read`
- **THEN** the default permission `git:read` (ON, indefinite) is used
- **AND** the command is allowed

#### Scenario: Default write permission inactive
- **WHEN** a `git push` command is checked
- **AND** no grant exists for `git:push`
- **THEN** the default permission `git:push` (OFF) is used
- **AND** an approval request is created

#### Scenario: Auth command permanently blocked
- **WHEN** a `gh auth login` command is checked
- **THEN** the command is immediately rejected regardless of grants or defaults
- **AND** the rejection reason states "auth commands are permanently blocked"

### Requirement: Unified permission check
The system SHALL use a single code path for all permission checks. The check SHALL:
1. Return BLOCKED if the action is in the permanently blocked list
2. Check active grants (pre-approvals) for a matching action+repo
3. Fall back to the default permission state for the action
4. Return NEEDS_APPROVAL if neither grant nor default allows the action

#### Scenario: Grant overrides default
- **WHEN** a `pr:create` command is checked for repo `rm-rf-etc/Willow`
- **AND** the default for `pr:create` is OFF
- **BUT** an active grant exists for `pr:create` on `rm-rf-etc/*` with TTL remaining
- **THEN** the command is allowed

#### Scenario: Grant expired
- **WHEN** a `pr:create` command is checked
- **AND** a grant exists for `pr:create` BUT its `expires_at` is in the past
- **THEN** the grant is ignored
- **AND** the default (OFF) is used
- **AND** an approval request is created

### Requirement: Pre-approval grants
The system SHALL support creating grants that activate permissions for a configurable time period. Each grant SHALL have: action pattern, repo pattern (glob), TTL, optional max uses, reason, and created_by.

#### Scenario: Create pre-approval for specific repo
- **WHEN** user creates a grant: `{action: "pr:create", repo: "rm-rf-etc/Willow", ttl: "2h", reason: "PR review session"}`
- **THEN** the grant is stored in the database with `expires_at` = now + 2h
- **AND** `pr:create` commands targeting `rm-rf-etc/Willow` are auto-approved until expiry

#### Scenario: Pre-approval with max uses
- **WHEN** a grant has `max_uses: 5`
- **AND** the grant has been used 5 times
- **THEN** subsequent commands matching this grant require fresh approval
- **AND** the grant remains in the database for audit purposes

#### Scenario: Pre-approval with wildcard repo
- **WHEN** a grant has `repo: "rm-rf-etc/*"`
- **THEN** commands targeting any repo under `rm-rf-etc` matching the action are auto-approved

### Requirement: Approval request workflow
When a command is not covered by any active permission, the system SHALL create an approval request. The request SHALL contain: command, args, action, repo, timestamp, and status (pending). The system SHALL notify the user via configured channels (Telegram, web UI). The request SHALL block until resolved (approved or rejected) or expired.

#### Scenario: Approval request created and approved
- **WHEN** a command triggers an approval request
- **AND** the user approves via Telegram
- **THEN** the approval status changes to approved
- **AND** the waiting patched zsh receives the allow response
- **AND** the approval is logged in the audit trail

#### Scenario: Approval request rejected
- **WHEN** the user rejects an approval request
- **THEN** the waiting patched zsh receives the reject response
- **AND** the rejection is logged in the audit trail with the reason

#### Scenario: Approval request expires
- **WHEN** an approval request has been pending for longer than a configurable timeout (default 30 minutes)
- **THEN** the request status changes to expired
- **AND** the waiting patched zsh receives a reject response with reason "approval expired"

### Requirement: Permission configuration via web UI and API
Default permission states SHALL be configurable via the web UI and REST API. Changes take effect immediately for subsequent permission checks.

#### Scenario: Change default permission state
- **WHEN** user changes `repo:read` from ON to OFF via the web UI
- **THEN** subsequent `repo:read` commands require approval
- **AND** the change is logged in the audit trail

#### Scenario: Reset to defaults
- **WHEN** user clicks "Reset to Defaults" in the web UI
- **THEN** all default permissions are restored to their initial seed values
- **AND** existing grants are NOT affected
