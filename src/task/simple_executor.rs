use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use alloc::collections::VecDeque;

use super::Task;

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = stub_waker();
            let mut cx = Context::from_waker(&waker);
            match task.poll(&mut cx) {
                Poll::Ready(()) => {}
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}

impl Default for SimpleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

fn stub_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        stub_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(core::ptr::null::<()>(), vtable)
}

fn stub_waker() -> Waker {
    unsafe { Waker::from_raw(stub_raw_waker()) }
}
