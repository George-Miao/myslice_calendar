use std::{env, str::FromStr};

use attohttpc::get;
use color_eyre::{eyre::ContextCompat, Result};
use scraper::Html;

use crate::{selector, Class, Course, CourseMeta, GetText, Hygiene};

pub const URL: &str =
    "https://cs92prod.ps.syr.edu/psc/CS92PROD/EMPLOYEE/SA/c/SA_LEARNER_SERVICES.SSR_SSENRL_LIST";

pub fn request_html() -> Result<String> {
    let session_id = env::var("SESSION_ID")?;
    let token = env::var("TOKEN")?;
    let cookie = format!("ITS-CSPRD101-80-PORTAL-PSJSESSIONID={session_id};PS_TOKEN={token}");
    let res = get(URL)
        .header("Cookie", cookie)
        .max_redirections(10000)
        .send()?;
    Ok(res.text()?)
}

fn parse_title(title: &str) -> Result<(String, u32, String)> {
    let (subj_code, title) = title.split_once(" - ").wrap_err("Bad title format")?;
    let (subj, code) = subj_code
        .split_once(' ')
        .wrap_err("Bad subject & code format")?;
    Ok((subj.into(), code.parse()?, title.into()))
}

pub fn parse_html(text: &str) -> Result<Vec<Course>> {
    let container_sel = selector!("div[id*=win0divDERIVED_REGFRM1_DESCR20] > table");
    let title_sel = selector!("tr > td");
    let class_sel = selector!(id = "trCLASS_MTG_VW");
    let status_sel = selector!(id = "STATUS");
    let class_nbr_sel = selector!(id = "DERIVED_CLS_DTL_CLASS_NBR");
    let section_sel = selector!(id = "MTG_SECTION");
    let schedule_sel = selector!(id = "MTG_SCHED");
    let location_sel = selector!(id = "MTG_LOC");
    let mode_sel = selector!(id = "INSTRUCTION_MODE");
    let instructor_sel = selector!(id = "DERIVED_CLS_DTL_SSR_INSTR_LONG");
    let dates_sel = selector!(id = "MTG_DATES");

    Html::parse_document(text)
        .select(&container_sel)
        .map(|table| {
            let title = table.select(&title_sel).into_text().unwrap();
            let status = table.select(&status_sel).into_text().unwrap();
            let (subject, code, title) = parse_title(title)?;
            let mut meta = CourseMeta {
                status: status.parse()?,
                subject,
                title,
                code,
                class_num: 0,
            };
            let mut classes = vec![];

            for class in table.select(&class_sel) {
                meta.add_class_num();

                let number = class
                    .select(&class_nbr_sel)
                    .into_text()
                    .hygiene()
                    .map(|x| x.parse().unwrap());

                let section = class
                    .select(&section_sel)
                    .into_text()
                    .hygiene()
                    .map(ToOwned::to_owned);

                let schedule = class
                    .select(&schedule_sel)
                    .into_text()
                    .wrap_err("Failed to find schedule")?;

                let location = class
                    .select(&location_sel)
                    .into_text()
                    .wrap_err("Failed to find location")?
                    .to_owned();

                let mode = class
                    .select(&mode_sel)
                    .into_text()
                    .wrap_err("Failed to find mode")?;

                let instructor = class
                    .select(&instructor_sel)
                    .into_text()
                    .wrap_err("Failed to find instructor")?
                    .to_owned();

                let dates = class
                    .select(&dates_sel)
                    .into_text()
                    .wrap_err("Failed to find dates")?;

                let class = Class {
                    number,
                    section,
                    location,
                    instructor,
                    mode: FromStr::from_str(mode)?,
                    dates: FromStr::from_str(dates)?,
                    schedule: FromStr::from_str(schedule)?,
                };

                classes.push(class);
            }

            Ok(Course { meta, classes })
        })
        .collect()
}
