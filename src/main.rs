use ash_meet_bot::calendar3;
use ash_meet_bot::{auth::build_calendar_hub, event::make_meet_event};

use calendar3::chrono::{Duration, Utc};

use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let hub = build_calendar_hub().await?;

    let now = Utc::now()
        .checked_add_signed(Duration::hours(8))
        .expect("time overflows");
    let req = make_meet_event("淫趴", now, now, Option::<&str>::None);

    let result = hub
        .events()
        .insert(req, "primary")
        .supports_attachments(true)
        .send_notifications(true)
        .conference_data_version(1)
        .doit()
        .await;

    let Ok(res) = result else {
        let e = result.unwrap_err();
        error!("{e}");
        return Err(e)?;
    };

    info!("ok: {res:?}");

    Ok(())
}
