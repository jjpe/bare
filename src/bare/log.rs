//! Simple logging module, capable of logging in color.
use std::io;
use term;
use term::color;
use term::color::Color;
use term::StdoutTerminal;

pub struct Writer {
    term: Box<StdoutTerminal>,
}

impl Writer {
    pub fn new() -> Self {
        Writer {
            term: term::stdout().unwrap(/* Option */),
        }
    }

    pub fn write(&mut self, text: &str) -> io::Result<usize> {
        self.term.write(text.as_bytes())
    }

    pub fn writeln(&mut self, text: &str) -> io::Result<usize> {
        let r = self.write(text);
        self.write("\n")?;
        r
    }

    pub fn write_color(&mut self, text: &str, color: Color) -> io::Result<usize> {
        self.term.fg(color)?;
        let r = self.write(text);
        self.term.reset()?;
        r
    }

    pub fn writeln_color(&mut self, text: &str, color: Color) -> io::Result<usize> {
        self.term.fg(color)?;
        let r = self.write(&format!("{}\n", text));
        self.term.reset()?;
        r
    }
}

pub struct RainbowLog {
    writer: Writer,
}

impl RainbowLog {
    pub fn new() -> Self {
        RainbowLog {
            writer: Writer::new(),
        }
    }

    fn log(&mut self, color: Color, tag: &str, message: &str) -> io::Result<()> {
        self.writer.write(&format!("["))?;
        self.writer.term.fg(color)?;
        self.writer.write(&format!("{}", tag))?;
        self.writer.term.reset()?;
        self.writer.write(&format!("] {}", message))?;
        Ok(())
    }

    pub fn error(&mut self, message: &str) -> io::Result<()> {
        self.log(color::RED, "E", message)
    }

    pub fn warn(&mut self, message: &str) -> io::Result<()> {
        self.log(color::YELLOW, "W", message)
    }

    pub fn info(&mut self, message: &str) -> io::Result<()> {
        self.log(color::BRIGHT_GREEN, "I", message)
    }

    pub fn debug(&mut self, message: &str) -> io::Result<()> {
        self.log(color::BRIGHT_BLUE, "D", message)
    }
}
