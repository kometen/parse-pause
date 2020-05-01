extern crate clap;
extern crate iso8601;
extern crate xml;

use clap::{Arg, App};
use std::fs::File;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};

fn main() {
    println!("Jeg æder blåbærsyltetøj!");

    struct Segment {
        offset: String,
        timeslot: [u32; 2]
    }

    let mut segment_vector: Vec<Segment> = Vec::new();

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

    // Initialise with first offset (chapter).
    let segment = Segment{ offset: "PT0S".to_string(), timeslot: [0,0] };
    segment_vector.push(segment);

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                let name = name.local_name;
                if name.eq(keyword) {
                    print!("{}", name);  // Tag; ie. silence

                    // name can be [ from | until ]. value is iso8601-formatted duration.
                    // Use an array to store from at [0] and until at [1].

                    let mut index = 0;
                    let mut segment = Segment{ offset: "".to_string(), timeslot: [0, 0]};

                    for attribute in attributes {
                        print!(":{}={}", attribute.name, attribute.value);
                        let _ = match iso8601::duration(&attribute.value) {
                            Err(e) => {
                                print!("Invalid date: {}", e);
                            },
                            Ok(v) => {
                                //print!("Date: {:?}", v);
                                match v {
                                    iso8601::Duration::YMDHMS{year, month, day, hour, minute, second, millisecond} => {
                                        let milliseconds = (hour * 3600 + minute * 60 + second) * 1000 + millisecond;
                                        print!("hour: {}, minute: {}, second: {}, millisecond: {}, milliseconds: {}", hour, minute, second, millisecond, milliseconds);
                                        // Only store from in offset.
                                        if index == 0 { segment.offset = attribute.value; }
                                        segment.timeslot[index] = milliseconds;
                                    },
                                    iso8601::Duration::Weeks(w) => print!("weeks: {}", w)
                                };
                                index += 1;
                            }
                        };
                    }
                    println!();
                    segment_vector.push(segment);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    for segment in segment_vector {
        let duration: u32 = segment.timeslot[1] - segment.timeslot[0];
        println!("offset: {}, duration: {}", segment.offset, duration);
    }
}
