///
///
/// # TODOs
///   * or, and und klammern in filter-Ausdrücken unterstützen
///


use nom::IResult;
use std::str;
use std::string::String;
use nom::eol;
use filter;
use query;

named!(komma, ws!(tag!(",")));
named!(comp_op, alt!(
          tag!("<")
        | tag!("<=")
        | tag!("==")
        | tag!(">=")
        | tag!(">")
        | tag!("~")
        )
    );

enum LogicOp {
    And,
    Or,
}

named!(logic_op<LogicOp>, 
       alt!(
        ws!(tag!("or")) =>  { |_| LogicOp::Or }
        | 
        ws!(tag!("and")) => { |_| LogicOp::And }
        )
);

named!(from_keyword, ws!(tag!("from")));
named!(where_keyword, ws!(tag!("where")));
named!(exec_keyword, ws!(tag!("exec")));
named!(attribute<String>, 
       do_parse!(
           tag: ws!(alt!( tag!("name") 
                    | tag!("size")
                    | tag!("mtime")
                    | tag!("ctime")
                    | tag!("filetype")
                    | tag!("mimetype")
                    | tag!("inode")
                    | tag!("basename"))) >>
           (String::from_utf8_lossy(tag).into_owned())
           )
       );

named!(directory<String>, 
       do_parse!(
           dir: re_bytes_find!("(/)?([^/\0, ]+(/)?)+") >>
           (String::from_utf8_lossy(dir).into_owned())
       )
   );
named!(num_paramter, re_bytes_find!("(-)?[0-9]+[a-zA-Z]"));
named!(str_paramter, re_bytes_find!("'[^']+'"));
named!(date_parameter, re_bytes_find!("(([0-9]{4}-[0-9]{2}-[0-9]{2})?\\s*[0-9]{1,2}:[0-9]{2})|([0-9]{4}-[0-9]{2}-[0-9]{2})"));

named!(parameter, alt!(num_paramter | str_paramter | date_parameter));

named!(filter<filter::Filter>, 
       do_parse!(
           attr: attribute >>
           op  : ws!(comp_op)   >>
           param: ws!(parameter) >>
           (filter::Filter::new(attr, 
                                String::from_utf8_lossy(op).into_owned(), 
                                String::from_utf8_lossy(param).into_owned()
                                )
            )
     )
);

named!(directory_list<Vec<String>>, separated_list!(komma, directory));
named!(attribute_list<Vec<String>>, separated_list!(komma, attribute));
named!(filter_list<Vec<filter::Filter>>,   separated_list!(logic_op, filter));
named!(command<String>, 
       do_parse!(
       cmd: re_bytes_find!(".*") >>
        (String::from_utf8_lossy(cmd).into_owned())
    )
);

named!(query<query::Query>, do_parse!(
        attributes: attribute_list >> 
        from_keyword >>
        directories: directory_list >>
        where_keyword >>
        filters: filter_list >>
        exec_keyword >>
        command: command >>
        (query::Query::new(Some(attributes), Some(directories), Some(filters), Some(command)))
        )
    );

#[test]
fn test_directory(){
    assert_eq!(directory(b"/usr/bin/bash"), IResult::Done(&b""[..], String::from("/usr/bin/bash"))); 
    assert_eq!(directory(b"./foo/bar"), IResult::Done(&b""[..], String::from("./foo/bar"))); 
    assert_eq!(directory(b".."), IResult::Done(&b""[..], String::from(".."))); 
    assert_eq!(directory(b"src"), IResult::Done(&b""[..], String::from("src"))); 
}

#[test]
fn test_parameter(){
    assert_eq!(parameter(b"1M"), IResult::Done(&b""[..], &b"1M"[..])); 
    assert_eq!(parameter(b"-2W"), IResult::Done(&b""[..], &b"-2W"[..])); 
    assert_eq!(parameter(b"1969-12-28 00:00"), IResult::Done(&b""[..], &b"1969-12-28 00:00"[..])); 
    assert_eq!(parameter(b"1969-12-28"), IResult::Done(&b""[..], &b"1969-12-28"[..])); 
    assert_eq!(parameter(b"13:37"), IResult::Done(&b""[..], &b"13:37"[..])); 
    assert_eq!(parameter(b"'some expression'"), IResult::Done(&b""[..], &b"'some expression'"[..])); 
}

#[test]
fn test_filter(){
    assert_eq!(filter(b"name ~ 'thorsten'"), 
               IResult::Done(&b""[..], filter::Filter::new(String::from("name"), String::from("~"), String::from("'thorsten'")))); 
}

#[test]
fn test_directory_list(){
    let mut exp = Vec::new();
    exp.push(String::from("/home/"));
    exp.push(String::from("/tmp/"));
    exp.push(String::from("/usr/var"));
    assert_eq!(directory_list(b"/home/, /tmp/, /usr/var"), IResult::Done(&b""[..], exp));
}

#[test]
fn test_attribute_list(){
    let mut exp = Vec::new();
    exp.push(String::from("name"));
    exp.push(String::from("basename"));
    exp.push(String::from("mtime"));
    assert_eq!(directory_list(b"name , basename, mtime"), IResult::Done(&b""[..], exp));
}

#[test]
fn test_query(){
    let mut exp_dir_list = Vec::new();
    exp_dir_list.push(String::from("/home"));
    exp_dir_list.push(String::from("/tmp"));

    let mut exp_attr_list = Vec::new();
    exp_attr_list.push(String::from("name"));
    exp_attr_list.push(String::from("basename"));
    exp_attr_list.push(String::from("mtime"));

    let exp_filter = vec![filter::Filter::new(String::from("name"), String::from("~"), String::from("'.git'"))];

    let command = String::from("rm -rf {}");

    let exp_query = query::Query::new(Some(exp_attr_list), Some(exp_dir_list), Some(exp_filter), Some(command));

    assert_eq!(query(b"name, basename, mtime from /home, /tmp where name ~ '.git' exec rm -rf {}"), IResult::Done(&b""[..], exp_query));
}

// fn main() {
//     let res = query(b"name, size from /home/timm.behner/Documents, /tmp/other where name ~ 'theodor' exec cp {} /home/timm.behner");

//     match res {
//         IResult::Done(i, o) => {
//             println!("I: {:?}", i);
//             for r in o.directories {
//                 println!("Directory: {:?}", String::from_utf8_lossy(r));
//             }
//             for r in o.attributes {
//                 println!("Attribute: {:?}", String::from_utf8_lossy(r));
//             }
//             for f in &o.filters {
//                 println!("Next filter:");
//                 println!("O: {:?}", str::from_utf8(f.attribute).unwrap());
//                 println!("O: {:?}", str::from_utf8(f.op).unwrap());
//                 println!("O: {:?}", str::from_utf8(f.param).unwrap());
//             }
//         },
//         IResult::Error(e)   => println!("Error {}", e),
//         _                   => println!("Something strange came back!"),
//     }
// }
