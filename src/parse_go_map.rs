use anyhow::Result;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::char,
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, separated_pair},
};
use serde_json::Value;

// map[contained:true match_str:S50_32SITES score_thresh:0.5]

pub fn parse_go_map(input: &str) -> Result<Value> {
    match go_map(input) {
        Ok((_, v)) => Ok(v),
        Err(_) => Err(anyhow::anyhow!("Failed to parse Go map")),
    }
}

fn go_map(input: &str) -> IResult<&str, Value> {
    match delimited(
        tag("map["),
        separated_list0(char(' '), parse_key_value),
        char(']'),
    )
    .parse(input)
    {
        Ok((rest, kvs)) => {
            let mut map = serde_json::Map::new();
            for (key, value) in kvs {
                map.insert(key, value);
            }
            Ok((rest, Value::Object(map)))
        }
        Err(e) => Err(e),
    }
}

fn parse_key_value(input: &str) -> IResult<&str, (String, Value)> {
    match separated_pair(take_till(|c| c == ':'), char(':'), parse_value).parse(input) {
        Ok((rest, (key, value))) => Ok((rest, (key.to_string(), value))),
        Err(e) => Err(e),
    }
}

fn parse_value(input: &str) -> IResult<&str, Value> {
    // maybe another map
    alt((go_map, parse_string_value)).parse(input)
}

fn parse_string_value(input: &str) -> IResult<&str, Value> {
    map(take_till(|c| c == ' ' || c == ']'), |s: &str| {
        if let Ok(n) = s.parse::<i64>() {
            Value::Number(n.into())
        } else if let Ok(f) = s.parse::<f64>() {
            Value::Number(serde_json::Number::from_f64(f).unwrap())
        } else if s == "true" {
            Value::Bool(true)
        } else if s == "false" {
            Value::Bool(false)
        } else {
            Value::String(s.to_string())
        }
    })
    .parse(input)
}
