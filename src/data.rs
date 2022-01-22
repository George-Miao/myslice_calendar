use std::{convert::Infallible, ops::Range, str::FromStr};

use chrono::{NaiveDate, NaiveTime};
use color_eyre::{eyre::bail, Result};
use icalendar::{self as ical, Component, Event, EventStatus};

use crate::{convert_time, format_ny_time, format_weekday, parse_date, parse_time};

#[derive(Debug, Clone)]

pub enum Status {
    Enrolled,
    Dropped,
    Other(String),
}

impl Status {
    pub fn is_enrolled(&self) -> bool {
        matches!(self, Status::Enrolled)
    }
}

impl FromStr for Status {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Enrolled" => Ok(Status::Enrolled),
            "Dropped" => Ok(Status::Dropped),
            s => Ok(Status::Other(s.to_owned())),
        }
    }
}

#[derive(Debug, Clone)]

pub enum Mode {
    InPerson,
    Online,
    Hybrid,
    Other(String),
}

impl FromStr for Mode {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "P" => Ok(Mode::InPerson),
            "O" => Ok(Mode::Online),
            "H" => Ok(Mode::Hybrid),
            s => Ok(Mode::Other(s.to_owned())),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Schedule {
    Determined {
        days: String,
        time: Range<NaiveTime>,
    },
    Others(String),
    Tba,
}

impl FromStr for Schedule {
    type Err = color_eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! some {
            ($expr:expr, $str:expr) => {
                match $expr {
                    Some(x) => x,
                    None => return Ok(Self::Others($str)),
                }
            };
        }
        match s {
            "TBA" => Ok(Self::Tba),
            s => {
                let (days, time) = some!(s.split_once(' '), s.to_owned());

                let (start, end) = some!(time.split_once(" - "), s.to_owned());

                Ok(Self::Determined {
                    days: days.to_owned(),
                    time: Range {
                        start: parse_time(start)?,
                        end: parse_time(end)?,
                    },
                })
            }
        }
    }
}

#[derive(Clone, Debug)]

pub struct Dates {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl FromStr for Dates {
    type Err = color_eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, end) = match s.split_once(" - ") {
            Some((a, b)) => (a, b),
            None => return Err(color_eyre::eyre::eyre!("Bad format")),
        };
        Ok(Self {
            start: parse_date(start)?,
            end: parse_date(end)?,
        })
    }
}

#[derive(Debug, Clone)]

pub struct Course {
    pub meta: CourseMeta,
    pub classes: Vec<Class>,
}

#[derive(Debug, Clone)]

pub struct CourseMeta {
    pub status: Status,
    pub subject: String,
    pub code: u32,
    pub title: String,
    pub class_num: u8,
}

impl CourseMeta {
    pub fn add_class_num(&mut self) {
        self.class_num += 1
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    pub number: Option<u32>,
    pub section: Option<String>,
    pub schedule: Schedule,
    pub location: String,
    pub mode: Mode,
    pub instructor: String,
    pub dates: Dates,
}

impl Class {
    pub fn as_events(&self, meta: &CourseMeta) -> Option<Event> {
        let rrule = self.as_rrule().ok()?;

        let CourseMeta {
            subject,
            code,
            title,
            class_num,
            status,
            ..
        } = meta;

        if !status.is_enrolled() {
            return None;
        }

        let Dates {
            start: start_date, ..
        } = &self.dates;

        let (start_time, end_time) = match &self.schedule {
            Schedule::Determined { time, .. } => (time.start, time.end),
            _ => unreachable!("Class#as_rrule should already tested this"),
        };

        let mut summary = format!("{} {}", subject, code);

        if *class_num > 1 {
            if let Some(sec) = &self.section {
                summary.push(' ');
                summary.push_str(sec)
            }
        }

        let description = format!("{} given by {}", title, self.instructor);

        let event = Event::new()
            .status(EventStatus::Confirmed)
            .class(ical::Class::Public)
            .summary(&summary)
            .starts(convert_time(start_date, start_time))
            .ends(convert_time(start_date, end_time))
            .location(&self.location)
            .description(&description)
            .add_property("RRULE", &rrule)
            .done();

        Some(event)
    }

    /// RRULE looks like:
    /// ```
    /// RRULE:FREQ=WEEKLY;UNTIL=20220505T035959Z;INTERVAL=1;BYDAY=MO,WE,FR
    /// ```
    pub fn as_rrule(&self) -> Result<String> {
        let mut ret = String::with_capacity(15);

        let days = match &self.schedule {
            Schedule::Determined { days, .. } => days,
            _ => bail!("Datetime this course is not determined"),
        };

        ret.push_str("FREQ=WEEKLY;INTERVAL=1;");

        let Dates {
            end: ref end_date, ..
        } = self.dates;

        let until = format_ny_time(&end_date.and_hms(23, 59, 59));

        ret.push_str(&format!("UNTIL={};BYDAY={}", until, format_weekday(days)));
        Ok(ret)
    }
}
