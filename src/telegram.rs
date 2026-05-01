use crate::types::{AppState, ApprovalRequest, GateError};

/// Initialize and start the Telegram bot.
///
/// Spec: gate-telegram/spec.md > "Telegram bot for approval notifications"
/// Tasks: 5.7
/// Async — initializes teloxide bot, registers handlers, starts dispatching.
pub async fn start_bot(config: crate::types::TelegramConfig, state: AppState) -> Result<(), GateError> {
    todo!("start_bot: create teloxide::Bot with token, register message and callback handlers, start dispatching")
}

/// Send an approval notification with inline Approve/Reject buttons.
///
/// Spec: gate-telegram/spec.md > "Approval notification sent"
/// Tasks: 5.8
/// Async — sends formatted Telegram message.
pub async fn send_approval_notification(
    bot: &teloxide::Bot,
    chat_id: i64,
    request: &ApprovalRequest,
) -> Result<(), GateError> {
    todo!("send_approval_notification: format approval message with command/action/repo, add inline keyboard with Approve/Reject buttons containing approval ID")
}

/// Handle Approve/Reject button callback from Telegram.
///
/// Spec: gate-telegram/spec.md > "User approves via Telegram"
/// Tasks: 5.9
/// Async — resolves approval, updates message.
pub async fn handle_callback(
    bot: &teloxide::Bot,
    callback: teloxide::types::CallbackQuery,
    state: &AppState,
) -> Result<(), GateError> {
    todo!("handle_callback: parse callback data for approval_id + approve/reject, call approvals::resolve_approval, update message text, answer callback query")
}

/// Handle /grant command from Telegram.
///
/// Spec: gate-telegram/spec.md > "Create grant via Telegram"
/// Tasks: 5.10
/// Async — creates grant from parsed command args.
pub async fn handle_grant_command(
    bot: &teloxide::Bot,
    msg: &teloxide::types::Message,
    args: &str,
    state: &AppState,
) -> Result<(), GateError> {
    todo!("handle_grant_command: parse args (action, repo, ttl, reason), call grants::create_grant, reply with confirmation")
}

/// Handle /revoke command from Telegram.
///
/// Spec: gate-telegram/spec.md > "Revoke grant via Telegram"
/// Tasks: 5.10
/// Async — revokes grant by ID.
pub async fn handle_revoke_command(
    bot: &teloxide::Bot,
    msg: &teloxide::types::Message,
    args: &str,
    state: &AppState,
) -> Result<(), GateError> {
    todo!("handle_revoke_command: parse grant_id from args, call grants::revoke_grant, reply with confirmation")
}
