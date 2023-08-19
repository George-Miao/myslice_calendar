use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

use color_eyre::{eyre::Context, Result};
use icalendar::{CalendarDateTime, Event};
use scraper::element_ref::Select;

use crate::Course;

pub trait Hygiene {
    #[must_use]
    fn hygiene(self) -> Self;
}

impl<'a> Hygiene for Option<&'a str> {
    fn hygiene(self) -> Self {
        match self {
            Some("\u{a0}") => None,
            rest => rest.map(str::trim),
        }
    }
}

pub trait GetText<'a>: Sized {
    fn into_text(self) -> Option<&'a str>;
    fn get_text(&'a mut self) -> Option<&'a str>;
}

impl<'a, 'b> GetText<'a> for Select<'a, 'b> {
    fn get_text(&'a mut self) -> Option<&'a str> {
        self.next().and_then(|x| x.text().next())
    }

    fn into_text(mut self) -> Option<&'a str> {
        self.next().and_then(|x| x.text().next())
    }
}

pub fn generate<'a>(iter: impl IntoIterator<Item = &'a Course>) -> Result<Vec<Event>> {
    iter.into_iter()
        .flat_map(|course| {
            course
                .classes
                .iter()
                .map(|class| class.as_event(&course.meta))
        })
        .filter_map(Result::transpose)
        .collect()
}

pub fn parse_date(date: &str) -> Result<NaiveDate> {
    let parts = date
        .split('/')
        .map(|x| -> Result<u32> { x.parse().wrap_err("Failed to parse number") })
        .collect::<Result<Vec<_>>>()?;
    let date = NaiveDate::from_ymd_opt(parts[2] as i32, parts[0], parts[1])
        .ok_or_else(|| color_eyre::eyre::eyre!("Failed to convert date"))?;
    Ok(date)
}

pub fn parse_time(date: &str) -> Result<NaiveTime> {
    let date = NaiveTime::parse_from_str(date, "%l:%M%p")?;
    Ok(date)
}

pub fn format_weekday(days: &str) -> String {
    days.to_uppercase()
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i != 0 && i % 2 == 0 {
                Some(',')
            } else {
                None
            }
            .into_iter()
            .chain(std::iter::once(c))
        })
        .collect()
}

pub fn format_ny_time(datetime: &NaiveDateTime) -> String {
    let tz = chrono_tz::Tz::America__New_York;
    chrono::Utc
        .from_local_datetime(&tz.from_local_datetime(datetime).unwrap().naive_local())
        .unwrap()
        .format("%Y%m%dT%H%M%SZ")
        .to_string()
}

pub fn convert_time_in_ny(date: &NaiveDate, time: NaiveTime) -> CalendarDateTime {
    CalendarDateTime::WithTimezone {
        date_time: date.and_time(time),
        tzid: "America/New_York".into(),
    }
}

#[test]
fn test_parse_date() {
    println!("{:#?}", parse_date("05/03/2022"));
    println!("{:#?}", parse_time("2:00PM"));
}

#[test]
fn test_parse_weekday() {
    assert_eq!(format_weekday("MoWeFr"), "MO,WE,FR");
    assert_eq!(format_weekday("Su"), "SU");
}
