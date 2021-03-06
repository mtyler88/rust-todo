//! Parser contains all the parsing logic for the file

use types::{DateTime, Time, Item};
use nom::{be_u8, be_u32, digit, IResult, non_empty, rest};
use std::str::{from_utf8, FromStr};

/// Get a number from an array of u8
named!(pub get_num<u32>, 
    map_res!(
        map_res!(digit,
            from_utf8
        ),
        FromStr::from_str
    ) 
);

/// Get a time, from a string of hh:mm, colon-optional!
named!(pub get_time<Time>, do_parse!(
        h: flat_map!(take!(2), get_num) >>
        opt!(tag!(":")) >>
        m: flat_map!(take!(2), get_num) >>
        (Time{hours: h as u8, minutes: m as u8})
        ));

/// Get a datetime struct from a string, dates separated by / or -, and optional time which can be
/// separated by t/T
named!(pub get_datetime<DateTime>, do_parse!(
        y: flat_map!(take!(4), get_num) >>
        one_of!("-/") >>
        m: flat_map!(take!(2), get_num) >>
        one_of!("-/") >>
        d: flat_map!(take!(2), get_num) >>
        t: opt!(preceded!(one_of!("tT"), get_time)) >>
        (DateTime{year: y, month: m as u8, day: d as u8, time: t})
        ));

/// Parse todo, either ([], [ ], [x])
named!(pub todo_box<bool>, 
       map!(delimited!(tag!("["), opt!(one_of!(" xX")), tag!("]")),
       |c| { match c {
            Some('x') | Some('X') => true,
            _ => false,
       }})
       );

/// Parse an item's header text
named!(pub item_head<String>, map_res!(map_res!(take_until_and_consume!(";;"), from_utf8), FromStr::from_str));

/// Parse an item's body text
named!(pub item_body<String>, map_res!(map_res!(rest, from_utf8), FromStr::from_str));

/// Parses a todo list item, fully consisting of:
/// [x] Line awlkdjhlkjhvr v ;; :2016/12/13T13:00:
///  alkwjdhalkfjhoisfh poishpogposugmpoeirug pwoeiug pwoireugpoiusf
named!(pub parse_item<Item>, do_parse!(
        todo: opt!(ws!(todo_box)) >>
        text: ws!(item_head) >>
        time: opt!(complete!(ws!(delimited!(tag!(":"), get_datetime ,tag!(":"))))) >>
        description: map!(opt!(complete!(ws!(item_body))), |c| {
            if c==Some(String::new()) { None } else { c }
        }) >>
        (Item{
            todo: todo, 
            text: text,
            time: time,
            description: description,
            children: vec!(),
        })
        ));

/// Count the number of double dashes (--) there are
named!(pub count_dash<usize>, do_parse!(
        dash_count: many1!(complete!(tag!("--"))) >>
        (dash_count.len())));

/// Matches with a set of dashes and a line
named!(pub match_line<(usize, &[u8] ) >, do_parse!(
        indentation: count_dash >>
        text: alt_complete!(take_until!("\n--") | rest) >>
        opt!(complete!(tag!("\n"))) >>
        (indentation, text)
        ));

/// Match a set of lines and return a vector of tuples containing the indentation and the text
named!(pub match_lines<Vec<(usize, &[u8])> >, alt_complete!(many1!(match_line) | value!(vec!(), ws!(eof!()))));

/// Function to convert a tuple (usize, &[u8]) into an Option<(usize, Item)>
pub fn convert_item_tup(ini_tup: (usize, &[u8])) -> Option<(usize, Item)> {
    use nom::IResult::Done;
    let item_byte_str = ini_tup.1;
    match parse_item(item_byte_str) {
        Done(_, item) => Some((ini_tup.0, item)),
        _ => None,
    }
}

/// Convert a vector of line tuples, keeping the ones that parse correctly
pub fn convert_vec_items(v: Vec<(usize, &[u8])>) -> Vec<(usize, Item)> {
    let mut parsed_v: Vec<(usize, Item)> = vec!(); 
    v.iter().for_each(|x| match convert_item_tup(*x) { Some(i) => parsed_v.push(i), _ => ()}) ;
    parsed_v
}

/// Match a set of lines and parse each line, returning a vector of tuples with (indentation, Item)
named!(pub read_lines_and_parse<Vec<(usize, Item)> >, map!(match_lines, convert_vec_items));
