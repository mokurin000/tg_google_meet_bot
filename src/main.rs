extern crate google_calendar3 as calendar3;

use calendar3::api::{ConferenceData, ConferenceSolutionKey, Event, EventDateTime};
use calendar3::chrono::{DateTime, Utc};
use calendar3::hyper::client::HttpConnector;
use calendar3::hyper_rustls::HttpsConnector;
use calendar3::oauth2::authenticator::Authenticator;
use calendar3::{hyper, hyper_rustls, oauth2, CalendarHub};
use std::default::Default;
use tracing::{error, info};

#[derive(serde::Deserialize, serde::Serialize)]
struct Installed {
    pub installed: oauth2::ApplicationSecret,
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let Installed { installed: secret } =
        serde_json::from_str(include_str!("../client_secret.json"))?;

    info!("Starting auth...");

    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("oauth_status.json")
    .build()
    .await?;

    let hub = build_calendar_hub(auth);

    let now = Utc::now();
    let req = make_meet_conf("淫趴", now, now, "Asia/Shanghai");

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

    // res.1.hangout_link;

    info!("ok: {res:?}");

    Ok(())
}

fn build_calendar_hub(
    auth: Authenticator<HttpsConnector<HttpConnector>>,
) -> CalendarHub<HttpsConnector<HttpConnector>> {
    CalendarHub::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        auth,
    )
}

fn unique_id() -> String {
    use rand::distributions::{Alphanumeric, Distribution};
    use rand::thread_rng;

    let mut rng = thread_rng();
    Alphanumeric
        .sample_iter(&mut rng)
        .take(32)
        .map(char::from)
        .collect()
}

fn make_meet_conf(
    title: impl Into<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    timezone: impl Into<String> + Clone,
) -> Event {
    let mut req = Event::default();
    let mut conf_data = ConferenceData::default();
    conf_data.create_request = Some(calendar3::api::CreateConferenceRequest {
        conference_solution_key: Some(ConferenceSolutionKey {
            type_: Some("hangoutsMeet".into()),
        }),
        request_id: Some(unique_id()),
        ..Default::default()
    });
    req.conference_data = Some(conf_data);
    req.summary = Some(title.into());
    req.start = Some(EventDateTime {
        date_time: Some(start_time),
        time_zone: Some(timezone.to_owned().into()),
        ..Default::default()
    });
    req.end = Some(EventDateTime {
        date_time: Some(end_time),
        time_zone: Some(timezone.into()),
        ..Default::default()
    });
    req
}
