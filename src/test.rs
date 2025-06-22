use polars::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Person {
    name: String,
    age: i32,
}

fn main() -> PolarsResult<()> {
    let df = df![
        "name" => &["Alice", "Bob"],
        "age" => &[30, 40],
    ]?;
    df.
    let people: Vec<Person> = df.deserialize()?;

    for person in people {
        println!("{:?}", person);
    }

    Ok(())
}