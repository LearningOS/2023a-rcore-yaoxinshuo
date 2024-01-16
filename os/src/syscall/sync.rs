use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut inner = process.inner_exclusive_access();
    if let Some(id) = inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        inner.mutex_list[id] = mutex;
        inner.mutex_allo[id] = None;
        id as isize
    } else {
        inner.mutex_list.push(mutex);
        inner.mutex_allo.push(None);
        inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    let process = current_process();
    let inner = process.inner_exclusive_access();
    let mutex = Arc::clone(inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(inner);
    if process.has_been_deadlock_m(tid, mutex_id) {
        drop(process);
        -0xDEAD
    } 
    else {
        let mut inner = process.inner_exclusive_access();
        inner.mutex_allo[mutex_id] = Some(tid);
        inner.mutex_need[tid] = None;
        mutex.lock();
        0
    }
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let inner = process.inner_exclusive_access();
    let mutex = Arc::clone(inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(inner);
    drop(process);
    mutex.unlock();
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    inner.mutex_allo[mutex_id] = None;
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    let id = if let Some(id) = inner
        .sem_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        inner.sem_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        inner.sem_avai[id] = res_count;
        let len = inner.thread_count();
        for i in 0..len {
            inner.sem_allo[i][id] = 0;
            inner.sem_need[i][id] = 0;
        }
        id
    } else {
        inner
            .sem_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        inner.sem_avai.push(res_count);
        let len = inner.thread_count();
        for i in 0..len {
            inner.sem_allo[i].push(0);
            inner.sem_need[i].push(0);
        }
        inner.sem_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    let process = current_process();
    let inner = process.inner_exclusive_access();
    let sem = Arc::clone(inner.sem_list[sem_id].as_ref().unwrap());
    drop(inner);
    sem.up();
    let mut inner = process.inner_exclusive_access();
    inner.sem_allo[tid][sem_id] -= 1;
    inner.sem_avai[sem_id] += 1;
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    let process = current_process();
    let inner = process.inner_exclusive_access();
    let sem = Arc::clone(inner.sem_list[sem_id].as_ref().unwrap());
    drop(inner);
    if process.has_been_deadlock_s(tid, sem_id) {
        -0xDEAD
    } 
    else {
        let mut inner = process.inner_exclusive_access();
        inner.sem_avai[sem_id] -= 1;
        inner.sem_allo[tid][sem_id] += 1;
        inner.sem_need[tid][sem_id] -= 1;
        sem.down();
        0
    }
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    let id = if let Some(id) = inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let inner = process.inner_exclusive_access();
    let condvar = Arc::clone(inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let inner = process.inner_exclusive_access();
    let condvar = Arc::clone(inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    match enabled {
        0 => {inner.deadlock_detect = false; 0},
        1 => {inner.deadlock_detect = true; 0},
        _ => -1
    }
}
