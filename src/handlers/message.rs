use line_bot_sdk_rust::{
    client::LINE,
    line_messaging_api::{
        apis::MessagingApiApi,
        models::{
            ChatReference, MarkMessagesAsReadRequest, Message, ReplyMessageRequest, StickerMessage,
            TextMessage,
        },
    },
    line_webhook::models::{MessageContent, MessageEvent, Source},
};

use super::commands;

pub async fn handle(line: &LINE, event: MessageEvent) -> Result<(), String> {
    // Mark messages as read so the sender sees the blue checkmarks
    if let Some(user_id) = extract_user_id(&event.source) {
        let req = MarkMessagesAsReadRequest::new(ChatReference::new(user_id));
        if let Err(e) = line.messaging_api_client.mark_messages_as_read(req).await {
            eprintln!("[mark_as_read] failed: {e}");
        }
    }

    let reply_token = event.reply_token.ok_or("No reply token")?;

    match *event.message {
        MessageContent::TextMessageContent(text) => {
            handle_text(line, reply_token, &text.text, &event.source).await
        }
        MessageContent::StickerMessageContent(sticker) => {
            println!(
                "[message/sticker] package={} sticker={}",
                sticker.package_id, sticker.sticker_id
            );
            let info = format!("{sticker:#?}");
            reply(
                line,
                reply_token,
                vec![
                    Message::TextMessage(TextMessage::new(info)),
                    Message::StickerMessage(StickerMessage::new(
                        "11537".to_string(),
                        "52002734".to_string(),
                    )),
                ],
            )
            .await
        }
        _ => {
            println!("[unhandled event]");
            Ok(())
        }
    }
}

async fn handle_text(
    line: &LINE,
    reply_token: String,
    text: &str,
    _source: &Option<Box<Source>>,
) -> Result<(), String> {
    if let Some(cmd) = text.strip_prefix('/') {
        let cmd = cmd.split_whitespace().next().unwrap_or("");
        println!("[command] /{cmd}");
        match cmd {
            "help" => commands::help::handle(line, reply_token).await,
            _ => {
                reply(
                    line,
                    reply_token,
                    vec![Message::TextMessage(TextMessage::new(format!(
                        "Unknown command: /{cmd}\nType /help for available commands."
                    )))],
                )
                .await
            }
        }
    } else {
        println!("[message/text] {text}");
        reply(
            line,
            reply_token,
            vec![Message::TextMessage(TextMessage::new(text.to_string()))],
        )
        .await
    }
}

fn extract_user_id(source: &Option<Box<Source>>) -> Option<String> {
    match source {
        Some(s) => match s.as_ref() {
            Source::UserSource(u) => u.user_id.clone(),
            Source::GroupSource(g) => g.user_id.clone(),
            Source::RoomSource(r) => r.user_id.clone(),
            _ => None,
        },
        None => None,
    }
}

async fn reply(line: &LINE, reply_token: String, messages: Vec<Message>) -> Result<(), String> {
    let req = ReplyMessageRequest {
        reply_token,
        messages,
        notification_disabled: Some(false),
    };

    line.messaging_api_client
        .reply_message(req)
        .await
        .map_err(|e| format!("reply_message failed: {e}"))?;

    Ok(())
}
