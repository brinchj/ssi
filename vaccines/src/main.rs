#[macro_use]
extern crate horrorshow;

use horrorshow::helper::doctype;
use horrorshow::Template;

use crate::table::{TimeSeries, TimeSeriesGroup};
use chrono::{Duration, NaiveDate};

mod table;
mod web;

fn last_column_only(row: Vec<&str>) -> i64 {
    row.last().unwrap().trim().parse().unwrap()
}

fn sum_all_columns(row: Vec<&str>) -> i64 {
    row.iter().map(|c| c.trim().parse::<i64>().unwrap()).sum()
}

fn start_from_last(ts: &TimeSeries, date: &NaiveDate) -> i64 {
    *ts.data.get(date).unwrap_or(&0)
}

fn start_from_7d_avg(ts: &TimeSeries, date: &NaiveDate) -> i64 {
    (0..7)
        .map(|d| ts.data.get(&(*date - Duration::days(d))).unwrap_or(&0))
        .sum::<i64>()
        / 7
}

fn delta_7d_avg(ts: &TimeSeries, date: &NaiveDate) -> i64 {
    (ts.data.get(&(*date - Duration::days(1))).unwrap_or(&0)
    - ts.data.get(&(*date - Duration::days(8))).unwrap_or(&0)) / 7
}

