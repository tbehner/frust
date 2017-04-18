use filter;
use filter::Filter;
use walkdir::DirEntry;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use nom::IResult;
use nom::Needed;
use chrono;
use chrono::NaiveTime;
use chrono::prelude::*;

enum RelativeTimeUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

struct RelativeTimeTuple {
    dist: i64,
    unit: RelativeTimeUnit,
}

named!(reltime_unit<RelativeTimeUnit>,
       do_parse!(
           unit: alt!(
               tag!("s") => {|_| RelativeTimeUnit::Second}
               | tag!("m") => {|_| RelativeTimeUnit::Minute}
               | tag!("h") => {|_| RelativeTimeUnit::Hour}
               | tag!("D") => {|_| RelativeTimeUnit::Day}
               | tag!("W") => {|_| RelativeTimeUnit::Week}
               | tag!("M") => {|_| RelativeTimeUnit::Month}
               | tag!("Y") => {|_| RelativeTimeUnit::Year}
               ) >>
           (unit)
       )
);

named!(integer_number<i64>,
       do_parse!( 
           int: re_bytes_find!("-?[0-9]+") >>
           (match String::from_utf8_lossy(int).into_owned().parse::<i64>() {
               Ok(i) => i,
               Err(e) => panic!("Could not parse to integer: {}", e),
            }
           )
       )
);

named!(reltime_parameter<RelativeTimeTuple>, 
       do_parse!(
           i: integer_number >> 
           u: reltime_unit >>
           (RelativeTimeTuple{dist: i, unit: u})
       )
    );

named!(onlydate<chrono::DateTime<Local>>, 
       do_parse!(
           year: re_bytes_find!("[0-9]{4}") >>
           tag!("-") >>
           month: re_bytes_find!("[0-9]{2}") >>
           tag!("-") >>
           day: re_bytes_find!("[0-9]{2}") >>
           (
               Local.ymd(String::from_utf8_lossy(year).into_owned().parse::<i32>().unwrap(), 
                         String::from_utf8_lossy(month).into_owned().parse::<u32>().unwrap(), 
                         String::from_utf8_lossy(day).into_owned().parse::<u32>().unwrap()).and_hms(0,0,0)           
            )
       )
);

named!(onlytime<chrono::DateTime<Local>>,
       do_parse!(
        hour: re_bytes_find!("[0-9]{2}") >>
        tag!(":") >>
        minute: re_bytes_find!("[0-9]{2}") >>
        (
            match Local::now().with_hour(String::from_utf8_lossy(hour).into_owned().parse::<u32>().unwrap()) {
                Some(t) => t.with_minute(String::from_utf8_lossy(minute).into_owned().parse::<u32>().unwrap()).unwrap(),
                None    => panic!("Error setting time for today!"),
            }
        )
      )
   );

pub struct TimeFilter {
    comp_op: filter::CompOp,
    attribute: filter::Attribute,
    epsilon: u64,
    timestamp: u64,
    operator_flip: bool,
}

fn parse_abs_date(param: &str) -> Option<chrono::DateTime<Local>> {
    let d : Option<chrono::DateTime<Local>> = match onlydate(param.as_bytes()) {
        IResult::Done(_, q)     => Some(q),
        IResult::Error(e)       => None,
        IResult::Incomplete(n)  => match n {
                Needed::Unknown => panic!("Need more input, but I haven't got a clou how much!"),
                Needed::Size(n) => panic!("Need {}bytes more input!", n),
            },
    };

    return d;
}

fn parse_abs_time(param: &str) -> Option<chrono::DateTime<Local>> {
    let t : Option<chrono::DateTime<Local>> = match onlytime(param.as_bytes()) {
        IResult::Done(_, q)     => Some(q),
        IResult::Error(e)       => None,
        IResult::Incomplete(n)  => match n {
                Needed::Unknown => panic!("Need more input, but I haven't got a clou how much!"),
                Needed::Size(n) => panic!("Need {}bytes more input!", n),
            },
    };

    return t;
}

fn parse_abs_datetime(param: &str) -> Option<chrono::DateTime<Local>> {
    let dt = Local.datetime_from_str(param, "%Y-%m-%d %H:%M");
    if dt.is_ok() {
        Some(dt.unwrap())
    } else {
        None
    }
}


