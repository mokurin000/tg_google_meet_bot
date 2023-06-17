use crate::calendar3;
use crate::utils;

use calendar3::api::{ConferenceData, ConferenceSolutionKey, Event, EventDateTime};
use calendar3::chrono::{DateTime, Utc};

pub fn make_meet_event(
    title: impl Into<String>,
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
    req.summary = Some(title.into());
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
