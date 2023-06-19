use std::error::Error;

use anyhow::Result;
use ash_meet_bot::auth::build_calendar_hub;
use ash_meet_bot::event::{get_meet_link, insert_meet_event};
use ash_meet_bot::time::parse_time;

use ash_meet_bot::CALENDAR_HUB;

use ash_meet_bot::AUTHORIZED_USERS;

use teloxide::repls::CommandReplExt;
use teloxide::Bot;

use tracing::{debug, error, warn, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    init_calendar_hub().await?;

    let bot = Bot::from_env();

    MeetCommand::repl(bot, answer).await;

    Ok(())
}

async fn init_calendar_hub() -> Result<(), Box<dyn Error>> {
    let hub = build_calendar_hub().await?;

    CALENDAR_HUB
        .set(hub)
        .map_err(|_| "cannot build calendar hub!")?;

    Ok(())
}

use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Requester, ResponseResult};
use teloxide::types::{Message, ParseMode};
use teloxide::utils::command::{BotCommands, ParseError};
#[derive(BotCommands, PartialEq, Debug, Clone)]
#[command(rename_rule = "lowercase", parse_with = split_once)]
enum MeetCommand {
    Meet(String, String),
}

async fn answer(bot: Bot, msg: Message, cmd: MeetCommand) -> ResponseResult<()> {
    let MeetCommand::Meet(summary, time) = cmd;

    if !msg
        .from()
        .is_some_and(|u| AUTHORIZED_USERS.contains(&u.id.0))
    {
        warn!("unauthorized access from {:#?}", msg.from());
        bot.send_message(
            msg.chat.id,
            "sorry, this bot is currently single user \n\nyou can run your instance with the [code](https://github.com/poly000/ash_meet_bot)",
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await
        ?;
        return Ok(());
    }

    let Some(time) = parse_time(&time) else {
        return Ok(());
    };

    let result = insert_meet_event(time, &summary).await;

    let Ok(res) = result else {
                let e = result.unwrap_err();
                error!("{e}");
                let error = format!("请求错误：{e}");
                bot.send_message(msg.chat.id, &error).await?;
                return Ok(());
            };

    debug!("{res:#?}");

    let Some(meet_link) = get_meet_link(&res.1) else {
        warn!("did not get meet link with an success request");
        warn!("may it's an API breaking change/BUG");
        return Ok(());
    };

    info!("created sex party at {meet_link}");
    bot.send_message(msg.chat.id, meet_link).await?;

    Ok(())
}

fn split_once(s: String) -> Result<(String, String), ParseError> {
    let (summary, time) = s.split_once(" ").unwrap_or((&s, ""));
    Ok((summary.into(), time.into()))
}
