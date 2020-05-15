extern crate clap;
extern crate iso8601;
extern crate xml;

use clap::{Arg, App};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use xml::reader::{EventReader, XmlEvent};

fn main() {
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

    // A unit of time, either from or until in the xml-file. Example:
    // <silence from="PT3M9S" until="PT3M11S" />
    // String is in iso8601-format, u32 is the format converted to milliseconds.

    //struct Timestamp(String, u32);

    #[derive(Copy, Clone)]
    struct Silence {
        from_ms: u32,
        until_ms: u32,
        duration: u32
    }

    let mut silence_vector: Vec<Silence> = Vec::new();
    
    let keyword = "silence";

    let file = matches.value_of("file").unwrap();
    let file = File::open(file).unwrap();
    let file = BufReader::new(file);
    let parser = EventReader::new(file);

    // Parse xml.
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                let name = name.local_name;
                if name.eq(keyword) {
                    //print!("{}", name);  // Tag; ie. silence

                    let mut from_ms: u32 = 0;
                    let mut until_ms = 0;

                     // name can be [ from | until ]. value is iso8601-formatted duration.

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
                                        //print!("h: {}, m: {}, s: {}, ms: {}, total ms: {}", hour, minute, second, millisecond, milliseconds);
                                        if attribute.name.to_string() == "from".to_string() {
                                            from_ms = milliseconds;
                                        } else if attribute.name.to_string() == "until".to_string() {
                                            until_ms = milliseconds;
                                        }
                                    },
                                    iso8601::Duration::Weeks(w) => print!("weeks: {}", w)
                                };
                            }
                        };
                    }
                    //println!();
                    let silence_duration = until_ms - from_ms;
                    let silence = Silence{from_ms: from_ms, until_ms: until_ms, duration: silence_duration};
                    silence_vector.push(silence);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    #[derive(Copy, Clone)]
    struct Part {
        from_ms: u32,
        until_ms: u32,
        duration: u32
    }
    
    let mut part_vector: Vec<Part> = Vec::new();
    let mut previous_from_ms: u32 = 0;
    let mut current_from_ms :u32;

    // Traverse silence-vector and calculate part-duration (and from- and until-ms) and push it to part-vector.
    // This way we get the duration of a part.
    for silence in &silence_vector {
        current_from_ms = silence.from_ms;
        let part = Part{from_ms: previous_from_ms, until_ms: current_from_ms, duration: silence.from_ms - previous_from_ms};
        part_vector.push(part);
        previous_from_ms = current_from_ms;
        //println!("from ms: {}, until ms: {}, duration: {}", silence.from_ms, silence.until_ms, silence.duration);
    }

    struct Segment {
        chapter: u32,
        part: u32,
        offset: String,
        duration: u32
    }

    let mut segment_vector: Vec<Segment> = Vec::new();
    let mut chapter = 1;
    let mut part = 1;
    let mut offset = "PT0S".to_string();

    let segment = Segment{chapter, part, offset, duration: part_vector[0].duration};
    segment_vector.push(segment);
    
    // And then to check whether a pause is long enough to make a chapter or part, or if a part is long enought and treat it like a chapter.
    for (i, silence) in silence_vector.iter().enumerate() {
        if (silence.duration >= chapter_transition) || (part_vector[i].duration > max_chapter_duration) {
            chapter += 1;
            part = 1;
            offset = duration2string(silence.until_ms as u64);
            let segment = Segment{chapter, part, offset, duration: part_vector[i].duration};
            segment_vector.push(segment);
        } else if silence.duration > part_transition && silence.duration < chapter_transition {
            part += 1;
            offset = duration2string(silence.until_ms as u64);
            let segment = Segment{chapter, part, offset, duration: part_vector[i].duration};
            segment_vector.push(segment);
        }
    }

    let mut first_row: bool = true;

    print!("{{");
    print!("\"segments\": [");

    for segment in segment_vector {
        //println!("{}, {}, {}, {}", segment.chapter, segment.part, segment.offset, segment.duration);

        // Don't print comma (,) on first item to make it valid json.
        if first_row == true {
            // Do nothing.
            first_row = false;
        } else {
            print!(",");
        }

        print!("{{");
        print!("\"title\": \"Chapter {}, part {}\"", segment.chapter, segment.part);
        print!(", ");
        print!("\"offset\": \"{}\", \"part_duration_ms\": \"{}\"", segment.offset, segment.duration);
        print!("}}");
    }

    print!("]}}");
}

fn duration2string(d: u64) -> String {
    let duration = Duration::from_millis(d);
    let ms_part = duration.as_millis() - (duration.as_secs() * 1000) as u128;
    let hours = duration.as_secs() / 3600;
    let seconds = duration.as_secs() - hours * 3600 ;
    let minutes = seconds / 60;
    let seconds = seconds - minutes * 60;
    let mut return_string = "PT".to_string();
    if hours > 0 {
        return_string = return_string + &hours.to_string() + &"H";
    }
    if minutes > 0 {
        return_string = return_string + &minutes.to_string() + &"M";
    }
    return_string = return_string + &seconds.to_string() + &"."
     + &format!("{:03}", &ms_part) + &"S";

    return return_string;
}
