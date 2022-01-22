use std::{fs, path::Path};

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

    if !Path::new("data").exists() {
        fs::create_dir("data").wrap_err("Unable to create dir `data`")?;
    }

    println!("Fetching data");
    let text = request_html()?;
    println!("Done fetching, generating");
    fs::write("data/result.html", &text)?;

    // let text = fs::read_to_string("data/result.html")?;

    if text.contains(BAD_TOKEN_MSG) {
        bail!("Bad token or session_id")
    };

    let res = parse_html(&text)?;

    let events = res.as_slice().generate();
    let num = events.len();

    println!("{} events found", num);

    let mut calendar = Calendar::new();
    calendar.extend(events);
    let res = calendar.to_string();

    fs::write("data/generated.ics", res)
        .wrap_err("Unable to write result to `data/generated.ics`")?;

    println!("Done generating, data stored in `./data/generated.ics`");
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
