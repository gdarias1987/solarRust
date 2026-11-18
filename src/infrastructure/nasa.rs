use crate::{Error, Result};
use std::{io::Read, time::Duration};

pub enum Fetch {
    Unchanged {
        modified: Option<String>,
    },
    Downloaded {
        modified: Option<String>,
        bytes: Vec<u8>,
    },
}
pub trait ImageSource {
    fn fetch(&self, url: &str, modified: Option<&str>) -> Result<Fetch>;
}
pub struct Nasa {
    client: reqwest::blocking::Client,
}
impl Nasa {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(60))
                .connect_timeout(Duration::from_secs(15))
                .user_agent("SolarObservatory/0.1")
                .https_only(true)
                .build()?,
        })
    }
}
pub fn not_newer(remote: &str, local: &str) -> bool {
    match (
        httpdate::parse_http_date(remote),
        httpdate::parse_http_date(local),
    ) {
        (Ok(remote), Ok(local)) => remote <= local,
        _ => remote == local,
    }
}
impl ImageSource for Nasa {
    fn fetch(&self, url: &str, modified: Option<&str>) -> Result<Fetch> {
        let mut request = self.client.get(url);
        if let Some(value) = modified {
            request = request.header(reqwest::header::IF_MODIFIED_SINCE, value);
        }
        let response = request.send()?;
        let remote = response
            .headers()
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            if modified.is_none() {
                return Err(Error::Invalid(
                    "Unexpected 304 without a local image".into(),
                ));
            }
            return Ok(Fetch::Unchanged {
                modified: remote.or_else(|| modified.map(str::to_owned)),
            });
        }
        let response = response.error_for_status()?;
        if remote
            .as_deref()
            .zip(modified)
            .is_some_and(|(r, l)| not_newer(r, l))
        {
            return Ok(Fetch::Unchanged { modified: remote });
        }
        const MAX: u64 = 32 * 1024 * 1024;
        let mut bytes = Vec::new();
        response.take(MAX + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX {
            return Err(Error::Invalid("Remote image exceeds 32 MiB".into()));
        }
        Ok(Fetch::Downloaded {
            modified: remote,
            bytes,
        })
    }
}
