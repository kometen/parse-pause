# parse-pause
Detect chapters and parts based on iso8601-durations in an xml-file.

Clone repo and open a terminal window at the top of the source folder. Then build and run the application.

$ `cargo build`

$ `./target/debug/parse-pause -f pause.xml -c 4 -p 1 -s 220`

Duration as parameters is in seconds. Internally it is using milliseconds.

It reads the xml-file, add `from` and `until` into a struct which is then pushed to a vector. This vector is traversed
and length of a spoken part is derived from the silent parts and put in another struct, along with the silence-duration that
immediately follows and pushed to another vector. This vector is traversed and when a chapter or part occurs based
on command line input put to a third struct and vector. Then traversed and print json (handcoded) to stdout.

Input:

```xml
<?xml version="1.0" encoding="UTF-8"?>  
<silences>  
    <silence from="PT1M44.025S" until="PT1M44.914S" />  
    <silence from="PT3M39.714S" until="PT3M40.251S" />  
    <silence from="PT5M58.959S" until="PT6M0.988S" />  
    <silence from="PT7M32.452S" until="PT7M34.431S" />  
</silences>
```

Output:

```json
{
  "segments": [
    {
      "title": "Chapter 1, part 1",
      "offset": "PT0.000S",
      "part_duration": "PT3M40.250S",
      "chapter_duration": "PT34M13.082S"
    },
    {
      "title": "Chapter 1, part 2",
      "offset": "PT3M40.250S",
      "part_duration": "PT2M20.738S",
      "chapter_duration": "PT34M13.082S"
    },
    {
      "title": "Chapter 1, part 3",
      "offset": "PT6M0.988S",
      "part_duration": "PT1M33.443S",
      "chapter_duration": "PT34M13.082S"
    }
  ]
}
```
