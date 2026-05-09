use crate::pipeline::NotifyMessage;
use crate::types::{AppState, ApprovalRequest, GateError};

use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub async fn start_bot(config: crate::types::TelegramConfig, _state: AppState) -> Result<(), GateError> {
    let bot = teloxide::Bot::new(&config.bot_token);

    let handler = dptree::entry()
        .branch(Update::filter_callback_query().endpoint(|_bot: teloxide::Bot, q: teloxide::types::CallbackQuery| async move {
            tracing::info!(data = ?q.data, "callback query received");
            Ok::<_, teloxide::RequestError>(())
        }));

    let mut dispatcher = teloxide::dispatching::Dispatcher::builder(bot, handler)
        .build();

    dispatcher.dispatch().await;

    Ok(())
}

pub async fn send_approval_notification(
    bot: &teloxide::Bot,
    chat_id: i64,
    request: &ApprovalRequest,
) -> Result<(), GateError> {
    let text = format!(
        "🔴 INTERCEPT\n─────────────────────────\n{} {}\n📂 {}\n─────────────────────────\nNo grant covers this command.",
        request.command,
        request.args.join(" "),
        request.repo
    );

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "✅ Approve".to_string(),
                format!("approve:{}", request.id),
            ),
            InlineKeyboardButton::callback(
                "❌ Reject".to_string(),
                format!("reject:{}", request.id),
            ),
        ],
    ]);

    bot.send_message(teloxide::types::ChatId(chat_id), text)
        .reply_markup(keyboard)
        .await
        .map_err(|e| format!("telegram send error: {}", e))?;

    Ok(())
}

pub async fn send_intercept(
    bot: &teloxide::Bot,
    chat_id: i64,
    msg: &NotifyMessage,
    approval_id: &str,
) -> Result<(), GateError> {
    let text = format!(
        "🔴 INTERCEPT\n─────────────────────────\n{} {}\n📂 {}\n─────────────────────────\n{}",
        msg.command,
        msg.args.join(" "),
        msg.cwd,
        msg.reason.as_deref().unwrap_or("No grant covers this command.")
    );

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "✅ Approve".to_string(),
                format!("approve:{}", approval_id),
            ),
            InlineKeyboardButton::callback(
                "❌ Reject".to_string(),
                format!("reject:{}", approval_id),
            ),
        ],
    ]);

    bot.send_message(teloxide::types::ChatId(chat_id), text)
        .reply_markup(keyboard)
        .await
        .map_err(|e| format!("telegram send error: {}", e))?;

    Ok(())
}

pub async fn send_advisory(
    bot: &teloxide::Bot,
    chat_id: i64,
    msg: &NotifyMessage,
) -> Result<(), GateError> {
    let confidence_text = msg
        .confidence
        .map(|c| format!("\nConfidence: {:.0}%", c * 100.0))
        .unwrap_or_default();

    let agenda_text = msg
        .agenda_id
        .as_ref()
        .map(|a| format!("\nAgenda: {}", a))
        .unwrap_or_default();

    let grant_text = msg
        .grant_id
        .as_ref()
        .map(|g| format!("\nGrant: {}", g))
        .unwrap_or_default();

    let text = format!(
        "🟡 ADVISORY\n─────────────────────────\n{} {}\n📂 {}\n─────────────────────────\n{}{}{}{}",
        msg.command,
        msg.args.join(" "),
        msg.cwd,
        msg.reason.as_deref().unwrap_or("Allowed with visibility"),
        confidence_text,
        agenda_text,
        grant_text,
    );

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "🛑 Revoke grant".to_string(),
                format!("revoke_grant:{}", msg.grant_id.as_deref().unwrap_or("none")),
            ),
            InlineKeyboardButton::callback(
                "👍 Got it".to_string(),
                "ack".to_string(),
            ),
        ],
    ]);

    bot.send_message(teloxide::types::ChatId(chat_id), text)
        .reply_markup(keyboard)
        .await
        .map_err(|e| format!("telegram send error: {}", e))?;

    Ok(())
}

