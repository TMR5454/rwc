use getopts::{Matches, Options};
use std::cmp::max;
use std::env;
use std::fs::File;
use std::io::ErrorKind;
use std::io::{self, BufRead};
use std::ops::{Add, AddAssign};
use std::path::Path;

use crate::nightly::unlikely;

#[derive(Default, Debug, Clone)]
struct Wc {
    path: String, /* file name */
    bytes: u32,
    chars: u32,
    lines: u32,
    words: u32,
    max_line_length: u32,
}

impl Wc {
    fn with_path(path: &str) -> Self {
        Self {
            path: path.to_string(),
            ..Self::default()
        }
    }
    fn new() -> Self {
        Self::with_path("")
    }
    fn analyze(&mut self, file: &File) -> std::io::Result<()> {
        let mut reader = io::BufReader::new(file);
        let mut line = String::with_capacity(256);

        loop {
            let size = reader.read_line(&mut line)?;
            if size == 0 {
                break;
            }
            self.bytes += size as u32;
            self.chars += line.chars().count() as u32;
            self.lines += 1;
            self.words += line.split_whitespace().count() as u32;
            self.max_line_length = max(
                self.max_line_length,
                line.chars()
                    .map(|c| {
                        if c.len_utf8() > 1 {
                            2
                        } else if c == '\t' {
                            8
                        } else if unlikely(c == '\n') {
                            /* not count new-line */
                            0
                        } else {
                            1
                        }
                    })
                    .sum(),
            );
            line.clear(); /* read_line() appending to String but read per line */
        }

        Ok(())
    }
    fn print_result(&self, matches: &Matches) {
        if matches.opt_present("l") {
            print!(" {}", self.lines);
        }
        if matches.opt_present("m") {
            print!(" {}", self.words);
        }
        if matches.opt_present("c") {
            print!(" {}", self.chars);
        }
        if matches.opt_present("w") {
            print!(" {}", self.bytes);
        }
        if matches.opt_present("L") {
            print!(" {}", self.max_line_length);
        }

        if !env::args().any(|s| s.starts_with('-')) {
            /* no option */
            print!(" {} {} {}", self.lines, self.chars, self.bytes);
        }

        println!(" {}", self.path);
    }
    fn run(&mut self) -> std::io::Result<()> {
        let file = File::open(&self.path)?;
        let path = Path::new(&self.path);

        if path.is_dir() {
            return Err(io::Error::new(ErrorKind::Unsupported, "Is a directory"));
        }

        self.analyze(&file)?;

        Ok(())
    }
}

impl Add for Wc {
    type Output = Wc;
    fn add(self, rhs: Wc) -> Wc {
        Wc {
            path: self.path,
            bytes: self.bytes + rhs.bytes,
            chars: self.chars + rhs.chars,
            lines: self.lines + rhs.lines,
            words: self.words + rhs.words,
            max_line_length: self.max_line_length,
        }
    }
}

impl AddAssign<&Wc> for Wc {
    fn add_assign(&mut self, rhs: &Self) {
        self.bytes += rhs.bytes;
        self.chars += rhs.chars;
        self.lines += rhs.lines;
        self.words += rhs.words;
    }
}

#[derive(Default)]
pub struct Rwc {
    opts: Options,
    wcvec: Vec<Wc>,
}

impl Rwc {
    pub fn with_opts(opts: Options) -> Self {
        Self {
            opts: opts,
            ..Self::default()
        }
    }

    pub fn new() -> Self {
        Self::with_opts(Options::new())
    }

    fn print_usage(&self, program: &str) {
        let brief = format!("Usage: {} FILE [options]", program);
        print!("{}", self.opts.usage(&brief));
    }

    pub fn exec(&mut self) -> std::io::Result<()> {
        let args: Vec<String> = env::args().collect();

        let matches = match self.opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                panic!("{}", f.to_string())
            }
        };

        if matches.opt_present("h") {
            let program = args[0].clone();
            self.print_usage(&program);
            return Ok(());
        }

        let mut total = Wc::with_path("total");

        for file in &matches.free {
            let mut wc = Wc::with_path(file);

            match wc.run() {
                Ok(_) => {
                    total += &wc;
                }
                Err(e) => {
                    eprintln!("{}: {}", &wc.path, e.to_string());
                }
            }
            wc.print_result(&matches);
            self.wcvec.push(wc);
        }

        if matches.free.len() > 1 {
            /* multiple files */
            total.print_result(&matches);
        }

        Ok(())
    }
}
