pub struct Client {
    // must keep Fork alive to prevent the destructor from running
    #[allow(dead_code)] pty_fork: pty::fork::Fork,
    pty_master: pty::fork::Master,
    pty_child: nix::unistd::Pid,
    vt100_parser: vt100::Parser,
    pty_master_fd: i32,
}

fn get_char_unbuffered(fd: i32) -> u8 {
    let mut buffer = [0u8];
    loop {
        let n = unsafe {
            libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, 1)
        };
        if n == 1 {
            break;
        }
    }
    buffer[0]
}

fn set_size_termios(fd: i32, rows: u16, cols: u16) {
    //use nix::ioctl_write_ptr;
    //ioctl_write_ptr!(ioctl_tiocswinsz, 't', 103, libc::winsize);
    //ioctl_tiocswinsz(fd, &ws).unwrap();
    let ws = libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0
    };
    unsafe {
        let res = libc::ioctl(
            fd,
            libc::TIOCSWINSZ, &ws as *const libc::winsize as u64
            );
        assert_eq!(res, 0);
    }
 }

#[allow(dead_code)]
impl Client {
    pub fn new(config: &ClientBuilder) -> Client {
        use std::io::Write;
        use std::os::unix::io::AsRawFd;
        use std::os::unix::process::CommandExt;
        let fork = pty::fork::Fork::from_ptmx().unwrap();
        match fork {
            pty::fork::Fork::Parent(pid, mut master) => {
                let fd = master.as_raw_fd().try_into().unwrap();
                set_size_termios(fd, config.rows, config.cols);
                let parser = vt100::Parser::new(config.rows, config.cols, 0);
                // signal we're done with termios configuration
                master.write_all("\n".as_bytes()).unwrap();
                Client {
                    pty_fork: fork,
                    vt100_parser: parser,
                    pty_master: master,
                    pty_child: nix::unistd::Pid::from_raw(pid),
                    pty_master_fd: fd
                }
            }
            pty::fork::Fork::Child(ref slave) => {
                // wait for termios configuration signal
                get_char_unbuffered(slave.as_raw_fd().try_into().unwrap());
                std::process::Command::new(config.command.to_owned())
                    .args(config.args.to_owned())
                    .exec();
                panic!("Could not exec bash");
            }
        }
    }

    pub fn set_size(&mut self, rows: u16, cols: u16) {
        self.vt100_parser.set_size(rows, cols);
        set_size_termios(self.pty_master_fd, rows, cols);
    }

    pub fn process(&mut self) {
        use std::io::Read;
        let mut buffer = [0u8; 128];
        let n = self.pty_master.read(&mut buffer).expect("could not read");
        self.vt100_parser.process(&buffer[0..n]);
    }

    pub fn process_ms(&mut self, ms: u64) {
        let start = std::time::SystemTime::now();
        while (start.elapsed().unwrap().as_millis() as u64) < ms {
            self.process();
        }
    }

    pub fn to_stdout(&self) {
        let (_rows, cols) = self.vt100_parser.screen().size();
        println!("+{:-<width$}+", "", width=cols as usize);
        for row in self.vt100_parser.screen().rows(0, cols) {
            println!("|{: <width$}|", row, width=cols as usize);
        }
        println!("+{:-<width$}+", "", width=cols as usize);
    }

    pub fn is_running(&self) -> bool {
        use nix::sys::wait::{waitpid,WaitPidFlag,WaitStatus};
        match waitpid(self.pty_child, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => true,
            Ok(_status) => false,
            Err(_err) => false
        }
    }

    pub fn kill(&mut self) {
        use nix::sys::signal::SIGTERM;
        nix::sys::signal::kill(self.pty_child, SIGTERM).unwrap();
    }

    pub fn wait(&mut self) {
        while self.is_running() {
            self.process();
        }
        self.process();
    }

    pub fn send_str(&mut self, input: &str) {
        use std::io::Write;
        self.pty_master.write_all(input.as_bytes()).unwrap();
    }

    pub fn send_bytes(&mut self, input: &[u8]) {
        use std::io::Write;
        self.pty_master.write_all(input).unwrap();
    }
}

pub struct ClientBuilder {
    rows: u16,
    cols: u16,
    command: String,
    args: Vec<String>

}

#[allow(dead_code)]
impl ClientBuilder {
    pub fn new() -> Self {
        ClientBuilder {
            rows: 24,
            cols: 80,
            command: "bash".to_string(),
            args: Vec::new()
        }
    }
    pub fn size(self, rows: u16, cols: u16) -> Self {
        Self { rows, cols, ..self }
    }
    pub fn cmd(self, command: &str) -> Self {
        Self { command: command.to_owned(), args: Vec::new(), ..self }
    }
    pub fn arg(self, arg: &str) -> Self {
        let mut args = self.args;
        args.push(arg.to_owned());
        Self { args, ..self }
    }
    pub fn build(&self) -> Client {
        Client::new(&self)
    }
}
