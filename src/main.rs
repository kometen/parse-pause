extern crate clap;
extern crate xml;

use clap::{Arg, App};
use std::fs::File;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};

fn main() {
    println!("Jeg æder blåbærsyltetøj!");

    let keyword = "silence";

    // Command line parameters.
    let matches = App::new("compare")
        .version("0.1")
        .about("compare sets")
        .author("Claus Guttesen")
        .arg(Arg::with_name("file")
            .help("input filename")
            .required(true)
            .takes_value(true)
            .short("f")
            .long("filename")
            .multiple(false)
        )
        .get_matches();

    let file = matches.value_of("file").unwrap();
    let file = File::open(file).unwrap();
    let file = BufReader::new(file);
    let parser = EventReader::new(file);

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                let name = name.local_name;
                if name.eq(keyword) {
                    print!("{}", name);
                    for attribute in attributes {
                        print!(":{}={}", attribute.name, attribute.value);
                    }
                    println!();
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
