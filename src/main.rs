mod parser;

fn main() {
    let data = "  { \"a\"\t: 42,
    \"b\": [ \"x\", \"y\", 12 ] ,
    \"c\": { \"hello\" : \"world\"
    }
    } ";

    println!(
        "will try to parse valid JSON data:\n\n**********\n{}\n**********\n",
        data
    );
    println!("parsing a valid file:\n{:#?}\n", parser::root(data));

    let data = "  { \"a\"\t: 42,
    \"b\": [ \"x\", \"y\", 12 ] ,
    \"c\": { 1\"hello\" : \"world\"
    }
    } ";
    println!(
        "will try to parse invalid JSON data:\n\n**********\n{}\n**********\n",
        data
    );
    println!(
        "basic errors - `root::<(&str, ErrorKind)>(data)`:\n{:#?}\n",
        parser::root(data)
    );
}
