extern crate clap;
extern crate iso8601;
extern crate xml;

use clap::{Arg, App};
use std::fs::File;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};

fn main() {
    struct Segment {
        offset: String,
        timeslot: [u32; 2]
    }

    let mut segment_vector: Vec<Segment> = Vec::new();

    let keyword = "silence";
    let mut chapter = 0;
    let mut part = 0;
    let mut first_row: bool = true;
    let mut print_json: bool = false;

    // Command line parameters.
    let matches = App::new("parse-pause")
        .version("0.1")
        .about("parse pause")
        .author("Claus Guttesen")
        .arg(Arg::with_name("file")
            .help("input filename")
            .required(true)
            .takes_value(true)
            .short("f")
            .long("filename")
            .multiple(false)
        )
        .arg(Arg::with_name("chapter")
            .help("chapter transition duration in seconds")
            .required(true)
            .takes_value(true)
            .short("c")
            .long("chapter")
            .multiple(false)
        )
        .arg(Arg::with_name("part")
            .help("part transition duration in seconds")
            .required(true)
            .takes_value(true)
            .short("p")
            .long("part")
            .multiple(false)
        )
        .arg(Arg::with_name("split")
            .help("duration in seconds before splitting a chapter into parts")
            .required(true)
            .takes_value(true)
            .short("s")
            .long("split")
            .multiple(false)
        )
        .get_matches();

    // convert string to int https://www.programming-idioms.org/idiom/22/convert-string-to-integer/1163/rust
    let chapter_transition = match matches.value_of("chapter").unwrap().parse::<u32>() {
        Ok(i) => i * 1000,
        Err(_) => {
            3 * 1000
        }
    };

    let part_transition = match matches.value_of("part").unwrap().parse::<u32>() {
        Ok(i) => {
            if i > chapter_transition {
                (chapter_transition - 1) * 1000
            } else {
                i * 1000
            }
        },
        Err(_) => {
            1
        }
    };

    let max_chapter_duration = match matches.value_of("split").unwrap().parse::<u32>() {
        Ok(i) => i * 1000,
        Err(_) => {
            300 * 1000
        }
    };


    let file = matches.value_of("file").unwrap();
    let file = File::open(file).unwrap();
    let file = BufReader::new(file);
    let parser = EventReader::new(file);

    // Initialise with first offset (chapter).
    let segment = Segment{ offset: "PT0S".to_string(), timeslot: [0,chapter_transition] };
    segment_vector.push(segment);

    // Parse xml.
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                let name = name.local_name;
                if name.eq(keyword) {
                    //print!("{}", name);  // Tag; ie. silence

                    // name can be [ from | until ]. value is iso8601-formatted duration.
                    // Use an array to store from at [0] and until at [1].

                    let mut index = 0;
                    let mut segment = Segment{ offset: "".to_string(), timeslot: [0, 0]};

                    for attribute in attributes {
                        //print!(":{}={}", attribute.name, attribute.value);
                        let _ = match iso8601::duration(&attribute.value) {
                            Err(e) => {
                                print!("Invalid date: {}", e);
                            },
                            Ok(v) => {
                                //print!("Date: {:?}", v);
                                match v {
                                    iso8601::Duration::YMDHMS{year, month, day, hour, minute, second, millisecond} => {
                                        let _ = year;
                                        let _ = month;
                                        let _ = day;
                                        let milliseconds = (hour * 3600 + minute * 60 + second) * 1000 + millisecond;
                                        //print!("hour: {}, minute: {}, second: {}, millisecond: {}, milliseconds: {}", hour, minute, second, millisecond, milliseconds);
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
                    //println!();
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

    // Traverse the vector and divide into chapters and parts.

    let mut previous_from_timestamp = 0;
    let mut chapter_duration = 0;
    let mut part_duration = 0;

    print!("{{");
    print!("\"segments\": [");

    for segment in segment_vector {

        let pause_duration: u32 = segment.timeslot[1] - segment.timeslot[0];

        // If pause duration is long enough to mark a chapter, or a chapter duration is long enough it can be split into parts.
        if (pause_duration >= chapter_transition) || (segment.timeslot[0] - previous_from_timestamp > max_chapter_duration) {
            chapter += 1;
            part = 1;
            print_json = true;
            chapter_duration = segment.timeslot[0] - previous_from_timestamp;
        }

        if pause_duration > part_transition && pause_duration < chapter_transition {
            part += 1;
            print_json = true;
            part_duration = segment.timeslot[0] - previous_from_timestamp;
        }

        if print_json {

            // Don't print comma (,) on first item to make it valid json.
            if first_row == true {
                // Do nothing.
                first_row = false;
            } else {
                print!(",");
            }

            print!("{{");
            print!("\"title\": \"Chapter {}, part {}\"", chapter, part);
            print!(", ");
            print!("\"offset\": \"{}\", \"pause_duration\": \"{}\"", segment.offset, pause_duration);
            print!(", ");
            print!("\"chapter_duration\": \"{}\", \"part_duration\": \"{}\"", chapter_duration, part_duration);
            print!("}}");
            print_json = false;
        }

        previous_from_timestamp = segment.timeslot[0];

    }
    print!("]}}");
}
