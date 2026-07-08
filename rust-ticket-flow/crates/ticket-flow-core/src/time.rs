use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub fn now_rfc3339() -> String {
    let now = OffsetDateTime::now_utc();
    match now.format(&Rfc3339) {
        Ok(value) => value,
        Err(_) => now.unix_timestamp().to_string(),
    }
}

pub fn today_yyyymmdd() -> String {
    let now = OffsetDateTime::now_utc();
    let month = u8::from(now.month());
    format!("{:04}{:02}{:02}", now.year(), month, now.day())
}
