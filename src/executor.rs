use os_pipe::{pipe, PipeReader, PipeWriter};

use crate::builtins;
use crate::lexer::Lexer;
use crate::parser::FileDescriptor;
use crate::parser::Parser;
use crate::parser::{Cmd, SingleCommand};
use crate::shell::Shell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

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

pub struct Executor {
    aliases: HashMap<String, String>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }

    pub fn execute(&mut self, ast: Cmd, capture: bool) -> Option<String> {
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

    fn visit(&mut self, node: Cmd, stdio: CmdMeta) -> bool {
        match node {
            Cmd::Single(single) => self.visit_single(single, stdio),
            Cmd::Not(cmd) => self.visit_not(*cmd, stdio),
            Cmd::Pipeline(left, right) => self.visit_pipe(*left, *right, stdio),
            Cmd::And(left, right) => self.visit_and(*left, *right, stdio),
            Cmd::Or(left, right) => self.visit_or(*left, *right, stdio),
            Cmd::NoOp => true,
        }
    }

    fn visit_not(&mut self, cmd: Cmd, stdio: CmdMeta) -> bool {
        !self.visit(cmd, stdio)
    }

    fn visit_and(&mut self, left: Cmd, right: Cmd, stdio: CmdMeta) -> bool {
        if self.visit(left, CmdMeta::inherit()) {
            self.visit(right, stdio)
        } else {
            false
        }
    }

    fn visit_or(&mut self, left: Cmd, right: Cmd, stdio: CmdMeta) -> bool {
        if !self.visit(left, CmdMeta::inherit()) {
            self.visit(right, stdio)
        } else {
            false
        }
    }

    fn visit_pipe(&mut self, left: Cmd, right: Cmd, stdio: CmdMeta) -> bool {
        let (reader, writer) = pipe().unwrap();
        self.visit(left, CmdMeta::pipe_out(writer));
        self.visit(right, stdio.new_in(reader))
    }

    fn visit_single(&mut self, mut single: SingleCommand, stdio: CmdMeta) -> bool {
        self.reconcile_io(&mut single, stdio);
        match &single.cmd[..] {
            "." => {
                let file = single.args.get(0).unwrap();
                let shell = Rc::new(RefCell::new(Shell::from_file(PathBuf::from(file))));
                loop {
                    let input = shell.borrow_mut().next();
                    if let Some(line) = input {
                        let lexer = Lexer::new(&line, Rc::clone(&shell));
                        let mut parser = Parser::new(lexer, Rc::clone(&shell));
                        match parser.get() {
                            Ok(command) => {
                                self.execute(command, false);
                            }
                            Err(e) => {
                                eprintln!("ERROR: {}", e);
                            }
                        }
                    } else {
                        break;
                    }
                }
                true
            }
            "alias" => builtins::alias(&mut self.aliases, single.env, single.args),
            "cd" => builtins::cd(single.args),
            "exit" => builtins::exit(single.args),
            command => {
                let mut cmd = self.resolve_command(String::from(command), single.args);
                // TODO Move alias builtin resolution to after alias resolution.

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

                cmd.envs(single.env);

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

    fn resolve_command(&self, cmd: String, args: Vec<String>) -> Command {
        let mut args = VecDeque::from(args.clone());
        args.push_front(cmd); // Args here should contain command + args.
        let mut visited = HashSet::new();

        while let Some(alias) = self.aliases.get(args.front().unwrap()) {
            if alias.starts_with("\\") || visited.contains(alias) {
                break;
            }

            visited.insert(alias);
            let mut local: Vec<String> = Vec::new();

            for segment in alias.split_ascii_whitespace() {
                local.push(String::from(segment));
            }

            args.remove(0); // Remove the currently unwrapped alias.
            for segment in local.iter().rev() {
                args.push_front(segment.clone());
            }
        }

        let mut program = args.remove(0).unwrap();
        program = String::from(program.strip_prefix("\\").unwrap_or(&program));
        let mut command = Command::new(program);
        command.args(args);
        command
    }
}
