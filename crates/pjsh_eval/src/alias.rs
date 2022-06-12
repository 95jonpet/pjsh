use std::{collections::VecDeque, sync::Arc};

use parking_lot::Mutex;
use pjsh_core::Context;

pub(crate) fn replace_aliases(args: &mut VecDeque<String>, ctx: Arc<Mutex<Context>>) {
    if args.is_empty() {
        return;
    }

    let mut is_aliasing = true;
    while is_aliasing {
        let first_arg = args.pop_front().unwrap();
        if let Some(alias) = ctx.lock().aliases.get(&first_arg) {
            args.push_front(alias.to_owned());
        } else {
            args.push_front(first_arg);
            is_aliasing = false;
        }
    }
}
