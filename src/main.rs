use std::{
    env, fs,
    path::{Path, PathBuf},
};

use color_eyre::{
    eyre::{bail, Context, ContextCompat},
    Result,
};
use icalendar::Calendar;
use tap::Pipe;

mod_use::mod_use![data, utils, fetch];

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv()?;

    if !Path::new("data").exists() {
        fs::create_dir("data").wrap_err("Unable to create dir `data`")?;
    }

    let path = env::args()
        .nth(1)
        .wrap_err("Please provide a filename as argument")?
        .pipe(PathBuf::from);

    if !path.exists() {
        bail!("File `{}` does not exist", path.display());
    } else {
        println!("Parsing file: {}", path.display());
    }

    let res = fs::read_to_string(&path)?.pipe_as_ref(parse_html)?;

    println!("{res:#?}");
    println!("HTML parsed, generate ICS");

    let events = generate(res.iter())?;
    let num = events.len();

    println!("{} events found", num);

    let mut calendar = Calendar::new();
    calendar.extend(events);
    let res = calendar.to_string();

    fs::write(path.with_extension("ics"), res).wrap_err("Unable to write result to file")?;

    println!(
        "Done generating, data stored in `./data/${}`",
        path.display()
    );
    Ok(())
}

macro_rules! selector {
    (id = $id:literal) => {{ ::scraper::Selector::parse(concat!("[id ^= ", $id, "]")).unwrap() }};
    ($raw:literal) => {{ ::scraper::Selector::parse($raw).unwrap() }};
}

pub(crate) use selector;
