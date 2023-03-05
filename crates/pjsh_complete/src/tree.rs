#[derive(Debug)]
struct Command {
    name: String,
    commands: Vec<Command>,
    flags: Vec<Flag>,
    arguments: Vec<Argument>,
}

#[derive(Debug)]
struct Flag {
    name: String,
}

#[derive(Debug)]
struct Argument {
    kind: ArgumentKind,
    repeat: bool,
}

#[derive(Debug)]
enum ArgumentKind {
    File,
    String,
}

#[derive(Debug)]
enum Match {
    File,
    Any,
    String(String),
}

fn complete(tree: &Vec<Command>, args: &[&str]) -> Vec<Match> {
    if args.is_empty() {
        return vec![];
    }

    let mut tree = tree;
    let mut args = args;
    let mut parent = None;
    while let Some(head) = args.first() {
        let Some(cmd) = tree.iter().find(|cmd| cmd.name == *head) else {
            break;
        };

        parent = Some(cmd);
        tree = &cmd.commands;
        args = &args[1..];
    }

    let Some(parent) = parent else {
        return vec![]; // Did not reach anything.
    };

    let mut completions = Vec::new();
    for cmd in &parent.commands {
        completions.push(Match::String(cmd.name.clone()));
    }

    for arg in &parent.arguments {
        match arg.kind {
            ArgumentKind::File => completions.push(Match::File),
            ArgumentKind::String => completions.push(Match::Any),
        }
    }

    for flag in &parent.flags {
        completions.push(Match::String(flag.name.clone()));
    }

    completions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let tree = vec![Command {
            name: "git".into(),
            commands: vec![
                Command {
                    name: "add".into(),
                    commands: vec![],
                    flags: vec![],
                    arguments: vec![Argument {
                        kind: ArgumentKind::File,
                        repeat: true,
                    }],
                },
                Command {
                    name: "show".into(),
                    commands: vec![],
                    flags: vec![],
                    arguments: vec![Argument {
                        kind: ArgumentKind::String,
                        repeat: false,
                    }],
                },
            ],
            flags: vec![Flag {
                name: "--verbose".into(),
            }],
            arguments: vec![],
        }];

        println!("{:?}", complete(&tree, &["git", "show", "a"]))
    }
}
