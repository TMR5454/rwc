use getopts::Options;
use rwc::wc::Rwc;

fn parse_opt() -> Options {
    let mut opts = Options::new();
    opts.optopt("", "files0-from", "read input from the files specified by\n\tNUL-terminated names in file F;\n\tIf F is - then read names from standard input", "F");
    opts.optflag("c", "bytes", " print the byte counts");
    opts.optflag("m", "chars", "print the character counts");
    opts.optflag("l", "lines", "print the newline counts");
    opts.optflag("L", "max-line-length", "print the maximum display width");
    opts.optflag("w", "words", " print the word counts");
    opts.optflag("h", "help", "print this help menu");

    opts
}

fn main() {
    let opts = parse_opt();
    match Rwc::with_opts(opts).exec() {
        Ok(_) => {},
        Err(_) => {
            std::process::exit(1);
        }
    }
}
