
# Epstain
A tool to scrape and download all, and only, PDF files disclosed by the United States Department of Justice.
This does not un-redact reports, however, ✨we might later.✨
###### Please note: this _should_ work on Windows. I don't have access to a Windows machine, so, I'm not sure.

---

# Building
### Non-technical operators:
Please contact your favorite technical friend.

### Technical operators:
1. Install Rust from the official [distribution](https://rust-lang.org/tools/install/).
2. `cargo build --release`

This has not been tested on Mac OS nor Windows. I welcome contributions.


### Notes

* This **only** works with top-level resources ex. `https://.../epstein/${RESOURCE}`
* This **does not** work with the "Oversight Committee Releases Epstein Records Provided by the Department of Justice" subsection.
That link's a proxy to a google drive, I trust you can figure that one.

# Usage
```shell
$ epstain -c 3 -u https://www.justice.gov/epstein/court-records
$ epstain -c 3 -u https://www.justice.gov/epstein/doj-disclosures
$ epstain -c 5 -u https://www.justice.gov/epstein/foia
```
or the traditional

```shell
$ cargo run -- -c 3 -u https://www.justice.gov/epstein/court-records
```
