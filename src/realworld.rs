use std::io::{self, Read, Write};

pub struct RealWorld {
    pub stdin: io::Stdin,
    pub stdout: io::Stdout,
    pub stderr: io::Stderr,
}

impl Default for RealWorld {
    fn default() -> RealWorld {
        RealWorld {
            stdin: io::stdin(),
            stdout: io::stdout(),
            stderr: io::stderr(),
        }
    }
}

impl Read for RealWorld {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdin.lock().read(buf)
    }
}

impl Write for RealWorld {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.lock().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
