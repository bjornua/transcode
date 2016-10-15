use std::borrow::Cow;

const DSECONDS_SECONDS: u64 = 100                    ;
const DSECONDS_MINUTES: u64 = 60   * DSECONDS_SECONDS;
const DSECONDS_HOURS:   u64 = 60   * DSECONDS_MINUTES;
const DSECONDS_DAYS:    u64 = 24   * DSECONDS_HOURS  ;
const DSECONDS_YEARS:   u64 = 365  * DSECONDS_DAYS   ;


fn div(p: u64, q: u64) -> (u64, u64) {
    (p / q, p % q)
}

pub fn pretty_centiseconds(num: i64) -> String {
    let is_negative = num < 0;
    let num: u64 = num.abs() as u64;

    let duration = num.clone();

    let (years  , duration) = div(duration, DSECONDS_YEARS  );
    let (days   , duration) = div(duration, DSECONDS_DAYS   );
    let (hours  , duration) = div(duration, DSECONDS_HOURS  );
    let (minutes, duration) = div(duration, DSECONDS_MINUTES);
    let (seconds, centisecond) = div(duration, DSECONDS_SECONDS);

    [
        if is_negative { Cow::Borrowed("-") } else { Cow::Borrowed("") },
        if DSECONDS_YEARS <= num { Cow::Owned(format!("{}y ", years)) } else { Cow::Borrowed("") },
        if DSECONDS_DAYS <= num { Cow::Owned(format!("{}d ", days)) } else { Cow::Borrowed("") },
        if DSECONDS_HOURS <= num { Cow::Owned(format!("{:02}:", hours)) } else { Cow::Borrowed("") },
        if DSECONDS_MINUTES <= num { Cow::Owned(format!("{:02}:", minutes)) } else { Cow::Borrowed("") },
        Cow::Owned(format!("{:02}.{:02}", seconds,  centisecond))
    ].concat()

}

#[test]
fn test() {
    for &(centiseconds, s) in &[
        (0000                ,             "00.00"),
        (0001                ,             "00.01"),
        (0002                ,             "00.02"),
        (0009                ,             "00.09"),
        (0010                ,             "00.10"),
        (0011                ,             "00.11"),
        (DSECONDS_MINUTES - 1,             "59.99"),
        (DSECONDS_MINUTES    ,          "01:00.00"),
        (DSECONDS_MINUTES + 1,          "01:00.01"),
        (DSECONDS_HOURS - 1  ,          "59:59.99"),
        (DSECONDS_HOURS      ,       "01:00:00.00"),
        (DSECONDS_HOURS + 1  ,       "01:00:00.01"),
        (DSECONDS_DAYS - 1   ,       "23:59:59.99"),
        (DSECONDS_DAYS       ,    "1d 00:00:00.00"),
        (DSECONDS_DAYS + 1   ,    "1d 00:00:00.01"),
        (DSECONDS_YEARS - 1  ,  "364d 23:59:59.99"),
        (DSECONDS_YEARS      , "1y 0d 00:00:00.00"),
        (DSECONDS_YEARS + 1  , "1y 0d 00:00:00.01"),
    ] {
        assert_eq!(pretty_centiseconds(centiseconds as i64), String::from(s))
    }
}
