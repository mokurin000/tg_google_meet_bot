use std::error::Error;

use crate::calendar3;
use crate::utils;
use crate::CALENDAR_HUB;

use calendar3::api::EntryPoint;
use calendar3::api::{ConferenceData, ConferenceSolutionKey, Event, EventDateTime};
use calendar3::chrono::{DateTime, Utc};
use calendar3::hyper::Body;
use calendar3::hyper::Response;
use tracing::debug;
use tracing::error;

fn make_meet_event(
    summary: impl Into<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    timezone: Option<impl Into<String> + Clone>,
) -> Event {
    let mut req = Event::default();
    let mut conf_data = ConferenceData::default();
    conf_data.create_request = Some(calendar3::api::CreateConferenceRequest {
        conference_solution_key: Some(ConferenceSolutionKey {
            type_: Some("hangoutsMeet".into()),
        }),
        request_id: Some(utils::unique_id(32)),
        ..Default::default()
    });
    req.conference_data = Some(conf_data);
    req.summary = Some(summary.into());
    req.start = Some(EventDateTime {
        date_time: Some(start_time),
        time_zone: timezone.clone().map(|s| s.into()),
        ..Default::default()
    });
    req.end = Some(EventDateTime {
        date_time: Some(end_time),
        time_zone: timezone.map(|s| s.into()),
        ..Default::default()
    });
    req
}

pub async fn insert_meet_event(
    time: DateTime<Utc>,
    summary: &str,
) -> Result<(Response<Body>, Event), Box<dyn Error>> {
    let req = make_meet_event(summary, time, time, Option::<&str>::None);

    let result = CALENDAR_HUB
        .get()
        .unwrap()
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

    if !res.0.status().is_success() {
        error!("{:#?}", res);
        return Err("request error".into());
    }

    debug!("ok: {res:#?}");

    Ok(res)
}

pub fn get_meet_link(event: &Event) -> Option<&str> {
    event
        .conference_data
        .as_ref()
        .and_then(|cdata| cdata.entry_points.as_ref())
        .and_then(|entry| {
            entry
                .get(0)
                .and_then(|EntryPoint { uri, .. }| uri.as_deref())
        })
}
