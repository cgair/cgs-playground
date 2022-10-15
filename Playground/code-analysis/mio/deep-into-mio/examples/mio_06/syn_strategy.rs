use std::{
    time::{Instant, Duration},
    sync::{
        atomic::{AtomicUsize, Ordering::*},
        Arc, Mutex, Condvar
    },
};

pub struct Poll {
    // Use an atomic to first check if a full lock will be required. This is a
    // fast-path check for single threaded cases avoiding the extra syscall
    lock_state: AtomicUsize,

    // Sequences concurrent calls to `Poll::poll`
    lock: Mutex<()>,

    // Wakeup the next waiter
    condvar: Condvar,
}

impl Poll {
    pub fn new() -> Poll {
        Poll {
            lock_state: AtomicUsize::new(0),
            lock: Mutex::new(()),
            condvar: Condvar::new(),
        }
    }

    fn poll1(&self, mut timeout: Option<Duration>) -> std::io::Result<usize> {
        let zero = Some(Duration::from_millis(0));

        // At a high level, the synchronization strategy is to acquire access to
        // the critical section by transitioning the atomic from unlocked ->
        // locked. If the attempt fails, the thread will wait on the condition
        // variable.
        //
        // # Some more detail
        //
        // The `lock_state` atomic usize combines:
        //
        // - locked flag, stored in the least significant bit
        // - number of waiting threads, stored in the rest of the bits.
        //
        // When a thread transitions the locked flag from 0 -> 1, it has
        // obtained access to the critical section.
        //
        // When entering `poll`, a compare-and-swap from 0 -> 1 is attempted.
        // This is a fast path for the case when there are no concurrent calls
        // to poll, which is very common.
        //
        // On failure, the mutex is locked, and the thread attempts to increment
        // the number of waiting threads component of `lock_state`. If this is
        // successfully done while the locked flag is set, then the thread can
        // wait on the condition variable.
        //
        // When a thread exits the critical section, it unsets the locked flag.
        // If there are any waiters, which is atomically determined while
        // unsetting the locked flag, then the condvar is notified.

        let mut curr = self.lock_state.compare_and_swap(0, 1, SeqCst);

        if 0 != curr {
            // Enter slower path
            let mut lock = self.lock.lock().unwrap();
            let mut inc = false;

            loop {
                if curr & 1 == 0 {
                    // The lock is currently free, attempt to grab it
                    let mut next = curr | 1;

                    if inc {
                        // The waiter count has previously been incremented, so
                        // decrement it here
                        next -= 2;
                    }

                    let actual = self.lock_state.compare_and_swap(curr, next, SeqCst);

                    if actual != curr {
                        curr = actual;
                        continue;
                    }

                    // Lock acquired, break from the loop
                    break;
                }

                if timeout == zero {
                    if inc {
                        self.lock_state.fetch_sub(2, SeqCst);
                    }

                    return Ok(0);
                }

                // The lock is currently held, so wait for it to become
                // free. If the waiter count hasn't been incremented yet, do
                // so now
                if !inc {
                    let next = curr.checked_add(2).expect("overflow");
                    let actual = self.lock_state.compare_and_swap(curr, next, SeqCst);

                    if actual != curr {
                        curr = actual;
                        continue;
                    }

                    // Track that the waiter count has been incremented for
                    // this thread and fall through to the condvar waiting
                    inc = true;
                }

                lock = match timeout {
                    Some(to) => {
                        let now = Instant::now();

                        // Wait to be notified
                        let (l, _) = self.condvar.wait_timeout(lock, to).unwrap();

                        // See how much time was elapsed in the wait
                        let elapsed = now.elapsed();

                        // Update `timeout` to reflect how much time is left to
                        // wait.
                        if elapsed >= to {
                            timeout = zero;
                        } else {
                            // Update the timeout
                            timeout = Some(to - elapsed);
                        }

                        l
                    }
                    None => {
                        self.condvar.wait(lock).unwrap()
                    }
                };

                // Reload the state
                curr = self.lock_state.load(SeqCst);

                // Try to lock again...
            }
        }

        /* access to the critical section */
        // let ret = self.poll2(events, timeout, interruptible);

        // Release the lock
        if 1 != self.lock_state.fetch_and(!1, Release) {
            // Acquire the mutex
            let _lock = self.lock.lock().unwrap();

            // There is at least one waiting thread, so notify one
            self.condvar.notify_one();
        }

        return Ok(0);
    }

}

fn main() {

}