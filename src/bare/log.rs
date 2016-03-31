//! Simple logging module, capable of logging in color.
use std::io;
use term;
use term::StdoutTerminal;
use term::color;
use term::color::Color;

pub struct Writer {
    term: Box<StdoutTerminal>,
}

impl Writer {
    pub fn new() -> Self { Writer { term: term::stdout().unwrap() } }

    pub fn write(&mut self, text: &str) -> io::Result<usize> {
        self.term.write(text.as_bytes())
    }

    pub fn writeln(&mut self, text: &str) -> io::Result<usize> {
        let r = self.write(text);
        self.write("\n").unwrap();
        r
    }

    pub fn write_color(&mut self,
                       text: &str,
                       color: Color) -> io::Result<usize> {
        self.term.fg(color).unwrap();
        let r = self.write(text);
        self.term.reset().unwrap();
        r
    }

    pub fn writeln_color(&mut self,
                         text: &str,
                         color: Color) -> io::Result<usize> {
        self.term.fg(color).unwrap();
        let r = self.write(&format!("{}\n", text));
        self.term.reset().unwrap();
        r
    }
}




pub struct RainbowLog {
    writer: Writer,
}

impl RainbowLog {
    pub fn new() -> Self { RainbowLog {  writer: Writer::new()  } }

    fn log(&mut self, color: Color, tag: &str, message: &str) {
        self.writer.write(&format!("[")).unwrap();
        self.writer.term.fg(color).unwrap();
        self.writer.write(&format!("{}", tag)).unwrap();
        self.writer.term.reset().unwrap();
        self.writer.write(&format!("] {}", message)).unwrap();
    }

    pub fn error(&mut self, message: &str) {
        self.log(color::RED, "E", message);
    }

    pub fn warn(&mut self, message: &str) {
        self.log(color::YELLOW, "W", message);
    }

    pub fn info(&mut self, message: &str) {
        self.log(color::BRIGHT_GREEN, "I", message);
    }

    pub fn debug(&mut self, message: &str) {
        self.log(color::BRIGHT_BLUE, "D", message);
    }
}
