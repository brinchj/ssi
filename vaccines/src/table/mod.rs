use crate::web;
use chrono::{DateTime, NaiveDate, Utc};
use im::ordmap::Entry;
use std::ops::Add;

pub struct TimeSeriesGroup {
    updated: DateTime<Utc>,
    series: Vec<TimeSeries>,
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    if s.contains('M') {
        let mut it = s.trim_start_matches('"').trim_end_matches('"').split('M').into_iter();
        let year = it.next()?;
        let mut it2 = it.next()?.split('D');
        let month = it2.next()?;
        let day = it2.next()?;
        Some(NaiveDate::from_ymd(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?))
    } else {
        let mut parts = s.split('-');
        let year: i32 = parts.next()?.parse().ok()?;
        let month: u32 = parts.next()?.parse().ok()?;
        let day: u32 = parts.next()?.parse().ok()?;
        Some(NaiveDate::from_ymd(year, month, day))
    }
}

impl TimeSeriesGroup {
    pub fn new(series: Vec<TimeSeries>) -> Self {
        let max_date = *series.iter().map(|ts| ts.latest_date()).max().unwrap();
        TimeSeriesGroup {
            updated: DateTime::from_utc(max_date.and_hms(0, 0, 0), Utc),
            series,
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

    fn final_date(&self) -> NaiveDate {
        let last_date = |ts: &TimeSeries| *ts.data.iter().last().unwrap().0;
        self.series.iter().map(last_date).max().unwrap()
    }

    pub fn accumulative(self) -> Self {
        let final_date = self.final_date();
        TimeSeriesGroup {
            updated: self.updated,
            series: self
                .series
                .into_iter()
                .map(|ts| ts.accumulative(final_date))
                .collect(),
        }
    }

    pub fn diff(self) -> Self {
        TimeSeriesGroup {
            updated: self.updated,
            series: self
                .series
                .into_iter()
                .map(|ts| ts.diff())
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.series.len()
    }

    pub fn prepend(self, val: i64, start: NaiveDate, step: chrono::Duration) -> Self {
        TimeSeriesGroup {
            updated: self.updated,
            series: self
                .series
                .into_iter()
                .map(|ts| ts.prepend(val, start, step))
                .collect(),
        }
    }

    pub fn last_sum(&self, start: impl Fn(&TimeSeries, &NaiveDate) -> i64) -> (NaiveDate, i64) {
        let last_date = |ts: &TimeSeries| *ts.data.iter().last().unwrap().0;
        let final_date = self.series.iter().map(last_date).max().unwrap();
        let final_sum: i64 = self.series.iter().map(|x| start(x, &final_date)).sum();
        (final_date, final_sum)
    }

    pub fn out_last_sum(self, out: &mut i64) -> Self {
        *out = self.last_sum(|ts, d| *ts.data.get(d).unwrap_or(&0)).1;
        self
    }

    pub fn future_goal_extrapolate(
        self,
        title: &str,
        goal: i64,
        step: chrono::Duration,
        speed: impl Fn(&TimeSeries, &NaiveDate) -> i64,
        start: impl Fn(&TimeSeries, &NaiveDate) -> i64,
        end_date_out: &mut NaiveDate,
    ) -> Self {
        let (final_date, final_sum) = self.last_sum(&start);
        let final_speed: i64 = self.series.iter().map(|x| speed(x, &final_date)).sum();

        if final_sum >= goal {
            return self
        }

        *end_date_out = final_date + step * ((goal - final_sum) / final_speed) as i32;
        return self.future_goal(title, *end_date_out, |_| goal, step, start);
    }

    pub fn future_goal(
        self,
        title: &str,
        date: NaiveDate,
        calc_goal: impl Fn(i64) -> i64,
        step: chrono::Duration,
        start: impl Fn(&TimeSeries, &NaiveDate) -> i64,
    ) -> Self {
        let (final_date, final_sum) = self.last_sum(&start);
        let goal = calc_goal(final_sum);

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
        if !goal_data.is_empty() {
            series.push(TimeSeries::new(tags, goal_data));
        }

        TimeSeriesGroup {
            updated: self.updated,
            series,
        }
    }

    pub fn plot_stacked(self, id: &str, title: &str, x: &str, y: &str) -> impl horrorshow::RenderOnce {
        let y = format!("{} — {}", y, self.updated.date().naive_local().to_string());
        web::ChartGraph::bar_plot_html(id.into(), title.into(), x.into(), y, self, true)
    }

    pub fn plot(self, id: &str, title: &str, x: &str, y: &str) -> impl horrorshow::RenderOnce {
        let y = format!("{} — {}", y, self.updated.date().naive_local().to_string());
        web::ChartGraph::bar_plot_html(id.into(), title.into(), x.into(), y, self, false)
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

    pub fn from_str(tags: im::OrdSet<String>, data: &str, f: impl Fn(Vec<&str>) -> i64) -> Self {
        let mut points = im::OrdMap::new();

        for line in data.lines() {
            let sep = if line.contains(';') { ';' } else { ',' };
            let mut it = line.split(sep);
            let date = parse_date(it.next().unwrap()).or_else(|| it.next().and_then(|s| parse_date(s)));

            if let Some(d) = date {
                let v = f(it.collect());
                match points.entry(d) {
                    Entry::Occupied(mut p) => *p.get_mut() = *p.get() + v,
                    Entry::Vacant(spot) => {
                        spot.insert(v);
                    }
                }
            }
        }

        Self::new(tags, points)
    }

    pub fn latest_date(&self) -> &NaiveDate {
        self.data.keys().max().unwrap()
    }

    pub fn accumulative(self, final_date: NaiveDate) -> Self {
        let init = (0i64, im::OrdMap::new());

        let (total, mut data) = self
            .data
            .into_iter()
            .fold(init, |(running_total, out), (t, y)| {
                ((y + running_total), out.update(t, y + running_total))
            });

        if !data.contains_key(&final_date) {
            data.insert(final_date, total);
        }

        TimeSeries {
            tags: self.tags,
            data,
        }
    }

    pub fn diff(self) -> Self {
        let init = (*self.data.iter().next().unwrap().1, im::OrdMap::new());
        let (_prev, data) = self
            .data
            .into_iter()
            .skip(1)
            .fold(init, |(prev, out), (t, y)| {
                (y, out.update(t, y - prev))
            });
        TimeSeries {
            tags: self.tags,
            data
        }
    }

    pub fn prepend(self, val: i64, start: NaiveDate, step: chrono::Duration) -> Self {
        let mut current = *self.data.keys().min().unwrap();
        let mut new_points = im::OrdMap::new();

        while current > start {
            current -= step;
            new_points.insert(current, val);
        }

        TimeSeries {
            tags: self.tags,
            data: new_points.union(self.data),
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
