extern crate clap;
extern crate iso8601;
extern crate xml;

use clap::{Arg, App};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use xml::reader::{EventReader, XmlEvent};
use std::collections::HashMap;

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

    #[derive(Copy, Clone)]
    struct Silence {
        from_ms: u32,
        until_ms: u32    }

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
                    let silence = Silence{from_ms: from_ms, until_ms: until_ms};
                    //println!("pause-duration: {}", until_ms - from_ms);
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

    // Part-duration and silence-duration. Total duration is with silence_until_ms. silence_from_ms is implicitly until_ms.
    #[derive(Copy, Clone)]
    struct Part {
        from_ms: u32,
        until_ms: u32,
        silence_until_ms: u32
    }
    
    let mut part_vector: Vec<Part> = Vec::new();
    let mut previous_from_ms: u32 = 0;

    // Traverse silence-vector and calculate part-duration and push to part-vector.
    for silence in &silence_vector {
        let part = Part{
            from_ms: previous_from_ms,
            until_ms: silence.from_ms,
            silence_until_ms: silence.until_ms
        };
        part_vector.push(part);
        previous_from_ms = silence.until_ms;
        //println!("part-duration: {}", duration2string(part.duration as u64));
    }

    struct Chapter {
        chapter: u32,
        part: u32,
        from_ms: u32,
        until_ms: u32
    }

    let mut chapter_vector: Vec<Chapter> = Vec::new();
    let mut chapter = 1;
    let mut part = 1;

    let segment = Chapter{
        chapter, part, from_ms: 0, until_ms: 0
    };
    chapter_vector.push(segment);
    
    // Check whether a pause is long enough to make a chapter or part, or if a part is long enough to treat it like a chapter.
    for p in part_vector {
        let silent_duration_ms = p.silence_until_ms - p.until_ms;
        let part_duration = p.until_ms - p.from_ms;
        if (silent_duration_ms >= chapter_transition) || (part_duration > max_chapter_duration) {
            chapter += 1;
            part = 1;
            let ch = Chapter{
                chapter, part, from_ms: p.from_ms, until_ms: p.silence_until_ms
            };
            chapter_vector.push(ch);
        } else if silent_duration_ms > part_transition && silent_duration_ms < chapter_transition {
            part += 1;
            let ch = Chapter{
                chapter, part, from_ms: p.from_ms, until_ms: p.silence_until_ms
            };
            chapter_vector.push(ch);
        }
    }

    // Now we have chapters and parts. But duration between parts is not aligned since not all parts
    // designate a chapter or a part. This produces gaps in the output. Traverse the chapter-vector to
    // properly align duration of chapter and part.
    struct Segment {
        chapter: u32,
        part: u32,
        offset_ms: u32,
        duration_ms: u32
    }

    let mut chapter_hashmap: HashMap<u32, u32> = HashMap::new();
    let mut segment_vector: Vec<Segment> = Vec::new();
    let mut previous_chapter = 1;
    let mut previous_chapter_timestamp = 0;
    let mut last_part_timestamp = 0;

    // First put chapter in hashmap so we get chapter-duration.
    // Then align offset.
    for (i, chapter) in chapter_vector.iter().enumerate() {
        if chapter.chapter > previous_chapter {
            chapter_hashmap.insert(previous_chapter, chapter.until_ms - previous_chapter_timestamp);
            previous_chapter = chapter.chapter;
            previous_chapter_timestamp = chapter.from_ms;
        }

        let duration = match chapter_vector.get(i + 1) {
            Some(duration) => duration.from_ms,
            None => chapter_vector[i].until_ms
        };
        let segment = Segment{
            chapter: chapter.chapter,
            part: chapter.part,
            offset_ms: chapter.from_ms,
            duration_ms: duration - chapter.from_ms
        };
        segment_vector.push(segment);
        last_part_timestamp = chapter.from_ms;
    }
    chapter_hashmap.insert(previous_chapter, last_part_timestamp - previous_chapter_timestamp);

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

        let chapter = match chapter_hashmap.get(&segment.chapter) {
            Some(chapter) => chapter,
            _ => &1
        };

        print!("{{");
        print!("\"title\": \"Chapter {}, part {}\"", segment.chapter, segment.part);
        print!(", ");
        print!("\"offset\": \"{}\"", duration2string(segment.offset_ms as u64));
        print!(", ");
        print!("\"part_duration\": \"{}\"", duration2string(segment.duration_ms as u64));
        print!(", ");
        print!("\"chapter_duration\": \"{}\"", duration2string(*chapter as u64));
        print!("}}");
    }

    print!("]}}");

    for chapter in chapter_hashmap {
        println!("{}, {}", chapter.0, duration2string(chapter.1 as u64));
    }
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
