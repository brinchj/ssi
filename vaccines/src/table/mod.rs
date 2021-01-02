use crate::web;
use chrono::{DateTime, NaiveDate, Utc};
use std::ops::Add;

pub struct TimeSeriesGroup {
    updated: DateTime<Utc>,
    series: Vec<TimeSeries>,
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    let mut parts = s.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    Some(NaiveDate::from_ymd(year, month, day))
}

impl TimeSeriesGroup {
    pub fn from_str(tags: im::OrdSet<String>, data: &str) -> Self {
        let mut points = im::OrdMap::new();

        for line in data.lines().skip(1) {
            let mut it = line.split(';');
            let date = parse_date(it.next().unwrap());

            if let Some(d) = date {
                let v = it.last().unwrap().trim();
                points.insert(d, v.parse().unwrap());
            }
        }

        let max_date = points.keys().max().unwrap();
        let updated = DateTime::from_utc(max_date.and_hms(0, 0, 0), Utc);
        TimeSeriesGroup {
            updated,
            series: vec![TimeSeries::new(tags, points)],
        }
    }

    pub fn series(&self) -> &[TimeSeries] {
        &self.series
    }

    pub fn xs(&self) -> im::OrdSet<NaiveDate> {
        self.series
            .iter()
            .flat_map(|f| f.data.keys())
            .cloned()
            .collect()
    }

    pub fn accumulative(self) -> Self {
        TimeSeriesGroup {
            updated: self.updated,
            series: self
                .series
                .into_iter()
                .map(|ts| ts.accumulative())
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.series.len()
    }

    pub fn future_goal(
        self,
        title: &str,
        date: NaiveDate,
        goal: i64,
        step: chrono::Duration,
    ) -> Self {
        let last_date = |ts: &TimeSeries| *ts.data.iter().last().unwrap().0;
        let final_date = self.series.iter().map(last_date).max().unwrap();

        let datapoint = |ts: &TimeSeries| *ts.data.get(&final_date).unwrap_or(&0);
        let final_sum: i64 = self.series.iter().map(datapoint).sum();

        let mut running_date = final_date;
        let all_days = (date - running_date).num_days();

        let mut goal_data = im::OrdMap::new();
        while running_date < date {
            running_date += step;

            let days_spent = (running_date - final_date).num_days();
            let progress = ((goal - final_sum) * days_spent) / all_days;
            goal_data.insert(running_date, final_sum + progress);
        }

        let tags = im::OrdSet::unit(title.to_string());
        let mut series = self.series;
        series.push(TimeSeries::new(tags, goal_data));

        TimeSeriesGroup {
            updated: self.updated,
            series,
        }
    }

    pub fn plot(self, id: &str, title: &str, x: &str, y: &str) -> impl horrorshow::RenderOnce {
        let y = format!("{} — {}", y, self.updated.date().naive_local().to_string());
        web::ChartGraph::bar_plot_html(id.into(), title.into(), x.into(), y, self)
    }
}

#[derive(Default, Clone)]
pub struct TimeSeries {
    pub tags: im::OrdSet<String>,
    pub data: im::OrdMap<NaiveDate, i64>,
}

impl TimeSeries {
    pub fn new(tags: im::OrdSet<String>, data: im::OrdMap<NaiveDate, i64>) -> TimeSeries {
        TimeSeries { tags, data }
    }

    pub fn accumulative(self) -> Self {
        let init = (0i64, im::OrdMap::new());
        let (_total, data) = self
            .data
            .into_iter()
            .fold(init, |(running_total, out), (t, y)| {
                ((y + running_total), out.update(t, y + running_total))
            });
        TimeSeries {
            tags: self.tags,
            data,
        }
    }
}

impl Add for TimeSeries {
    type Output = TimeSeries;

    fn add(self, rhs: Self) -> Self::Output {
        TimeSeries {
            tags: self.tags.union(rhs.tags),
            data: self.data.union_with(rhs.data, std::ops::Add::add),
        }
    }
}
