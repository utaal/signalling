use std::cell::UnsafeCell;
use std::sync::{Arc, atomic::AtomicBool};
use std::sync::atomic::Ordering;

const TOKEN_PRESENT: bool = true;
const TOKEN_ABSENT: bool = false;

pub fn signal() -> (Signaller, PackedSignalled) {
    let signal = Arc::new(Signal {
        token: AtomicBool::new(TOKEN_ABSENT),
        thread: UnsafeCell::new(None),
    });

    (Signaller {
        signal: signal.clone(),
    },
    PackedSignalled {
        signal
    })
}

struct Signal {
    token: AtomicBool,
    thread: UnsafeCell<Option<::std::thread::Thread>>,
}


#[derive(Clone)]
pub struct Signaller {
    signal: Arc<Signal>,
}

unsafe impl Send for Signaller { }

impl Signaller {
    pub fn ping(&self) {
        if let &Some(ref thread) = unsafe { &*self.signal.thread.get() } {
            thread.unpark();
        } else {
            self.signal.token.store(TOKEN_PRESENT, Ordering::Release);
        }
    }
}

pub struct PackedSignalled {
    signal: Arc<Signal>,
}

impl PackedSignalled {
    pub fn this_thread(self) -> Signalled {
        let PackedSignalled { signal } = self;
        unsafe {
            *signal.thread.get() = Some(std::thread::current());
        }
        Signalled {
            signal
        }
    }
}

unsafe impl Send for PackedSignalled { }

pub struct Signalled {
    signal: Arc<Signal>,
}

impl Signalled {
    pub fn wait(&self) {
        if self.signal.token.swap(TOKEN_ABSENT, Ordering::Acquire) == TOKEN_ABSENT {
            std::thread::park();
        }
    }
}
