## ADDED Requirements

### Requirement: Telegram bot for approval notifications
The system SHALL integrate with Telegram via Bot API. When an approval request is created, the bot SHALL send a message to a configured chat ID containing: the command, action, repo, agent context, and inline keyboard buttons for Approve/Reject.

#### Scenario: Approval notification sent
- **WHEN** a command triggers an approval request
- **THEN** the Telegram bot sends a message like:
  ```
  🔐 Approval Request
  Command: gh pr create --repo rm-rf-etc/Willow --title "Fix bug"
  Action: gh:pr:create
  Agent: sandbox-opencode

  [✅ Approve] [❌ Reject]
  ```
- **AND** the message includes inline keyboard buttons with callback data containing the approval ID

#### Scenario: User approves via Telegram
- **WHEN** user taps the "Approve" button on the Telegram message
- **THEN** the bot updates the message to show "✅ Approved by @username"
- **AND** the approval request status changes to approved
- **AND** the waiting patched zsh receives the allow response

#### Scenario: User rejects via Telegram
- **WHEN** user taps the "Reject" button on the Telegram message
- **THEN** the bot updates the message to show "❌ Rejected by @username"
- **AND** the approval request status changes to rejected

#### Scenario: Multiple approval requests queued
- **WHEN** multiple commands are pending approval simultaneously
- **THEN** each gets its own Telegram message with its own Approve/Reject buttons
- **AND** approving/rejecting one does not affect the others

### Requirement: Pre-approval management via Telegram
The Telegram bot SHALL support creating pre-approval grants via commands.

#### Scenario: Create grant via Telegram
- **WHEN** user sends `/grant pr:create rm-rf-etc/Willow 2h "PR review session"`
- **THEN** the bot creates a grant for `pr:create` on `rm-rf-etc/Willow` with 2h TTL
- **AND** confirms with a message showing the grant details

#### Scenario: List active grants via Telegram
- **WHEN** user sends `/grants`
- **THEN** the bot replies with a list of active grants showing action, repo, expires_at, use_count

#### Scenario: Revoke grant via Telegram
- **WHEN** user sends `/revoke <grant_id>`
- **THEN** the bot disables the grant immediately
- **AND** confirms with a message

### Requirement: Telegram bot configuration
The Telegram bot token and chat ID SHALL be configured via the gate-server config file or environment variables.

#### Scenario: Bot token not configured
- **WHEN** gate-server starts and no Telegram bot token is configured
- **THEN** gate-server logs a warning and continues without Telegram integration
- **AND** approvals are only accessible via the web UI

#### Scenario: Bot token invalid
- **WHEN** gate-server starts with an invalid Telegram bot token
- **THEN** gate-server logs an error
- **AND** retries connection with exponential backoff
- **AND** the web UI remains functional
