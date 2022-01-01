use std::sync::Arc;

use parking_lot::Mutex;
use pjsh_core::{Context, InternalCommand, InternalIo};

use crate::status;

#[derive(Clone)]
pub struct False;
impl InternalCommand for False {
    fn name(&self) -> &str {
        "false"
    }

    fn run(&self, _: &[String], _: Arc<Mutex<Context>>, _: Arc<Mutex<InternalIo>>) -> i32 {
        status::GENERAL_ERROR
    }
}

#[derive(Clone)]
pub struct True;
impl InternalCommand for True {
    fn name(&self) -> &str {
        "true"
    }

    fn run(&self, _: &[String], _: Arc<Mutex<Context>>, _: Arc<Mutex<InternalIo>>) -> i32 {
        status::SUCCESS
    }
}
