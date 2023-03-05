use std::collections::HashMap;

#[derive(Debug, Default)]
struct State {
    trees: HashMap<String, Vec<Completion>>,
}

#[derive(Debug, Clone)]
struct Completion {
    value: CompleteValue,
    children: Vec<Completion>,
}

#[derive(Debug, Clone)]
enum CompleteValue {
    Fixed(String),
    Value(CompleteType),
    Prefixed {
        prefix: String,
        suffix: CompleteType,
    },
}

#[derive(Debug, Clone)]
enum CompleteType {
    Any,
    File,
}

fn complete(state: &State, args: &[&str]) -> Vec<Completion> {
    if args.is_empty() {
        return vec![];
    }

    // Find the tree to complete using the program name.
    let Some(tree) = state.trees.get(args[0]) else {
        return vec![];
    };
    let mut args = &args[1..];

    while !args.is_empty() {
        for branch in tree {
            println!("{:?}", branch);
        }
    }

    tree.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut state = State::default();
        state.trees.insert(
            "program".into(),
            vec![
                Completion {
                    value: CompleteValue::Fixed("--fixed".into()),
                    children: vec![],
                },
                Completion {
                    value: CompleteValue::Value(CompleteType::File),
                    children: vec![],
                },
                Completion {
                    value: CompleteValue::Prefixed {
                        prefix: "--file=".into(),
                        suffix: CompleteType::File,
                    },
                    children: vec![],
                },
            ],
        );

        println!("{:?}", complete(&state, &["program", "--asd"]));
    }
}
