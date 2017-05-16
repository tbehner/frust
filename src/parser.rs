//! frust query parser
//!
//! # Talkin' Syntax
//! For now the precendence of 'and' and 'or' is not implemented, nor are brackets supported.
//! This meens that if you want (f1 and f2) or f3, you have to express it as f3 or f1 and f2.
//! I know this is limitting and I'm going to improve this in the future...

use std::string::String;
use filter;
use filter_tree;
use query;

named!(komma, ws!(tag!(",")));
named!(comp_op<filter::CompOp>, alt!(
          tag!("<")  => { |_| filter::CompOp::Lower }  
        | tag!("<=") => { |_| filter::CompOp::LowerEqual }
        | tag!("==") => { |_| filter::CompOp::Equal }
        | tag!("not =") => { |_| filter::CompOp::Equal }
        | tag!(">=") => { |_| filter::CompOp::GreaterEqual }
        | tag!(">")  => { |_| filter::CompOp::Greater }
        | tag!("~")  => { |_| filter::CompOp::Like }
        | tag!("not ~")  => { |_| filter::CompOp::Unlike }
        )
    );

named!(logic_op<filter_tree::LogicOp>, 
       alt!(
        ws!(tag!("or")) =>  { |_| filter_tree::LogicOp::Or }
        | 
        ws!(tag!("and")) => { |_| filter_tree::LogicOp::And }
        )
);

named!(from_keyword, ws!(tag!("from")));
named!(where_keyword, ws!(tag!("where")));
named!(exec_keyword, ws!(tag!("exec")));
named!(attribute<filter::Attribute>, 
       do_parse!(
           tag: ws!(alt!( 
                      tag!("name")     => { |_| filter::Attribute::Name }
                    | tag!("size")     => { |_| filter::Attribute::Size }
                    | tag!("mtime")    => { |_| filter::Attribute::Mtime }
                    | tag!("ctime")    => { |_| filter::Attribute::Ctime }
                    | tag!("atime")    => { |_| filter::Attribute::Atime }
                    | tag!("filetype") => { |_| filter::Attribute::Filetype }
                    | tag!("mimetype") => { |_| filter::Attribute::Mimetype }
                    | tag!("inode")    => { |_| filter::Attribute::Inode }
                    | tag!("basename") => { |_| filter::Attribute::Basename }
                )
               ) >>
               (tag)
           )
       );

named!(directory<String>, 
       do_parse!(
           dir: re_bytes_find!("(/)?([^/\0,; ]+(/)?)*") >>
           (String::from_utf8_lossy(dir).into_owned())
       )
   );
named!(num_paramter, re_bytes_find!("-?[0-9]+(\\.[0-9]*)?[a-zA-Z]"));
named!(str_paramter, 
       do_parse!(
           tag!("'") >> 
           pattern: re_bytes_find!("[^']+") >>
           tag!("'") >>
           (pattern)
           )
       );
named!(date_parameter, re_bytes_find!("(([0-9]{4}-[0-9]{2}-[0-9]{2})? ?[0-9]{1,2}:[0-9]{2})|([0-9]{4}-[0-9]{2}-[0-9]{2})"));

named!(parameter, alt!(num_paramter | str_paramter | date_parameter));

named!(filter<filter_tree::FilterTuple>, 
       do_parse!(
           attr: attribute >>
           op  : ws!(comp_op)   >>
           param: ws!(parameter) >>
           (filter_tree::FilterTuple::new(attr, 
                                op, 
                                String::from_utf8_lossy(param).into_owned()
                                )
           )
     )
);

named!(directory_list<Vec<String>>, separated_list!(komma, directory));
named!(attribute_list<Vec<filter::Attribute>>, separated_list!(komma, attribute));

named!(filter_expr<filter_tree::FilterTree>,   
           do_parse!(
               lhs : filter         >>
               op  : opt!(logic_op) >>
               rhs : opt!(filter_expr)   >>
               ( match rhs {
                   Some(r) => filter_tree::FilterTree::new(Some(lhs), op, Some(Box::new(r))),
                   None    => filter_tree::FilterTree::new(Some(lhs), None, None),
                 }
               )
           )
       );

named!(command<String>, 
       do_parse!(
       cmd: re_bytes_find!("[^;]+") >>
        (String::from_utf8_lossy(cmd).into_owned())
    )
);

named!(select_part<Option<Vec<filter::Attribute>>>,
       opt!(
           do_parse!(
               attrs: attribute_list >>
               (attrs)
           )
       )
   );

named!(from_part<Option<Vec<String>>>,
       opt!(
           do_parse!(
               from_keyword >>
               dirs: directory_list >>
               (dirs)
           )
       )
   );

named!(where_part<Option<filter_tree::FilterTree>>,
       opt!(
           do_parse!(
               where_keyword >>
               filters: filter_expr >>
               (filters)
           )
       )
   );

named!(exec_part<Option<String>>,
       opt!(
           do_parse!(
               exec_keyword >>
               cmd : command >>
               (cmd)
           )
       )
   );

named!(pub query<query::Query>, do_parse!(
        attributes: select_part >> 
        directories: from_part >>
        filters: where_part >>
        command: exec_part >>
        (query::Query::new(attributes, directories, filters, command))
        )
    );
