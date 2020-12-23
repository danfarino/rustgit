const MINUTE: f64 = 60.;
const HOUR: f64 = MINUTE * 60.;
const DAY: f64 = HOUR * 24.;
const WEEK: f64 = DAY * 7.;
const MONTH: f64 = DAY * (365. / 12.);
const YEAR: f64 = MONTH * 12.;

fn format(n: f64, unit: f64, word: &str) -> String {
    let n = (n / unit).floor() as i64;
    let mut word = word.to_owned();
    if n != 1 {
        word += "s"
    }
    format!("{} {}", n, word)
}

pub fn format_age(dur: &chrono::Duration) -> String {
    match dur.num_seconds().abs() as f64 {
        n if n > YEAR => format(n, YEAR, "year"),
        n if n > MONTH => format(n, MONTH, "month"),
        n if n > WEEK => format(n, WEEK, "week"),
        n if n > DAY => format(n, DAY, "day"),
        n if n > HOUR => format(n, HOUR, "hour"),
        n if n > MINUTE => format(n, MINUTE, "minute"),
        n => format(n, 1., "second"),
    }
}
