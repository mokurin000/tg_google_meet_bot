use std::error::Error;

use ash_meet_bot::auth::build_calendar_hub;
use ash_meet_bot::event::insert_meet_event;
use ash_meet_bot::time::parse_time;

use ash_meet_bot::{calendar3, CALENDAR_HUB};
use calendar3::api::EntryPoint;

use ash_meet_bot::AUTHORIZED_USERS;

use tracing::{debug, error, info};
#[tokio::main]
async fn main() -> anyhow::Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let hub = build_calendar_hub().await?;
    CALENDAR_HUB
        .set(hub)
        .map_err(|_| "cannot build calendar hub!")
        .unwrap();

    let now = parse_time("").unwrap();
    let res = insert_meet_event(now, "淫趴").await?;

    let meet_link = res
        .1
        .conference_data
        .as_ref()
        .and_then(|cdata| cdata.entry_points.as_ref())
        .and_then(|entry| {
            entry
                .get(0)
                .and_then(|EntryPoint { uri, .. }| uri.as_deref())
        });

    println!("{meet_link:?}");

    Ok(())
}
