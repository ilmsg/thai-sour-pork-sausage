use line_bot_sdk_rust::{client::LINE, line_webhook::models::Event};

use crate::handlers;

pub async fn handle_event(line: &LINE, event: Event) {
    let result = match event {
        Event::MessageEvent(e) => handlers::message::handle(line, e).await,
        Event::FollowEvent(e) => handlers::follow::handle(line, e).await,
        Event::UnfollowEvent(e) => handlers::unfollow::handle(e),
        _ => {
            println!("[unhandle event]");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("[error] {e}");
    }
}
