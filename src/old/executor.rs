use os_pipe::{pipe, PipeReader, PipeWriter};

use crate::builtin_utils::cd::Cd;
use crate::builtin_utils::Builtin;
use crate::old::builtins;
// use crate::builtins;
use super::parser::FileDescriptor;
use super::parser::{Cmd, SimpleCommand};
use super::shell::Shell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::env;
use std::io::Read;
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
    pub shell: Rc<RefCell<Shell>>,
    aliases: HashMap<String, String>,
}

impl Executor {
    pub fn new(shell: Rc<RefCell<Shell>>) -> Self {
        Self {
            shell,
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
            Cmd::Simple(simple) => self.visit_simple(simple, stdio),
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

    fn visit_simple(&mut self, mut simple: SimpleCommand, stdio: CmdMeta) -> bool {
        self.reconcile_io(&mut simple, stdio);
        match &simple.cmd[..] {
            "alias" => builtins::alias(&mut self.aliases, simple.env, simple.args),
            "bg" => unimplemented!(),
            "cd" => Cd::execute(&simple.args, &self).is_ok(),
            "command" => unimplemented!(),
            "false" => unimplemented!(),
            "fc" => unimplemented!(),
            "fg" => unimplemented!(),
            "getopts" => unimplemented!(),
            "hash" => unimplemented!(),
            "jobs" => unimplemented!(),
            "kill" => unimplemented!(),
            "newgrp" => unimplemented!(),
            "pwd" => unimplemented!(),
            "read" => unimplemented!(),
            "true" => unimplemented!(),
            "umask" => unimplemented!(),
            "unalias" => unimplemented!(),
            "wait" => unimplemented!(),

            "export" => {
                if let Some(variable_name) = simple.args.first() {
                    if let Some(variable_value) = self.shell.borrow_mut().get_var(variable_name) {
                        env::set_var(variable_name, variable_value);
                        true
                    } else {
                        unimplemented!()
                    }
                } else {
                    unimplemented!()
                }
            }

            "source" | "." => builtins::source(&simple.args, self),
            "exit" => builtins::exit(simple.args),
            command => {
                let mut full_env = self.shell.borrow().vars.clone();
                full_env.extend(simple.env);

                let mut cmd = self.resolve_command(String::from(command), simple.args);
                // TODO Move alias builtin resolution to after alias resolution.

                if let Some(stdin) = simple.stdin.borrow_mut().get_stdin() {
                    cmd.stdin(stdin);
                } else {
                    return false;
                }
                if let Some(stdout) = simple.stdout.borrow_mut().get_stdout() {
                    cmd.stdout(stdout);
                } else {
                    return false;
                }
                if let Some(stderr) = simple.stdin.borrow_mut().get_stderr() {
                    cmd.stderr(stderr);
                } else {
                    return false;
                }

                cmd.envs(full_env);

                match cmd.status() {
                    Ok(child) => child.success(),
                    Err(e) => {
                        eprintln!("pjsh: {}: {}", simple.cmd, e);
                        false
                    }
                }
            }
        }
    }

    fn reconcile_io(&self, simple: &mut SimpleCommand, stdio: CmdMeta) {
        if let Some(stdout) = stdio.stdout {
            if *simple.stdout.borrow() == FileDescriptor::Stdout {
                *simple.stdout.borrow_mut() = FileDescriptor::PipeOut(stdout);
            }
        }
        if let Some(stdin) = stdio.stdin {
            if *simple.stdin.borrow() == FileDescriptor::Stdin {
                *simple.stdin.borrow_mut() = FileDescriptor::PipeIn(stdin);
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