pub async fn send_request(
    bot: &teloxide::Bot,
    chat_id: i64,
    msg: &NotifyMessage,
    request_id: &str,
    duration: &str,
) -> Result<(), GateError> {
    let text = format!(
        "🔵 REQUEST\n─────────────────────────\n{}\n📂 {}\n─────────────────────────\nReason: {}\nDuration: {}",
        msg.command,
        msg.cwd,
        msg.reason.as_deref().unwrap_or("Not specified"),
        duration,
    );

    let keyboard = InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback(
                "✅ Grant".to_string(),
                format!("grant:{}", request_id),
            ),
            InlineKeyboardButton::callback(
                "❌ Deny".to_string(),
                format!("deny:{}", request_id),
            ),
        ],
    ]);

    bot.send_message(teloxide::types::ChatId(chat_id), text)
        .reply_markup(keyboard)
        .await
        .map_err(|e| format!("telegram send error: {}", e))?;

    Ok(())
}

pub async fn send_bootstrap_questions(
    bot: &teloxide::Bot,
    chat_id: i64,
    questions: &[crate::bootstrap::BootstrapQuestion],
    agenda_temp_id: &str,
) -> Result<(), GateError> {
    let mut text = String::from("📋 Quick questions to pre-approve future commands:\n\n");

    for (i, q) in questions.iter().enumerate() {
        text.push_str(&format!("{}. {}\n", i + 1, q.question));
        if !q.options.is_empty() {
            for opt in &q.options {
                text.push_str(&format!("   • {}\n", opt));
            }
        }
        text.push('\n');
    }

    text.push_str("Reply with your answers to set up the agenda.");

    let mut buttons = Vec::new();
    for (i, q) in questions.iter().enumerate() {
        if q.q_type == "choice" && !q.options.is_empty() {
            let row: Vec<InlineKeyboardButton> = q
                .options
                .iter()
                .map(|opt| {
                    InlineKeyboardButton::callback(
                        opt.clone(),
                        format!("bootstrap:{}:{}:{}", agenda_temp_id, i, opt),
                    )
                })
                .collect();
            buttons.push(row);
        }
    }

    let msg_builder = bot.send_message(teloxide::types::ChatId(chat_id), text);
    let msg_builder = if !buttons.is_empty() {
        msg_builder.reply_markup(InlineKeyboardMarkup::new(buttons))
    } else {
        msg_builder
    };

    msg_builder
        .await
        .map_err(|e| format!("telegram send error: {}", e))?;

    Ok(())
}

pub async fn handle_callback(
    bot: &teloxide::Bot,
    callback: teloxide::types::CallbackQuery,
    _state: &AppState,
) -> Result<(), GateError> {
    let data = callback.data.as_deref().unwrap_or("");
    let chat_id = callback.message.as_ref().map(|m| m.chat().id);

    if data == "ack" {
        if let Some(chat_id) = chat_id {
            bot.send_message(chat_id, "Acknowledged.")
                .await
                .map_err(|e| format!("telegram error: {}", e))?;
        }
        return Ok(());
    }

    if let Some((action, id)) = data.split_once(':') {
        match action {
            "approve" | "grant" => {
                tracing::info!(approval_id = %id, "approved via telegram");
            }
            "reject" | "deny" => {
                tracing::info!(approval_id = %id, "rejected via telegram");
            }
            "revoke_grant" => {
                tracing::info!(grant_id = %id, "grant revoked via telegram");
            }
            _ => {}
        }
    }

    if let Some(chat_id) = chat_id {
        bot.send_message(
            chat_id,
            format!("Processed: {}", data),
        )
        .await
        .map_err(|e| format!("telegram error: {}", e))?;
    }

    Ok(())
}

pub async fn handle_grant_command(
    bot: &teloxide::Bot,
    msg: &teloxide::types::Message,
    args: &str,
    _state: &AppState,
) -> Result<(), GateError> {
    bot.send_message(
        msg.chat.id,
        format!("Grant command received: {}", args),
    )
    .await
    .map_err(|e| format!("telegram error: {}", e))?;

    Ok(())
}

pub async fn handle_revoke_command(
    bot: &teloxide::Bot,
    msg: &teloxide::types::Message,
    args: &str,
    _state: &AppState,
) -> Result<(), GateError> {
    bot.send_message(
        msg.chat.id,
        format!("Revoke command received: {}", args),
    )
    .await
    .map_err(|e| format!("telegram error: {}", e))?;

    Ok(())
}
