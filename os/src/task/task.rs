//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskInfoInner {
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub start_time: usize,
}
impl TaskInfoInner {
        pub fn new() -> Self {
            TaskInfoInner {
                syscall_times: [0; MAX_SYSCALL_NUM], // 初始化 syscall_times 数组
                start_time: 0, // 初始化时间
            }
        }
    }
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    pub task_info_inner: TaskInfoInner,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}