impl TimeFilter {
    pub fn new(attribute: filter::Attribute, comp_op: filter::CompOp, param: &str) -> TimeFilter{
        // Disclaimer: I will not care about leap-anything until I do. 
        // Oh, and every month has 30 days....
        let reltime: Option<RelativeTimeTuple> = match reltime_parameter(param.as_bytes()) {
            IResult::Done(_, q) => Some(q),
            IResult::Error(e)   => None,
            IResult::Incomplete(n)  => match n {
                    Needed::Unknown => panic!("Need more input, but I haven't got a clou how much!"),
                    Needed::Size(n) => panic!("Need {}bytes more input!", n),
                },
        };

        let mut offset = 0u64;
        let mut flip = false;
        let mut epsilon = 0u64;

        if reltime.is_some(){
            let t = reltime.unwrap();
            let seconds = match t.unit {
                RelativeTimeUnit::Second => t.dist,
                RelativeTimeUnit::Minute => t.dist * 60,
                RelativeTimeUnit::Hour   => t.dist * 60 * 60,
                RelativeTimeUnit::Day    => t.dist * 60 * 60 * 24,
                RelativeTimeUnit::Week   => t.dist * 60 * 60 * 24 * 7,
                RelativeTimeUnit::Month  => t.dist * 60 * 60 * 24 * 30,
                RelativeTimeUnit::Year   => t.dist * 60 * 60 * 24 * 365,
            };

            // this will break if:
            //   * after Fri, 11 Apr 2262 23:47:16 GMT
            //   * checking for things before 1970-01-01
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            offset = now - seconds.abs() as u64;

            flip = match comp_op {
                filter::CompOp::Lower => seconds < 0, 
                filter::CompOp::LowerEqual => seconds < 0, 
                filter::CompOp::Greater => seconds < 0, 
                filter::CompOp::GreaterEqual => seconds < 0, 
                _    => false,
            };
        } else {
            let mut abs_datetime = parse_abs_datetime(param);
            let mut abs_date = parse_abs_date(param);
            let mut abs_time = parse_abs_time(param);
            if abs_datetime.is_some(){
                offset = abs_datetime.unwrap().timestamp() as u64;
                epsilon = 60 as u64;
            } else if abs_date.is_some() {
                offset = abs_date.unwrap().timestamp() as u64;
                epsilon = 60 * 60 * 24 as u64;
            } else if abs_time.is_some() {
                offset = abs_time.unwrap().timestamp() as u64;
                epsilon = 60 as u64;
            } else {
                panic!("Could not parse absolute datetime format {}. Supported datetime formats are: YYYY-MM-DD HH:MM, YYYY-MM-DD, HH:MM", param);
            }
        }
        TimeFilter{attribute: attribute, comp_op: comp_op, timestamp: offset, operator_flip: flip, epsilon: epsilon }
    }

    fn get_attribute(&self, dir_entry: &DirEntry) -> Duration {
        // to many unwraps on to little lines... this will go wrong really fast
        match self.attribute {
            filter::Attribute::Mtime => dir_entry.metadata().unwrap().modified().unwrap().duration_since(UNIX_EPOCH).unwrap(),
            filter::Attribute::Atime => match dir_entry.metadata().unwrap().accessed(){
                Ok(t) => t.duration_since(UNIX_EPOCH).unwrap(),
                Err(e) => panic!("Atime error: {}", e),
            },
            filter::Attribute::Ctime => match dir_entry.metadata().unwrap().created(){
                Ok(t) => t.duration_since(UNIX_EPOCH).unwrap(),
                Err(e) => panic!("Ctime error: {}", e),
            },
            _ => panic!("{:?} is not a type of time.", self.attribute),
        }
    }
}

impl Filter for TimeFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        let res = match self.comp_op {
            filter::CompOp::Equal        => {
                if self.epsilon == 0 {
                    self.get_attribute(dir_entry) == Duration::new(self.timestamp, 0)
                } else {
                    Duration::new(self.timestamp, 0) <= self.get_attribute(dir_entry) && self.get_attribute(dir_entry) <= Duration::new(self.timestamp + self.epsilon, 0)
                }
            },
            filter::CompOp::Lower        => self.get_attribute(dir_entry) <  Duration::new(self.timestamp, 0),
            filter::CompOp::LowerEqual   => self.get_attribute(dir_entry) <= Duration::new(self.timestamp, 0),
            filter::CompOp::Greater      => self.get_attribute(dir_entry) >  Duration::new(self.timestamp, 0),
            filter::CompOp::GreaterEqual => self.get_attribute(dir_entry) >= Duration::new(self.timestamp, 0),
            _ => panic!("Comparison operator {:?} is not supported for relative time comparisons.", self.comp_op),
        };
        return if self.operator_flip { !res } else { res }
    }
}
