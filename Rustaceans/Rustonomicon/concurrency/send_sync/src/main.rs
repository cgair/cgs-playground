/*
Some types allow you to have multiple aliases of a location in memory while mutating it.
Unless these types use synchronization to manage this access, they are absolutely not thread-safe. 

* Rust captures this through the Send and Sync traits:
    A type is Send if it is safe to send it to another thread.
    A type is Sync if it is safe to share between threads (T is Sync if and only if &T is Send).
*/
fn main() {}