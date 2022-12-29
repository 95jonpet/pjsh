use mockall::mock;

use crate::shell::{MockShell, ShellInput};
use pjsh_core::Host;

use super::*;

mock! {
    TestHost {}
    impl Host for TestHost {
        fn add_child_process(&mut self, child: std::process::Child);
        fn add_thread(&mut self, thread: std::thread::JoinHandle<i32>);
        fn kill_all_processes(&mut self);
        fn join_all_threads(&mut self);
        fn take_exited_child_processes(&mut self) -> std::collections::HashSet<u32>;
    }
}

#[test]
fn shell_interrupt() {
    let mut context = Context::default();
    let executor: Box<dyn Execute> = Box::new(ProgramExecutor::new(true));
    let mut mock_host = MockTestHost::new();
    mock_host
        .expect_take_exited_child_processes()
        .returning(std::collections::HashSet::new);

    mock_host
        .expect_kill_all_processes()
        .times(1)
        .return_const(());
    mock_host
        .expect_join_all_threads()
        .times(1)
        .return_const(());

    context.host = Arc::new(parking_lot::Mutex::new(mock_host));
    let ctx = Arc::new(Mutex::new(context));

    let mut counter = 0;
    let mut mock_shell = MockShell::new();
    mock_shell.expect_is_interactive().returning(|| true);
    mock_shell.expect_save_history().times(1).return_const(());
    mock_shell
        .expect_prompt_line()
        .times(2)
        .returning(move |_| {
            counter += 1;
            if counter != 1 {
                ShellInput::Logout
            } else {
                ShellInput::Interrupt
            }
        });

    run_shell(Box::new(mock_shell), executor.as_ref(), ctx);
}
