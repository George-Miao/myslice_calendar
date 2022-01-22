use std::fs;

use color_eyre::Result;
use icalendar::Calendar;

mod_use::mod_use![data, utils, fetch];

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv()?;

    // let text = request_html()?;
    // fs::write("result.html", &text)?;

    let text = fs::read_to_string("data/result.html")?;

    assert!(text.contains("ENL"));

    let res = parse_html(&text)?;

    let events = res.as_slice().generate();

    let mut calendar = Calendar::new();
    calendar.extend(events);
    let res = calendar.to_string();

    fs::write("data/generated.ics", res)?;

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
