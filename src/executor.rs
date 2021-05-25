use os_pipe::{pipe, PipeReader, PipeWriter};

use crate::parser::FileDescriptor;
use crate::parser::{Cmd, SingleCommand};
use std::io::Read;
use std::process::Command;

#[derive(Debug)]
pub struct CmdMeta {
    stdin: Option<PipeReader>,
    stdout: Option<PipeWriter>,
}

impl CmdMeta {
    fn inherit() -> CmdMeta {
        CmdMeta {
            stdin: None,
            stdout: None,
        }
    }

    fn pipe_out(writer: PipeWriter) -> CmdMeta {
        CmdMeta {
            stdin: None,
            stdout: Some(writer),
        }
    }

    fn new_in(self, reader: PipeReader) -> CmdMeta {
        CmdMeta {
            stdin: Some(reader),
            stdout: self.stdout,
        }
    }
}

pub struct Executor {}

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&self, ast: Cmd, capture: bool) -> Option<String> {
        if capture {
            let (mut reader, writer) = pipe().unwrap();
            self.visit(ast, CmdMeta::pipe_out(writer));
            let mut output = String::new();
            reader.read_to_string(&mut output).unwrap();
            Some(output)
        } else {
            self.visit(ast, CmdMeta::inherit());
            None
        }
    }

    fn visit(&self, node: Cmd, stdio: CmdMeta) -> bool {
        match node {
            Cmd::Single(single) => self.visit_single(single, stdio),
            Cmd::Pipeline(left, right) => self.visit_pipe(*left, *right, stdio),
            Cmd::And(left, right) => self.visit_and(*left, *right, stdio),
            Cmd::Or(left, right) => self.visit_or(*left, *right, stdio),
            _ => false,
        }
    }

    fn visit_and(&self, left: Cmd, right: Cmd, stdio: CmdMeta) -> bool {
        if self.visit(left, CmdMeta::inherit()) {
            self.visit(right, stdio)
        } else {
            false
        }
    }

    fn visit_or(&self, left: Cmd, right: Cmd, stdio: CmdMeta) -> bool {
        if !self.visit(left, CmdMeta::inherit()) {
            self.visit(right, stdio)
        } else {
            false
        }
    }

    fn visit_pipe(&self, left: Cmd, right: Cmd, stdio: CmdMeta) -> bool {
        let (reader, writer) = pipe().unwrap();
        self.visit(left, CmdMeta::pipe_out(writer));
        self.visit(right, stdio.new_in(reader))
    }

    fn visit_single(&self, mut single: SingleCommand, stdio: CmdMeta) -> bool {
        self.reconcile_io(&mut single, stdio);
        match &single.cmd[..] {
            command => {
                let mut cmd = Command::new(command);
                cmd.args(&single.args);

                if let Some(stdin) = single.stdin.borrow_mut().get_stdin() {
                    cmd.stdin(stdin);
                } else {
                    return false;
                }
                if let Some(stdout) = single.stdout.borrow_mut().get_stdout() {
                    cmd.stdout(stdout);
                } else {
                    return false;
                }
                if let Some(stderr) = single.stdin.borrow_mut().get_stderr() {
                    cmd.stderr(stderr);
                } else {
                    return false;
                }
                if let Some(env) = single.env {
                    cmd.envs(env);
                }

                match cmd.status() {
                    Ok(child) => child.success(),
                    Err(e) => {
                        eprintln!("pjsh: {}: {}", single.cmd, e);
                        false
                    }
                }
            }
        }
    }

    fn reconcile_io(&self, single: &mut SingleCommand, stdio: CmdMeta) {
        if let Some(stdout) = stdio.stdout {
            if *single.stdout.borrow() == FileDescriptor::Stdout {
                *single.stdout.borrow_mut() = FileDescriptor::PipeOut(stdout);
            }
        }
        if let Some(stdin) = stdio.stdin {
            if *single.stdin.borrow() == FileDescriptor::Stdin {
                *single.stdin.borrow_mut() = FileDescriptor::PipeIn(stdin);
            }
        }
    }
}
