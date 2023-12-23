use chrono::{NaiveDate, NaiveDateTime};

pub const HOURS: [&'static str; 24] = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23"];

pub const MINUTES: [&'static str; 60] = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59"];

pub type LimitDateTime = (NaiveDate, usize, usize);

pub fn construct_datetime_utc(limit_dt: &LimitDateTime) -> NaiveDateTime {
  limit_dt.0
    .and_hms_opt(
      limit_dt.1 as u32,
      limit_dt.2 as u32,
      0u32
    )
    .unwrap()
}
