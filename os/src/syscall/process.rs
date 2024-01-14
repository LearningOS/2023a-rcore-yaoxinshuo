//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{exit_current_and_run_next, suspend_current_and_run_next,get_current_task_info, TaskStatus},
    timer::get_time_us,
};

//hehe
#[repr(C)]
#[derive(Debug)]
//hehe
pub struct TimeVal {
    //hehe
    pub sec: usize,
    //hehe
    pub usec: usize,
    //hehe
}

/// Task information
#[allow(dead_code)]
#[derive(Copy, Clone)]
//hehe
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [i32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}
impl TaskInfo {
    // 初始化函数
    pub fn new() -> Self {
        TaskInfo {
            status: TaskStatus::UnInit, // 设置初始状态
            syscall_times: [0; MAX_SYSCALL_NUM], // 初始化 syscall_times 数组
            time: 0, // 初始化时间
        }
    }
}
/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let task_info = get_current_task_info();
    unsafe {
        *ti = TaskInfo {
            status: task_info.status,
            syscall_times: task_info.syscall_times,
            time: get_time_us() - task_info.time,
        };
    }
    1
}
