#[macro_use]
extern crate horrorshow;

use horrorshow::helper::doctype;
use horrorshow::Template;

use crate::table::{TimeSeries, TimeSeriesGroup};
use chrono::{Duration, NaiveDate};

mod table;
mod web;

fn nth_column(n: usize, row: Vec<&str>) -> i64 {
    row[n].trim().parse().unwrap()
}

fn last_column(row: Vec<&str>) -> i64 {
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

fn delta_6weeks_avg(ts: &TimeSeries, date: &NaiveDate) -> i64 {
    let days = 6 * 7;
    let day = |delta| ts.data.get(&(*date - delta)).unwrap_or(&0);
    [
        day(Duration::days(0)) - day(Duration::days(days)),
        day(Duration::days(1)) - day(Duration::days(days + 1)),
    ]
    .iter()
    .max()
    .unwrap()
        / days
}

fn main() {
    let start_date = NaiveDate::from_ymd(2020, 2, 1);

    let phase_1 = 1_400_000;
    let phase_2 = 3_500_000;
    let phase_3 = 4_500_000;

    // Updated by extrapolation when setting up `vacciner` timeseries.
    let mut phase_1_end = NaiveDate::from_ymd(2021, 5, 1);
    let mut phase_2_end = NaiveDate::from_ymd(2021, 8, 1);
    let mut phase_3_end = NaiveDate::from_ymd(2021, 11, 1);

    // Updated below after extrapolation when setting up `vacciner` timeseries.
    let mut vaccinations_so_far = 0;

    let vaccine_data = include_str!("../data/vacciner.csv");
    let smitte_data = include_str!("../data/Municipality_cases_time_series.csv");
    let indlagte_data = include_str!("../data/Newly_admitted_over_time.csv");
    let dode_data = include_str!("../data/Deaths_over_time.csv");

    // People who have started vaccination.
    let vac_started = TimeSeries::from_str(
        vec!["Personer med 1 af 2 stik".to_string()].into(),
        vaccine_data,
        |r| nth_column(0, r),
    );

    // People who have started and completed vaccination.
    let vac_done = TimeSeries::from_str(
        vec!["Færdigvaccinerede".to_string()].into(),
        vaccine_data,
        |r| nth_column(1, r),
    );

    // Do not count someone `done` as `started`. Every person is counted only once.
    let vac_only_started = TimeSeries::new(
        vac_started.tags.clone(),
        vac_started
            .data
            .union_with(vac_done.data.clone(), std::ops::Sub::sub),
    );

    let vacciner = TimeSeriesGroup::new(vec![vac_done, vac_only_started])
        .prepend(0, start_date, Duration::days(1))
        .accumulative()
        .out_last_sum(&mut vaccinations_so_far)
        .future_goal_extrapolate(
            "Mål 1: Nedbring død og alvorlig sygdom",
            phase_1,
            chrono::Duration::days(1),
            delta_6weeks_avg,
            start_from_last,
            &mut phase_1_end,
        )
        .future_goal_extrapolate(
            "Mål 2: Forebyg smittespredning",
            phase_2,
            chrono::Duration::days(1),
            delta_6weeks_avg,
            start_from_last,
            &mut phase_2_end,
        )
        .future_goal_extrapolate(
            "Flok-immunitet",
            phase_3,
            chrono::Duration::days(1),
            delta_6weeks_avg,
            start_from_last,
            &mut phase_3_end,
        )
        .plot(
            "vaccines",
            "Antal vaccinerede",
            "dag",
            "Antal personer vaccineret mod ny coronavirus i alt",
        );

    // Update progress using new information on total vaccinations
    let calc_progress = |i: i64, n: i64| if i >= n { 1.0 } else { i as f64 / n as f64 };
    let phase_1_progress = calc_progress(vaccinations_so_far, phase_1);
    let phase_2_progress = calc_progress(vaccinations_so_far, phase_2);
    let phase_3_progress = calc_progress(vaccinations_so_far, phase_3);

    let calc_goal = |now, target_pct, progress| *[now, (now as f64 * (target_pct + progress - target_pct * progress)) as i64].iter().min().unwrap();

    let smitte = TimeSeriesGroup::new(vec![TimeSeries::from_str(
        vec!["Smittede per dag".to_string()].into(),
        smitte_data,
        sum_all_columns,
    )])
    .prepend(0, start_date, Duration::days(1))
    .future_goal(
        "Mål 1: Minimering af død og alvorlig sygdom",
        phase_1_end,
        |now| calc_goal(now, 0.75, phase_1_progress),
        chrono::Duration::days(1),
        start_from_7d_avg,
    )
    .future_goal(
        "Mål 2: Forebyggelse af smittespredning",
        phase_2_end,
        |now| calc_goal(now, 0.4, phase_2_progress),
        chrono::Duration::days(1),
        start_from_last,
    )
    .future_goal(
        "Flok-immunitet",
        phase_3_end,
        |now| calc_goal(now, 0.0, phase_3_progress),
        chrono::Duration::days(1),
        start_from_last,
    )
    .plot(
        "smitte",
        "Antal smittede per dag",
        "dag",
        "Antal personer smittet med ny coronavirus per dag",
    );

    let indlagte = TimeSeriesGroup::new(vec![TimeSeries::from_str(
        vec!["Nyindlagte per dag".to_string()].into(),
        indlagte_data,
        last_column,
    )])
    .prepend(0, start_date, Duration::days(1))
    .future_goal(
        "Mål 1: Minimering af død og alvorlig sygdom",
        phase_1_end,
        |now| calc_goal(now, 0.2, phase_1_progress),
        chrono::Duration::days(1),
        start_from_7d_avg,
    )
    .future_goal(
        "Mål 2: Forebyggelse af smittespredning",
        phase_2_end,
        |now| calc_goal(now, 0.0, phase_2_progress),
        chrono::Duration::days(1),
        start_from_last,
    )
    .future_goal(
        "Flok-immunitet",
        phase_3_end,
        |now| calc_goal(now, 0.0, phase_3_progress),
        chrono::Duration::days(1),
        start_from_last,
    )
    .plot(
        "indlagte",
        "Antal indlagte",
        "dag",
        "Personer nyindskrevet med ny coronavirus per dag",
    );

    let dode = TimeSeriesGroup::new(vec![TimeSeries::from_str(
        vec!["Antal døde per dag".to_string()].into(),
        dode_data,
        last_column,
    )])
    .prepend(0, start_date, Duration::days(1))
    .future_goal(
        "Mål 1: Minimering af død og alvorlig sygdom",
        phase_1_end,
        |now| calc_goal(now, 0.0, phase_1_progress),
        chrono::Duration::days(1),
        start_from_7d_avg,
    )
    .future_goal(
        "Mål 2: Forebyggelse af smittespredning",
        phase_2_end,
        |now| calc_goal(now, 0.0, phase_2_progress),
        chrono::Duration::days(1),
        start_from_last,
    )
    .future_goal(
        "Flok-immunitet",
        phase_3_end,
        |now| calc_goal(now, 0.0, phase_3_progress),
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
                          : "Min tidslinje og udvikling er baseret på videreførelse af de sidste 6 ugers trend. Det er ikke forudsigelser eller prognoser. "
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
