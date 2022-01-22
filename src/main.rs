use std::fs;

use color_eyre::{
    eyre::{bail, Context},
    Result,
};
use icalendar::Calendar;

mod_use::mod_use![data, utils, fetch];

const BAD_TOKEN_MSG: &str = "UnAuthorized Token has been detected by the System.";

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv()?;

    fs::create_dir("data").wrap_err("Unable to create dir `data`")?;

    let text = request_html()?;
    fs::write("data/result.html", &text)?;

    // let text = fs::read_to_string("data/result.html")?;

    if text.contains(BAD_TOKEN_MSG) {
        bail!("Bad token or session_id")
    };

    let res = parse_html(&text)?;

    let events = res.as_slice().generate();

    let mut calendar = Calendar::new();
    calendar.extend(events);
    let res = calendar.to_string();

    fs::write("data/generated.ics", res)
        .wrap_err("Unable to write result to `data/generated.ics`")?;

    Ok(())
}

#[macro_export]
macro_rules! selector {
    (id = $id:literal) => {{
        ::scraper::Selector::parse(concat!("[id*=", $id, "]")).unwrap()
    }};
    ($raw:literal) => {{
        ::scraper::Selector::parse($raw).unwrap()
    }};
}
