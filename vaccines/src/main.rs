#[macro_use]
extern crate horrorshow;

use horrorshow::helper::doctype;
use horrorshow::Template;

use crate::table::TimeSeriesGroup;
use chrono::NaiveDate;

mod table;
mod web;

fn main() {
    let phase_1_end = NaiveDate::from_ymd(2021, 5, 1);
    let phase_2_end = NaiveDate::from_ymd(2021, 8, 1);

    let vaccine_data = include_str!("../data/vacciner.csv");
    let smitte_data = include_str!("../data/Newly_admitted_over_time.csv");
    let dode_data = include_str!("../data/Deaths_over_time.csv");

    let vacciner = TimeSeriesGroup::from_str(
        vec!["Vaccinerede personer i alt".to_string()].into(),
        vaccine_data,
    )
    .accumulative()
    .future_goal(
        "Mål 1: Minimering af død og alvorlig sygdom",
        phase_1_end,
        1_500_000,
        chrono::Duration::days(1),
    )
    .future_goal(
        "Mål 2: Forebyggelse af smittespredning",
        phase_2_end,
        3_500_000,
        chrono::Duration::days(1),
    )
    .plot(
        "vaccines",
        "Antal vaccinerede",
        "dag",
        "Personer der har påbegyndt vaccination mod ny coronavirus i alt",
    );

    let smitte =
        TimeSeriesGroup::from_str(vec!["Nyindlagte per dag".to_string()].into(), smitte_data)
            .future_goal(
                "Mål 1: Minimering af død og alvorlig sygdom",
                phase_1_end,
                0,
                chrono::Duration::days(1),
            )
            .future_goal(
                "Mål 2: Forebyggelse af smittespredning",
                phase_2_end,
                0,
                chrono::Duration::days(1),
            )
            .plot(
                "indlagte",
                "Antal indlagte",
                "dag",
                "Personer nyindskrevet med ny coronavirus per dag",
            );

    let dode = TimeSeriesGroup::from_str(vec!["Antal døde per dag".to_string()].into(), dode_data)
        .future_goal(
            "Mål 1: Minimering af død og alvorlig sygdom",
            phase_1_end,
            0,
            chrono::Duration::days(1),
        )
        .future_goal(
            "Mål 2: Forebyggelse af smittespredning",
            phase_2_end,
            0,
            chrono::Duration::days(1),
        )
        .plot(
            "dode",
            "Antal døde",
            "dag",
            "Personer der er død med ny coronavirus per dag",
        );

    let html = html! {
          : doctype::HTML;
          html {
            head {
                link(rel="stylesheet", href="https://cdn.jsdelivr.net/npm/bootstrap@4.5.3/dist/css/bootstrap.min.css") {}
                script(src = "https://code.jquery.com/jquery-3.5.1.slim.min.js", integrity="sha384-DfXdz2htPH0lsSSs5nCTpuj/zy4C+OGpamoFVy38MVBnE+IbbVYUew+OrCXaRkfj", crossorigin="anonymous") {}
                script(src = "https://cdn.jsdelivr.net/npm/bootstrap@4.5.3/dist/js/bootstrap.bundle.min.js", integrity="sha384-ho+j7jyWK8fNQe+A12Hb8AhRq26LrZ/JpcUGGOn+Y7RsweNrtN/tE3MoK7ZeZDyx", crossorigin="anonymous") {}
                script(src = "https://cdnjs.cloudflare.com/ajax/libs/Chart.js/2.9.4/Chart.min.js") {}
                script(src = "https://cdnjs.cloudflare.com/ajax/libs/chartjs-plugin-annotation/0.5.7/chartjs-plugin-annotation.min.js") {}
             }
             body {
                div(class="container") {
                  div(class="row") {
                    div(class="col col-lg-12") {
                      blockquote(class="blockquote lead") {
                        p(class="mb-0") {
                          : "Vaccinen er vores vej tilbage til hverdagen. Samværet. Krammet. Festerne. Alt det, vi længes efter. Men vaccinen er ikke en smutvej til at ophæve restriktioner eller slække på adfærden. Påskedag er i år den 4. april. Her vil årstiden igen hjælpe os. Vi vil være nået langt med vaccinationerne. Jeg tror - jeg håber - at påske bliver vores vendepunkt."
                        }
                        footer(class="blockquote-footer text-right") {
                          a(href="https://www.dr.dk/nyheder/politik/mette-frederiksen-varsler-moerke-og-barske-maaneder-forventer-foerst-corona", target="_blank") {
                            : "Mette Frederiksen, januar 2021"
                          }
                        }
                      }
                    }
                  }
                  div(class="row") {
                    div(class="col col-lg-12") {
                      : vacciner
                    }
                  }
                  div(class="row") {
                    div(class="col col-lg-12") {
                      : smitte
                    }
                  }
                  div(class="row") {
                    div(class="col col-lg-12") {
                      : dode
                    }
                  }
                }
             }
            }
    };

    println!("{}", html.into_string().unwrap());
}
