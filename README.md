# parse-pause
Detect duration in an xml-file.

Clone repo and open a terminal windows at the top of the source folder. Then build and run the application.

$ cargo build  
$ ./target/debug/parse-pause -f pause.xml -c 4 -p 1

Duration is in seconds. Internally it is using milliseconds.

It reads the xml-file, add `from` and `until` into a struct which is then pushed to a vector. This vector is traversed
at the end and print json (handcoded) if chapter or part duration is satisfied.

TODO:

Need to split long chapters into smaller ones.
