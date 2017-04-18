use filter;
use filter::Filter;
use walkdir::DirEntry;
use nom::IResult;
use nom::Needed;

enum SizeUnit {
    Bytes,
    Kilo,
    Mega,
    Giga,
    Terra,
}

struct SizeTuple {
    size: f64,
    unit: SizeUnit,
}

named!(size_unit<SizeUnit>, 
       do_parse!(
           unit: alt!(
                   tag!("b")    => { |_| SizeUnit::Bytes }
                   | tag!("k")  => { |_| SizeUnit::Kilo }
                   | tag!("M")  => { |_| SizeUnit::Mega }
                   | tag!("G")  => { |_| SizeUnit::Giga }
                   | tag!("T")  => { |_| SizeUnit::Terra }
               ) >>
           (unit)
        )
);

named!(float_number<f64>, 
       do_parse!(
           float: re_bytes_find!("-?[0-9]+(\\.[0-9]*)?") >>
           (match String::from_utf8_lossy(float).into_owned().parse::<f64>() {
               Ok(f)  => f,
               Err(e) => panic!("Could not parse size {}", e),
            }
           )
       )
   );

named!(size_parameter<SizeTuple>, do_parse!(
        s: float_number >>
        u: size_unit >>
        (SizeTuple{size: s, unit: u})
       )
    );

pub struct SizeFilter {
    size : u64,
    comp_op : filter::CompOp,
}

impl SizeFilter {
    pub fn new(op: filter::CompOp, size_param: &str) -> SizeFilter {
        let stuple = match size_parameter(size_param.as_bytes()) {
            IResult::Done(_, q) => q,
            IResult::Error(e)   => panic!("Size filter syntax error {}", e),
            IResult::Incomplete(n)  => match n {
                    Needed::Unknown => panic!("Need more input, but I haven't got a clou how much!"),
                    Needed::Size(n) => panic!("Need {}bytes more input!", n),
                },
        };
        let bytesize = match stuple.unit {
            SizeUnit::Bytes => stuple.size * (1024u64.pow(0) as f64),
            SizeUnit::Kilo  => stuple.size * (1024u64.pow(1) as f64),
            SizeUnit::Mega  => stuple.size * (1024u64.pow(2) as f64),
            SizeUnit::Giga  => stuple.size * (1024u64.pow(3) as f64),
            SizeUnit::Terra => stuple.size * (1024u64.pow(4) as f64),
        } as u64;
        SizeFilter{size: bytesize, comp_op: op}
    }
}

impl Filter for SizeFilter {
    fn test(&self, dir_entry: &DirEntry) -> bool {
        match self.comp_op {
            filter::CompOp::Lower        => dir_entry.metadata().unwrap().len() < self.size,
            filter::CompOp::LowerEqual   => dir_entry.metadata().unwrap().len() <= self.size,
            filter::CompOp::Equal        => dir_entry.metadata().unwrap().len() == self.size,
            filter::CompOp::GreaterEqual => dir_entry.metadata().unwrap().len() >= self.size,
            filter::CompOp::Greater      => dir_entry.metadata().unwrap().len() > self.size,
            _                            => panic!("Operator {:?} not covered for attribute size!", self.comp_op),
        }
    }
}
