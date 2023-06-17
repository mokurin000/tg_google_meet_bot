extern crate google_calendar3 as calendar3;

use calendar3::api::Calendar;
use calendar3::client::Hub;
use calendar3::hyper::service::Service;
use calendar3::hyper::Uri;
use calendar3::oauth2::storage::TokenStorage;
use calendar3::{chrono, hyper, hyper_rustls, oauth2, CalendarHub, FieldMask};
use calendar3::{Error, Result};
use std::default::Default;

#[derive(serde::Deserialize, serde::Serialize)]
struct Installed {
    pub installed: oauth2::ApplicationSecret,
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    // Get an ApplicationSecret instance by some means. It contains the `client_id` and
    // `client_secret`, among other things.
    let Installed { installed: secret } =
        serde_json::from_str(include_str!("../client_secret.json"))?;

    // Instantiate the authenticator. It will choose a suitable authentication flow for you,
    // unless you replace  `None` with the desired Flow.
    // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
    // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
    // retrieve them from storage.
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("oauth_status.json")
    .build()
    .await?;
    let hub = CalendarHub::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        auth,
    );

    let result = {
        let title = "淫趴";
        let mut req = Calendar::default();
        req.summary = Some(title.into());

        hub.calendars().insert(req).doit().await
    };

    match result {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
            Error::HttpError(_)
            | Error::Io(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => println!("Success: {:?}", res),
    }

    Ok(())
}
