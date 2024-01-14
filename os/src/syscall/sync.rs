use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;

static mut ASAS: [usize; 1000] = [0;1000];
///asas
pub fn modify_asas(x:usize, v:usize){
    unsafe{
        ASAS[x] = v;
    }
}

static mut AVALIABLE: [i32; 1000] = [0; 1000];
static mut NEED: [[i32; 1000]; 1000] = [[0; 1000]; 1000];
static mut ALLOCATION: [[i32; 1000]; 1000] = [[0; 1000]; 1000];

static mut AVALIABLE2: [i32; 1000] = [0; 1000];
static mut NEED2: [[i32; 1000]; 1000] = [[0; 1000]; 1000];
static mut ALLOCATION2: [[i32; 1000]; 1000] = [[0; 1000]; 1000];

///cnm
pub fn has_been_deadlock() -> i32 { 
    unsafe{
        let process = current_process();
        if ASAS[process.pid.0] == 0{
            return 0;
        }
        let mut work = AVALIABLE.clone(); 
        let mut finish = [0; 1000]; 
        let n = 999;
        for i in 0..n { 
            let mut haveneed = 0;
            for j in 0..n {
                haveneed += NEED[i][j];
            }
            if haveneed == 0{
                finish[i] = 1;
                for j in 0..n {
                    work[j] += ALLOCATION[i][j];
                }
            }
        } 
        let mut notfinish = 1;
        let mut newfinish = 0;
        while notfinish == 1{
            notfinish = 0;
            if newfinish == 0{
                return 1
            }
            newfinish = 0;
            for i in 0..n {
                if finish[i] == 0{
                    notfinish = 1;
                    for j in 0..n{
                        if NEED[i][j] == 1{
                            if work[j] > 0{
                                finish[i] = 1;
                                for k in 0..n {
                                    work[k] += ALLOCATION[i][k];
                                }
                            }
                            break;
                        }
                    }
                }
            }
        } 
        
        work = AVALIABLE2.clone(); 
        finish = [0; 1000]; 
        for i in 0..n { 
            let mut haveneed = 0;
            for j in 0..n {
                haveneed += NEED2[i][j];
            }
            if haveneed == 0{
                finish[i] = 1;
                for j in 0..n {
                    work[j] += ALLOCATION2[i][j];
                }
            }
        } 
        let mut notfinish = 1;
        let mut newfinish = 0;
        while notfinish == 1{
            notfinish = 0;
            if newfinish == 0{
                return 1;
            }
            newfinish = 0;
            for i in 0..n {
                if finish[i] == 0{
                    notfinish = 1;
                    for j in 0..n{
                        if NEED2[i][j] == 1{
                            if work[j] > 0{
                                finish[i] = 1;
                                for k in 0..n {
                                    work[k] += ALLOCATION2[i][k];
                                }
                            }
                            break;
                        }
                    }
                }
            }
        } 
    }
    return 0;
}

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
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        unsafe{
            AVALIABLE[id] = 1;
        }
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        unsafe{
            AVALIABLE[process_inner.mutex_list.len()] = 1;
            process_inner.mutex_list.len() as isize - 1
        }
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
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);

    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;

    unsafe{
        NEED[tid][mutex_id]=1;
    }
    mutex.lock();
    if has_been_deadlock()==1{   
        mutex.unlock();
        return 0xDEAD;
    }
    else{
        unsafe{
            AVALIABLE[mutex_id]-=1;
            NEED[tid][mutex_id]=0;
            ALLOCATION[tid][mutex_id]+=1;
        }
    }
    0
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
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mutex.unlock();
    unsafe{
        AVALIABLE[mutex_id]+=1;
        ALLOCATION[tid][mutex_id]-=1;
    }
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
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    unsafe{
        AVALIABLE2[id] = res_count as i32;
    }
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
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    drop(process_inner);
    sem.up();
    unsafe{
        AVALIABLE2[sem_id]+=1;
        ALLOCATION2[tid][sem_id]-=1;
    }
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
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;

    unsafe{
        NEED2[tid][sem_id]=1;
    }
    sem.down();
    if has_been_deadlock()==1{    
        sem.up();
        return 0xDEAD;
    }
    else{
        unsafe{
            AVALIABLE2[sem_id]-=1;
            NEED2[tid][sem_id]=0;
            ALLOCATION2[tid][sem_id]+=1;
    
        }
    }
    0
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
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
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
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
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
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(is_enable: usize) -> isize{
    if is_enable != 0 && is_enable != 1{
        return -1
    }
    else if has_been_deadlock() == 1 {
        return -1
    }
    let process = current_process().pid.0;
    modify_asas(process,is_enable);
    return 0;
}