fn main() {
    let start_date = NaiveDate::from_ymd(2020, 2, 1);

    let mut phase_1_end = NaiveDate::from_ymd(2021, 5, 1);
    let mut phase_2_end = NaiveDate::from_ymd(2021, 8, 1);
    let mut phase_3_end = NaiveDate::from_ymd(2021, 11, 1);

    let vaccine_data = include_str!("../data/vacciner.csv");
    let smitte_data = include_str!("../data/Municipality_cases_time_series.csv");
    let indlagte_data = include_str!("../data/Newly_admitted_over_time.csv");
    let dode_data = include_str!("../data/Deaths_over_time.csv");

    let vacciner = TimeSeriesGroup::from_str(
        vec!["Vaccinerede personer i alt".to_string()].into(),
        vaccine_data,
        last_column_only,
    )
    .prepend(0, start_date, Duration::days(1))
    .accumulative()
    .future_goal_extrapolate(
        "Mål 1: Minimering af død og alvorlig sygdom",
        1_400_000,
        chrono::Duration::days(1),
        delta_7d_avg,
        start_from_7d_avg,
        &mut phase_1_end
    )
    .future_goal_extrapolate(
        "Mål 2: Forebyggelse af smittespredning",
        3_500_000,
        chrono::Duration::days(1),
        delta_7d_avg,
        start_from_last,
        &mut phase_2_end
    )
    .future_goal_extrapolate(
        "Flok-immunitet",
        4_500_000,
        chrono::Duration::days(1),
        delta_7d_avg,
        start_from_last,
        &mut phase_3_end
    )
    .plot(
        "vaccines",
        "Antal vaccinerede",
        "dag",
        "Antal personer der har påbegyndt vaccination mod ny coronavirus i alt",
    );

    let smitte = TimeSeriesGroup::from_str(
        vec!["Smittede per dag".to_string()].into(),
        smitte_data,
        sum_all_columns,
    )
    .prepend(0, start_date, Duration::days(1))
    .future_goal(
        "Mål 1: Minimering af død og alvorlig sygdom",
        phase_1_end,
        1000,
        chrono::Duration::days(1),
        start_from_7d_avg,
    )
    .future_goal(
        "Mål 2: Forebyggelse af smittespredning",
        phase_2_end,
        500,
        chrono::Duration::days(1),
        start_from_last,
    )
    .future_goal(
        "Flok-immunitet",
        phase_3_end,
        0,
        chrono::Duration::days(1),
        start_from_last,
    )
    .plot(
        "smitte",
        "Antal smittede per dag",
        "dag",
        "Antal personer smittet med ny coronavirus per dag",
    );

    let indlagte = TimeSeriesGroup::from_str(
        vec!["Nyindlagte per dag".to_string()].into(),
        indlagte_data,
        last_column_only,
    )
    .prepend(0, start_date, Duration::days(1))
    .future_goal(
        "Mål 1: Minimering af død og alvorlig sygdom",
        phase_1_end,
        25,
        chrono::Duration::days(1),
        start_from_7d_avg,
    )
    .future_goal(
        "Mål 2: Forebyggelse af smittespredning",
        phase_2_end,
        0,
        chrono::Duration::days(1),
        start_from_last,
    )
    .future_goal(
        "Flok-immunitet",
        phase_3_end,
        0,
        chrono::Duration::days(1),
        start_from_last,
    )
    .plot(
        "indlagte",
        "Antal indlagte",
        "dag",
        "Personer nyindskrevet med ny coronavirus per dag",
    );

    let dode = TimeSeriesGroup::from_str(
        vec!["Antal døde per dag".to_string()].into(),
        dode_data,
        last_column_only,
    )
    .prepend(0, start_date, Duration::days(1))
    .future_goal(
        "Mål 1: Minimering af død og alvorlig sygdom",
        phase_1_end,
        0,
        chrono::Duration::days(1),
        start_from_7d_avg,
    )
    .future_goal(
        "Mål 2: Forebyggelse af smittespredning",
        phase_2_end,
        0,
        chrono::Duration::days(1),
        start_from_last,
    )
    .future_goal(
        "Flok-immunitet",
        phase_3_end,
        0,
        chrono::Duration::days(1),
        start_from_last,
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
                  div(class="row mt-1") {
                    div(class="col col-lg-12") {
                      blockquote(class="blockquote lead") {
                        span(class="mb-0") {
                          : "Min tidslinje og udvikling er baseret på videreførelse af sidste 7 dages trend. Det er ikke forudsigelser eller prognoser. "
                        }
                        span(class="mb-0") {
                          : "Vi kan ikke forudsige hvor mange vaccinedoser vi kommer til at modtage og hvornår. "
                        }
                        span(class="mb-0") {
                          : "Jeg ved, at vaccinationsprogrammets første mål er at "
                        }
                        span(class="mb-0") {
                          a(href="") {
                            : "mindske død og alvorlig sygdom ved vaccination af ~1.4 mio sårbare danskere"
                          }
                        }
                        span(class="mb-0") {
                          : ". Selvom denne gruppe prioriteres vil nogle vaccinedoser nok blive brugt til andre grupper, f.eks. personale på hospitaler. "
                        }
                        span(class="mb-0") {
                          a(href="https://www.dr.dk/nyheder/indland/forskere-advarer-om-ny-mutation-herhjemme-skraekscenariet-er-en-pandemi-ude-af", target="_blank")
                          : "Det ser ud til, at vi forhåbentlig kan opnå flok-immutet og stoppe smitten, når vi har vaccineret 60-80%, altså 3.5-4.5 mio danskere. "
                        }
                        footer(class="blockquote-footer text-right") {
                          : "Johan Brinch (mig, datalog, amatør, nørd), januar 2021"
                        }
                      }
                    }
                  }
                  hr {}
                  div(class="row") {
                    div(class="col col-lg-12") {
                      : dode
                    }
                  }
                  div(class="row mt-1") {
                    div(class="col col-lg-12") {
                      blockquote(class="blockquote lead") {
                        p(class="mb-0") {
                          : "Danmark prioriterer mindre død og alvorlig sygdom. Effekten af vaccination bliver nok ikke en lige linje som vist, men en anden form for løbende udvikling. Det bliver interessant at se den virkelige udvikling. Niveauet af smitte i samfundet afhænger i høj grad også af samfundsaktiviteten og virussens evne til at sprede sig."
                        }
                      }
                    }
                  }
                  hr {}
                  div(class="row") {
                    div(class="col col-lg-12") {
                      : indlagte
                    }
                  }
                  div(class="row mt-1") {
                    div(class="col col-lg-12") {
                      blockquote(class="blockquote lead") {
                        p(class="mb-0") {
                          : "Jeg forventer at se et markant dyk i indlæggelser, når vi har vaccineret de mest sårbare danskere."
                        }
                      }
                    }
                  }
                  hr {}
                  div(class="row") {
                    div(class="col col-lg-12") {
                      : smitte
                    }
                    blockquote(class="blockquote lead") {
                      p(class="mb-0") {
                        : "Jeg forventer først at se et markant dyk i antal smittede, når vi har vaccineret 60-80% af danskerne. Husk på, at samfundsaktivitet og vores opførsel også i høj grad driver smitten. Så vejen bliver ikke en lige linje i virkeligheden."
                      }
                    }
                  }
                  hr {}
                  div(class="row") {
                    a(href="https://github.com/brinchj/ssi/tree/master/vaccines", target="_blank") {
                      : "Kildekode på Github"
                    }
                  }
                }
              }
            }
    };

    println!("{}", html.into_string().unwrap());
}
