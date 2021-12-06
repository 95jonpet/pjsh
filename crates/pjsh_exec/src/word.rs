use pjsh_ast::Word;
use pjsh_core::Context;

pub fn interpolate_word(word: Word, context: &Context) -> String {
    match word {
        Word::Literal(literal) => literal,
        Word::Quoted(quoted) => quoted,
        Word::Variable(key) => match key.as_str() {
            "?" => context.last_exit.to_string(),
            "0" => todo!(),
            _ => {
                if let Ok(positional) = key.parse::<usize>() {
                    return context
                        .arguments
                        .get(positional - 1)
                        .map(String::to_owned)
                        .unwrap_or_else(String::new);
                }

                context.scope.get_env(&key).unwrap_or_default()
            }
        },
        Word::Interpolation(units) => {
            let mut output = String::new();

            for unit in units {
                match unit {
                    pjsh_ast::InterpolationUnit::Literal(literal) => output.push_str(&literal),
                    pjsh_ast::InterpolationUnit::Unicode(ch) => output.push(ch),
                    pjsh_ast::InterpolationUnit::Variable(variable) => {
                        output.push_str(&context.scope.get_env(&variable).unwrap_or_default())
                    }
                }
            }

            output
        }
    }
}
