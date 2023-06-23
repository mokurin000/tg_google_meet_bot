use std::error::Error;

use anyhow::Result;
use ash_meet_bot::auth::build_calendar_hub;
use ash_meet_bot::event::{get_meet_link, insert_meet_event};
use ash_meet_bot::time::{parse_time_to_utc, utc8_now};

use ash_meet_bot::CALENDAR_HUB;

use ash_meet_bot::AUTHORIZED_USERS;

use google_calendar3::chrono::{FixedOffset, TimeZone};
use teloxide::repls::CommandReplExt;
use teloxide::Bot;

use tracing::{error, info, trace, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env_filter = tracing_subscriber::EnvFilter::from_default_env();
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .init();

    init_calendar_hub().await?;

    dotenv::dotenv().ok();

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
#[command(rename_rule = "lowercase", parse_with = split_once, description = "These commands are supported:")]
enum MeetCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(
        description = "/meet hh:mm [|[DD/MM/YYYY][|duration]], [] means optional\nfor the duration format check https://docs.rs/duration-str/"
    )]
    Meet(String, String, Option<String>),
}

async fn answer(bot: Bot, msg: Message, cmd: MeetCommand) -> ResponseResult<()> {
    if cmd == MeetCommand::Help {
        let description = MeetCommand::descriptions().to_string();
        bot.send_message(msg.chat.id, description)
            .reply_to_message_id(msg.id)
            .disable_web_page_preview(true)
            .await?;
        return Ok(());
    }

    let MeetCommand::Meet(summary, time_str, duration) = cmd else {
        unreachable!()
    };

    if !msg.from().is_some_and(|u| is_admin(u.id.0)) {
        warn!("unauthorized access from {:#?}", msg.from());
        bot.send_message(
            msg.chat.id,
            "sorry, this bot is currently single user \n\nyou can run your instance with the [code](https://github.com/poly000/ash_meet_bot)",
        )
        .parse_mode(ParseMode::MarkdownV2)
        .reply_to_message_id(msg.id)
        .disable_web_page_preview(true)
        .await
        ?;
        return Ok(());
    }

    let now = utc8_now();
    let time_parsed = parse_time_to_utc(&time_str, duration.as_deref(), now);
    let Ok((start_time, end_time)) =  time_parsed else {
        let error = time_parsed.unwrap_err();

        bot.send_message(
            msg.chat.id,
            format!("time format error: {error}"),
        )
        .reply_to_message_id(msg.id)
        .await
        ?;
        return Ok(());
    };
    let result = insert_meet_event(start_time, end_time, &summary).await;

    let Ok(res) = result else {
                let e = result.unwrap_err().to_string();
                error!("{e}");
                bot.send_message(msg.chat.id, e)
                    .reply_to_message_id(msg.id)
                    .await?;
                return Ok(());
            };

    trace!("{res:#?}");

    let Some(meet_link) = get_meet_link(&res.1) else {
        warn!("did not get meet link with an success request");
        warn!("may it's an API breaking change/BUG");
        return Ok(());
    };

    info!("created sex party {meet_link} from {start_time} to {end_time}");

    bot.send_message(
        msg.chat.id,
        format!(
            "created {meet_link}\ntitle: {summary}, start: {}, end: {}",
            FixedOffset::east_opt(3600 * 8)
                .map(|fo| fo.from_utc_datetime(&start_time.naive_utc()))
                .unwrap()
                .to_rfc3339(),
            FixedOffset::east_opt(3600 * 8)
                .map(|fo| fo.from_utc_datetime(&end_time.naive_utc()))
                .unwrap()
                .to_rfc3339()
        ),
    )
    .reply_to_message_id(msg.id)
    .await?;

    Ok(())
}

fn is_admin(user: u64) -> bool {
    AUTHORIZED_USERS
        .get_or_init(|| {
            std::env::var("AUTHORIZED_USERS")
                .unwrap()
                .split(',')
                .filter_map(|id| id.trim().parse().ok())
                .collect()
        })
        .contains(&user)
}

fn split_once(s: String) -> Result<(String, String, Option<String>), ParseError> {
    split_once_imp(&s).map(|(l, r, p)| (l.into(), r.into(), p.map(|s| s.into())))
}

fn split_once_imp(s: &str) -> Result<(&str, &str, Option<&str>), ParseError> {
    let (summary, time) = s.split_once('|').unwrap_or((&s, ""));
    let (time, duration) = time.split_once('|').unwrap_or((time, ""));
    if duration.is_empty() {
        Ok((summary.trim(), time.trim(), None))
    } else {
        Ok((summary.trim(), time.trim(), Some(duration)))
    }
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use crate::split_once_imp;

    #[test_case("Sex Party | 12:00  " => ("Sex Party", "12:00", None))]
    #[test_case(" Sex Party | " => ("Sex Party", "", None))]
    #[test_case("  Sex Party Plus  " => ("Sex Party Plus", "", None))]
    #[test_case("淫趴 || 12s" => ("淫趴", "", Some("12s")))]
    #[test_case("淫趴 | 12:00| 12m" => ("淫趴", "12:00", Some("12s")))]
    fn test_command_split(input: &str) -> (&str, &str, Option<&str>) {
        split_once_imp(input).unwrap()
    }
}
