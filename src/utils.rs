use anyhow::Result;

use dateparser::DateTimeUtc;

pub(crate) const COLORS: [(u8, u8, u8); 20] = [
    (200, 10, 20),
    (125, 30, 20),
    (130, 130, 10),
    (10, 150, 120),
    (220, 165, 0),
    (207, 64, 207),
    (255, 117, 43),
    (38, 169, 173),
    (114, 39, 219),
    (219, 39, 78),
    (60, 105, 230),
    (60, 230, 130),
    (5, 171, 74),
    (105, 201, 14),
    (15, 103, 135),
    (161, 66, 51),
    (120, 89, 6),
    (245, 44, 44),
    (230, 195, 20),
    (5, 2, 207),
];

pub trait ToSQL {
    fn to_sql(self) -> String;
}

impl<'a> ToSQL for Option<&'a str> {
    fn to_sql(self) -> String {
        match self {
            Some(v) => v.to_string(),
            None => "NULL".to_string(),
        }
    }
}


pub(crate) fn opt_from_sql<T, R>(repr: R) -> Option<T>
where
    T: From<String>,
    R: AsRef<str>,
{
    match repr.as_ref() {
        "NULL" => None,
        o => Some(o.to_string().into()),
    }
}

const SQLITE_DATETIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

/// Returns the given date `dt` to the format used by the db
pub(crate) fn dt_to_string(dt: DateTimeUtc) -> String {
    chrono::DateTime::<chrono::Local>::from(dt.0)
        .format(SQLITE_DATETIME_FORMAT)
        .to_string()
}

pub(crate) fn sql_string_to_dt(s: impl AsRef<str>) -> Result<chrono::NaiveDateTime> {
    Ok(chrono::NaiveDateTime::parse_from_str(s.as_ref(), SQLITE_DATETIME_FORMAT)?)
}

pub(crate) fn get_conflicting_column_name(err: &sqlite::Error) -> Option<String> {
    if let Some(19) = err.code {
        if let Some(ref msg) = err.message {
            if msg.starts_with("UNIQUE constraint failed: ") {
                let col = &msg["UNIQUE constraint failed: ".len()-1..].trim();
                return Some(col.to_string());
            }
        }
    }
    None
}


// adapted from https://github.com/chronotope/chrono/issues/342
/// Returns whether the input is a valid strftime format string
pub(crate) fn format_string_is_valid(s: impl AsRef<str>) -> bool {
    !chrono::format::StrftimeItems::new(s.as_ref()).any(|item| matches!(item, chrono::format::Item::Error))
}

#[macro_export]
macro_rules! read_sql_response {
    ($stmt:expr, $($col_name:ident => $t:ty),+) => {
        $(
            let $col_name = $stmt.read::<$t, _>(stringify!($col_name))?;
        )+
    };
}