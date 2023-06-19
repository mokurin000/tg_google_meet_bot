use crate::calendar3;

use calendar3::hyper_rustls::HttpsConnector;
use calendar3::oauth2::ConsoleApplicationSecret;
use calendar3::{hyper, hyper_rustls, oauth2, CalendarHub};
use hyper::client::HttpConnector;

use tracing::info;

pub async fn build_calendar_hub(
) -> Result<CalendarHub<HttpsConnector<HttpConnector>>, anyhow::Error> {
    let ConsoleApplicationSecret {
        installed: secret,
        web: web_secret,
    } = serde_json::from_str(include_str!("../client_secret.json"))?;
    let secret = secret
        .or(web_secret)
        .expect("both installed and web was not found!");

    info!("Starting auth...");

    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPPortRedirect(11451),
    )
    .persist_tokens_to_disk("oauth_status.json")
    .build()
    .await?;

    Ok(CalendarHub::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        auth,
    ))
}
