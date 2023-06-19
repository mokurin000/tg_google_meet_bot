pub mod auth;
pub mod event;
pub mod time;
pub mod utils;

use std::sync::OnceLock;

use calendar3::{hyper::client::HttpConnector, hyper_rustls::HttpsConnector, CalendarHub};
pub use google_calendar3 as calendar3;

pub const AUTHORIZED_USERS: [u64; 1] = [6004227082];
pub static CALENDAR_HUB: OnceLock<CalendarHub<HttpsConnector<HttpConnector>>> = OnceLock::new();
