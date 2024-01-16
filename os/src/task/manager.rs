//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use crate::config::BIG_STRIDE;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        trace!("fetch");
    
        let (max_index, max_task) = {
            let max_by_stride = |(_, a): &(usize, &Arc<TaskControlBlock>), (_, b): &(usize, &Arc<TaskControlBlock>)| {
                let stride_a = a.inner_exclusive_access().stride;
                let stride_b = b.inner_exclusive_access().stride;
                stride_b.cmp(&stride_a)
            };
    
            self.ready_queue
                .iter()
                .enumerate()
                .max_by(max_by_stride)?
        };
    
        trace!("{:#?}", max_index);
    
        {
            let mut inner = max_task.inner_exclusive_access();
            inner.stride += BIG_STRIDE / inner.priority as u64;
        }
    
        self.ready_queue.remove(max_index)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
