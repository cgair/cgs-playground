# Concurrency: An Introduction
A classic process(which we can now call a single-threaded process), there is a single stack, usually residing at the bottom of the address space.
However, in a multi-threaded process, each thread runs independently and of course may call into various routines to do whatever work it is doing. 

Instead of a single stack in the address space, there will be one per thread:
+-------------------+ 0 KB
|   Program Code    |
|-------------------| 1 KB
|       Heap        |
|-------------------| 2 KB
|                   |
|       (free)      |
|                   |
+-------------------+
|      Stack(2)     |
|-------------------|
|      (free)       |
|-------------------| 15KB
|      Stack(1)     |
+-------------------+ 16KB

# Locks
## HOW TO AVOID SPINNING?
A Simple Approach: Just Yield(give up the CPU and let another thread run). Consider the case where there are many threads (say 100) contending for a lock repeatedly.
* In this case, if one thread acquires the lock and is preempted before releasing it, the other 99 will each call lock(), find the lock held, and yield the CPU. While better than our spinning approach (which would waste 99 time slices spinning), this approach is still costly; the cost of a context switch can be substantial, and there is thus plenty of waste.
* **Starvation problem:** a thread may get caught in an endless yield loop while other threads repeatedly enter and exit the critical section.
