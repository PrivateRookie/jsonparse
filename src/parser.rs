use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_till1, take_while_m_n},
    character::complete::multispace0,
    combinator::{map, peek, value as n_value},
    error::context,
    multi::separated_list,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair},
    IResult,
};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Str(String),
    Boolean(bool),
    Null,
    Num(f64),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

fn normal(i: &str) -> IResult<&str, &str> {
    take_till1(|c: char| c == '\\' || c == '"' || c.is_ascii_control())(i)
}

fn parse_hex(i: &str) -> IResult<&str, &str> {
    context(
        "hex string",
        preceded(
            peek(tag("u")),
            take_while_m_n(5, 5, |c: char| c.is_ascii_hexdigit() || c == 'u'),
        ),
    )(i)
}

fn escapable(i: &str) -> IResult<&str, &str> {
    context(
        "escaped",
        alt((
            tag("\""),
            tag("\\"),
            tag("/"),
            tag("b"),
            tag("f"),
            tag("n"),
            tag("r"),
            tag("t"),
            parse_hex,
        )),
    )(i)
}

fn parse_str(i: &str) -> IResult<&str, &str> {
    escaped(normal, '\\', escapable)(i)
}

fn string(i: &str) -> IResult<&str, &str> {
    context(
        "string",
        alt((tag("\"\""), delimited(tag("\""), parse_str, tag("\"")))),
    )(i)
}

fn boolean(i: &str) -> IResult<&str, bool> {
    let parse_true = n_value(true, tag("true"));
    let parse_false = n_value(false, tag("false"));
    alt((parse_true, parse_false))(i)
}

fn null(i: &str) -> IResult<&str, JsonValue> {
    map(tag("null"), |_| JsonValue::Null)(i)
}

fn value(i: &str) -> IResult<&str, JsonValue> {
    context(
        "value",
        delimited(
            multispace0,
            alt((
                map(object, JsonValue::Object),
                map(array, JsonValue::Array),
                map(string, |s| JsonValue::Str(String::from(s))),
                map(double, JsonValue::Num),
                map(boolean, JsonValue::Boolean),
                null,
            )),
            multispace0,
        ),
    )(i)
}

fn array(i: &str) -> IResult<&str, Vec<JsonValue>> {
    context(
        "array",
        delimited(
            tag("["),
            separated_list(tag(","), delimited(multispace0, value, multispace0)),
            tag("]"),
        ),
    )(i)
}

fn key(i: &str) -> IResult<&str, &str> {
    delimited(multispace0, string, multispace0)(i)
}

fn object(i: &str) -> IResult<&str, HashMap<String, JsonValue>> {
    context(
        "object",
        delimited(
            tag("{"),
            map(
                separated_list(
                    tag(","),
                    separated_pair(key, tag(":"), delimited(multispace0, value, multispace0)),
                ),
                |tuple_vec: Vec<(&str, JsonValue)>| {
                    tuple_vec
                        .into_iter()
                        .map(|(k, v)| (String::from(k), v))
                        .collect()
                },
            ),
            tag("}"),
        ),
    )(i)
}

pub fn root(i: &str) -> IResult<&str, JsonValue> {
    delimited(
        multispace0,
        alt((map(object, JsonValue::Object), map(array, JsonValue::Array))),
        multispace0,
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::{error::ErrorKind, Err};

    #[test]
    fn test_parse_string() {
        assert_eq!(string(r#""hello""#), Ok(("", "hello")));
        assert_eq!(string(r#""he\rllo""#), Ok(("", r"he\rllo")));
        assert_eq!(string(r#""he\tllo""#), Ok(("", r"he\tllo")));
        assert_eq!(string(r#""he\u1234""#), Ok(("", r"he\u1234")));
        assert_eq!(string(r#""""#), Ok(("", r#""""#)));
    }

    #[test]
    fn test_array() {
        let v: Vec<JsonValue> = vec![];
        assert_eq!(array("[]"), Ok(("", v)));
        assert_eq!(
            array(r#"["abc"   , null, true,  false]"#),
            Ok((
                "",
                vec![
                    JsonValue::Str(String::from("abc")),
                    JsonValue::Null,
                    JsonValue::Boolean(true),
                    JsonValue::Boolean(false)
                ]
            ))
        );
    }

    #[test]
    fn test_object() {
        assert_eq!(object(r#"{}"#), Ok(("", HashMap::new())));
        let mut hash = HashMap::new();
        hash.insert(String::from("key"), JsonValue::Str(String::from("val")));
        hash.insert(
            String::from("arr"),
            JsonValue::Array(vec![
                JsonValue::Boolean(true),
                JsonValue::Boolean(false),
                JsonValue::Null,
            ]),
        );
        assert_eq!(
            object(r#"{"key": "val"  , "arr" :    [true, false, null]}"#),
            Ok(("", hash))
        );
    }

    #[test]
    fn test_value() {
        assert_eq!(value("true"), Ok(("", JsonValue::Boolean(true))));
        assert_eq!(value("false"), Ok(("", JsonValue::Boolean(false))));
        assert_eq!(value("null"), Ok(("", JsonValue::Null)));
        assert_eq!(
            value(r#""\b\\\"\f\n\r\n\t\u1234""#),
            Ok((
                "",
                JsonValue::Str(String::from("\\b\\\\\\\"\\f\\n\\r\\n\\t\\u1234"))
            ))
        );
        assert_eq!(
            value(r#"["abc", true, false, null]"#),
            Ok((
                "",
                JsonValue::Array(vec![
                    JsonValue::Str(String::from("abc")),
                    JsonValue::Boolean(true),
                    JsonValue::Boolean(false),
                    JsonValue::Null
                ])
            ))
        );
        let hashmap = vec![
            (String::from("key"), JsonValue::Str(String::from("val"))),
            (String::from("arr"), JsonValue::Array(vec![])),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            value(r#"{"key": "val", "arr": []}"#),
            Ok(("", JsonValue::Object(hashmap)))
        )
    }

    #[test]
    fn test_parse_str() {
        assert_eq!(parse_str(r#"\u1234"#), Ok(("", r#"\u1234"#)));
        assert_eq!(
            parse_str(r#"\b\\\"\f\n\r\n\t\u1234"#),
            Ok(("", r#"\b\\\"\f\n\r\n\t\u1234"#))
        );
        assert_eq!(
            parse_str(r#""abcd"#),
            Err(Err::Error(("\"abcd", ErrorKind::Escaped)))
        );
    }

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex(r#"u1234"#), Ok(("", r#"u1234"#)));
        assert_eq!(parse_hex(r#"u12346"#), Ok(("6", r#"u1234"#)));
        assert_eq!(
            parse_hex(r#"u1g34"#),
            Err(Err::Error((r#"u1g34"#, ErrorKind::TakeWhileMN)))
        );
    }
}
