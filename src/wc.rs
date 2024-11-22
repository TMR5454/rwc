use getopts::{Matches, Options};
use std::cmp::max;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::io::{self, BufRead};
use std::ops::{Add, AddAssign};

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

enum InputFrom {
    File(fs::File),
    Stdin,
}

impl InputFrom {
    fn stream(self) -> Box<dyn std::io::Read> {
        match self {
            InputFrom::File(file) => Box::new(file),
            InputFrom::Stdin => Box::new(io::stdin()),
        }
    }
}

impl Wc {
    fn with_path(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Self::default()
        }
    }
    fn new() -> Self {
        /* if path.is_empty() then read from stdin */
        Self::with_path("")
    }
    fn analyze<T: std::io::Read>(&mut self, stream: T) -> std::io::Result<()> {
        let mut reader = io::BufReader::new(stream);
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
    fn print_result(&self, matches: &Matches, width: usize) {
        if matches.opt_present("l") {
            print!(" {:>width$}", self.lines);
        }
        if matches.opt_present("m") {
            print!(" {:>width$}", self.words);
        }
        if matches.opt_present("c") {
            print!(" {:>width$}", self.chars);
        }
        if matches.opt_present("w") {
            print!(" {:>width$}", self.bytes);
        }
        if matches.opt_present("L") {
            print!(" {:>width$}", self.max_line_length);
        }

        if !env::args().any(|s| s.starts_with('-')) {
            /* no option */
            print!(" {} {} {}", self.lines, self.chars, self.bytes);
        }

        println!(" {}", self.path);
    }
    fn run(&mut self) -> std::io::Result<()> {
        let input = if self.path.is_empty() {
            InputFrom::Stdin
        } else {
            InputFrom::File(fs::File::open(&self.path)?)
        };

        if let InputFrom::File(file) = &input {
            let metadata = file.metadata()?;
            if metadata.is_dir() {
                return Err(io::Error::new(ErrorKind::Unsupported, "Is a directory"));
            }
        }

        self.analyze(input.stream())?;

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
            max_line_length: max(self.max_line_length, rhs.max_line_length),
        }
    }
}

impl AddAssign<&Wc> for Wc {
    fn add_assign(&mut self, rhs: &Self) {
        self.bytes += rhs.bytes;
        self.chars += rhs.chars;
        self.lines += rhs.lines;
        self.words += rhs.words;
        self.max_line_length = max(self.max_line_length, rhs.max_line_length);
    }
}

pub struct Rwc {
    program: String,
    opts: Options,
    matches: Matches,
    width: usize,
}

impl Rwc {
    fn calc_width(matches: &Matches) -> usize {
        let mut sum = 0;

        for file in &matches.free {
            if let Ok(metadata) = fs::metadata(file) {
                sum += metadata.len();
            }
        }

        sum.to_string().len()
    }
    pub fn with_opts(opts: Options) -> Self {
        let args: Vec<String> = env::args().collect();
        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                panic!("{}", f.to_string())
            }
        };

        Self {
            program: args[0].clone(),
            width: Self::calc_width(&matches),
            opts,
            matches,
        }
    }

    pub fn new() -> Self {
        Self::with_opts(Options::new())
    }

    fn print_usage(&self) {
        let brief = format!("Usage: {} FILE [options]", self.program);
        print!("{}", self.opts.usage(&brief));
    }

    pub fn exec(&mut self) -> std::io::Result<()> {
        if self.matches.opt_present("h") {
            self.print_usage();
            return Ok(());
        }

        if self.matches.free.is_empty() {
            /* from stdin */
            let mut wc = Wc::new();
            wc.run()?;
            wc.print_result(&self.matches, self.width);
            return Ok(());
        }

        let mut total = Wc::with_path("total");

        for file in &self.matches.free {
            let mut wc = Wc::with_path(file);

            match wc.run() {
                Ok(_) => {
                    total += &wc;
                }
                Err(e) => {
                    eprintln!("{}: {}", &wc.path, e);
                }
            }
            wc.print_result(&self.matches, self.width);
        }

        if self.matches.free.len() > 1 {
            /* multiple files */
            total.print_result(&self.matches, self.width);
        }

        Ok(())
    }
}
