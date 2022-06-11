use std::collections::VecDeque;

enum Statement {
    Command(String),
    If(bool, String),
    Repeat(usize, String),
    For(
        usize,
        Box<dyn Fn(usize) -> bool>,
        Box<dyn Fn(usize) -> usize>,
        Box<dyn Fn(usize) -> String>,
    ),
}

struct Executor {
    statements: VecDeque<Statement>,
}

impl Executor {
    fn execute(&mut self) {
        while let Some(statement) = self.statements.pop_front() {
            for next in self.execute_statement(statement).into_iter().rev() {
                self.statements.push_front(next);
            }
        }
    }

    fn execute_statement(&self, statement: Statement) -> Vec<Statement> {
        match statement {
            Statement::Command(command) => {
                println!("Command: {command}");
                Vec::new()
            }
            Statement::If(false, _) => Vec::new(),
            Statement::If(true, command) => vec![Statement::Command(command)],
            Statement::Repeat(0, _) => Vec::new(),
            Statement::Repeat(i, command) => vec![
                Statement::Command(command.clone()),
                Statement::Repeat(i - 1, command),
            ],
            Statement::For(i, condition, next_i, body) => {
                if !condition(i) {
                    return Vec::new();
                }

                vec![
                    Statement::Command(body(i)),
                    Statement::For(next_i(i), condition, next_i, body),
                ]
            }
        }
    }
}

#[test]
fn it_works() {
    let statements = VecDeque::from([
        Statement::Command("Command".into()),
        Statement::If(false, "Should not be executed".into()),
        Statement::If(true, "Should be executed".into()),
        Statement::Repeat(3, "Repeat 3 times".into()),
        Statement::For(
            0,
            Box::new(|i| i < 5),
            Box::new(|i| i + 1),
            Box::new(|i| format!("i={i}")),
        ),
    ]);

    let mut executor = Executor { statements };
    executor.execute();
}
