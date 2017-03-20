#[macro_use] extern crate nom;

extern crate colored;
extern crate oauth_client;
extern crate core;

use std::collections::HashMap;
use nom::{IResult};
use std::fmt;
use colored::*;
use std::str::from_utf8;
use oauth_client::Token;
use std::u32;
use core::char::from_u32;

pub fn get_credentials(contents: &str) -> (Token, Token) {
    let mut iter = contents.split_whitespace();
    iter.next();
    let api_key = iter.next().expect("");
    iter.next();
    let api_sec = iter.next().expect("");
    iter.next();
    let tok = iter.next().expect("");
    iter.next();
    let tok_sec = iter.next().expect("");
    let key = oauth_client::Token::new(api_key, api_sec);
    let token = oauth_client::Token::new(tok,tok_sec);
    (key, token)
}

// TODO consider making this a methd?
// HOT TAKE: oop is just functional programming where composition is backwards
fn replace_specials(string: &str) -> String {
    let result = string.replace("\\/", "/").replace("\\n","\n").replace("\\\"","\"");
    result
}

fn replace_unicode(string: &str) -> char {
    let num_int = u32::from_str_radix(&string[2..6], 16)
        .expect("Failed to parses hexadecimal");
    from_u32(num_int)
        .expect("Failed to convert to unicode")
}

/// Display profile for a given user. Takes screen name and number of tweets to return as
/// parameters. 
///
/// Note that Twitter's API allow for a maximum of 3200 tweets at a time by this method. 
///
/// # Examples
/// 
/// ```
/// print_profile(realDonaldTrump, 100, API_KEY, TOKEN);
/// ```
pub fn print_profile(screen_name: &str, num: u8, api_key: Token, token: Token) {
    let mut param = HashMap::new();
    let num_str = num.to_string();
    let _ = param.insert("screen_name".into(), screen_name.into());
    let _ = param.insert("count".into(), num_str.into()); // TODO accept number of tweets to get
    let bytes_raw = oauth_client::get(api::USER_PROFILE, &api_key, Some(&token), Some(&param)).unwrap();
    // convert vector of u8's to &[u8] (array slice)
    let resp = String::from_utf8(bytes_raw).unwrap();
    let bytes_slice = resp.as_bytes();
    let parsed_maybe = parse_tweets(bytes_slice);
    if let IResult::Done(_,parsed) = parsed_maybe {
        for i in 0..parsed.len() {
            println!("{}", parsed[i]);
        }
    }
    else {
        println!("Parse error");
    }
}

/// Send a tweet
///
/// # Examples
///
/// ```
/// tweet("having a good day :)", API_KEY, TOKEN);
/// ```
pub fn tweet(sent_text: &str, api_key: Token, token: Token) {
    let mut param = HashMap::new();
    let _ = param.insert("status".into(), sent_text[0..140].into());
    let bytes_raw = oauth_client::post(api::STATUS_UPDATE, &api_key, Some(&token), Some(&param)).unwrap();
    let resp = String::from_utf8(bytes_raw).unwrap();
    let bytes_slice = resp.as_bytes();
    let parsed_maybe = parse_tweets(bytes_slice);
    if let IResult::Done(_,parsed) = parsed_maybe {
        for i in 0..parsed.len() {
            println!("{}", parsed[i]);
        }
    }
    else {
        println!("Parse error");
    }
}

// .as_slice() to convert char vector to str
fn char_vector_to_string(v: Vec<char>) -> String {
    let s:String = v.into_iter().collect();
    s
}

/// Display timeline. Takes number of tweets to return as
/// a parameter. 
///
/// Note that Twitter's API allow for a maximum of 3200 tweets at a time by this method. 
///
/// # Examples
/// 
/// ```
/// print_timeline(5, API_KEY, TOKEN);
/// ```
pub fn print_timeline(num: u8, api_key: Token, token: Token) {
    let num_str = num.to_string();
    let mut param = HashMap::new();
    let _ = param.insert("count".into(), num_str.into()); 
    let bytes_raw = oauth_client::get(api::TIMELINE, &api_key, Some(&token), Some(&param)).unwrap();
    // convert vector of u8's to &[u8] (array slice)
    let resp = String::from_utf8(bytes_raw).unwrap();
    let bytes_slice = resp.as_bytes();
    let parsed_maybe = parse_tweets(bytes_slice);
    if let IResult::Done(_,parsed) = parsed_maybe {
        for i in 0..parsed.len() {
            println!("{}", parsed[i]);
        }
    }
    else {
        println!("Parse error");
    }
}

pub struct Tweet<'a>{
    pub text: &'a[u8],
    pub name: &'a[u8],
    pub retweets: &'a[u8],
    pub favorites: &'a[u8],
}

impl<'a> fmt::Display for Tweet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let heart = "\u{1F49C}".red(); // \u{2665}
        let retweets = "\u{F079}".green(); // \u{267A}
        write!(f, "{}\n    {}\n    {} {} {}  {}\n", 
               replace_specials((from_utf8(self.name)).unwrap()).yellow(), 
               replace_specials((from_utf8(self.text)).unwrap()), 
               heart,
               from_utf8(self.favorites).unwrap(),
               retweets,
               from_utf8(self.retweets).unwrap())
    }
}
named!(prefield, take_until!("\",")); // FIXME this mostly works.
named!(field, delimited!(char!('"'), prefield, char!('"')));
named!(int_field, take_until!(","));
named!(text_value,
  do_parse!(
    take_until!("\"text\"") >>
    tag!("\"text\":") >>
    value: field >>
    (value)
  )
);
named!(unicode_char,
  do_parse!(
    tag!("\\u") >>
    num: take!(4) >>
    (num)
  )
);
//from_u32 is what we want I think
named!(special_char,
  do_parse!(
    char!('\\') >>
    value: take!(1) >>
    (value)
  )
);
named!(name_value,
  do_parse!(
    take_until!("\"name\"") >>
    tag!("\"name\":") >> // fix so it doesn't take the first 
    value: field >>
    (value)
  )
);
named!(retweets_value,
  do_parse!(
    take_until!("\"retweet_count\"") >>
    tag!("\"retweet_count\":") >>
    value: int_field >>
    (value)
  )
);
named!(favorites_value,
  do_parse!(
    take_until!("\"favorite_count\"") >>
    tag!("\"favorite_count\":") >>
    value: int_field >>
    (value)
  )
);
named!(step_parse<&[u8], Tweet >,
  do_parse!(
    get_text: text_value >>
    get_name: name_value >>
    get_retweets: retweets_value >>
    get_favorites: favorites_value >>
    (Tweet{text: get_text, name: get_name, retweets: get_retweets, favorites: get_favorites })
  )
);
named!(big_parser<&[u8], Vec<Tweet> > , many0!(step_parse)); 

// consider an example? long though that may be
/// Parse a string of bytes as a vector of tweets
pub fn parse_tweets(str_in: &[u8]) -> IResult<&[u8], Vec<Tweet>> {
    big_parser(str_in)
}

/// urls for the twitter api 
pub mod api {
    pub const USER_PROFILE: &'static str = "https://api.twitter.com/1.1/statuses/user_timeline.json";
    pub const TIMELINE: &'static str = "https://api.twitter.com/1.1/statuses/home_timeline.json";
    pub const STATUS_UPDATE: &'static str = "https://api.twitter.com/1.1/statuses/update.json";
}
