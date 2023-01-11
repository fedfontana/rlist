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

impl<T> ToSQL for Option<T>
where
    T: ToString + From<String>,
{
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
