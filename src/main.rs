use std::process::{Command, Stdio};

fn main() {
    let mut child = Command::new("vd")
    .arg("~/Downloads/dedupe-example.csv")
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .spawn()
    .expect("failed to start VisiData");
    // fetch the data and store it in private file
    // visidata should respect the data and show properly
    // watch the changes at this file
    println!("Visidata started!");
    child.wait().expect("VisiData process failed");
    println!("Stopped Visidata");

}
