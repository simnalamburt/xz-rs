#[cfg(any(unix, windows))]
use crate::types::c_int;
#[cfg(windows)]
use crate::types::c_uint;
use crate::types::c_void;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
#[cfg(windows)]
use windows_sys::Win32::System::SystemInformation::GetTickCount;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    CONDITION_VARIABLE, CRITICAL_SECTION, DeleteCriticalSection, EnterCriticalSection, INFINITE,
    InitializeConditionVariable, InitializeCriticalSection, LeaveCriticalSection,
    SleepConditionVariableCS, WaitForSingleObject, WakeConditionVariable,
};

#[cfg(windows)]
unsafe extern "C" {
    fn _beginthreadex(
        security: *mut c_void,
        stack_size: c_uint,
        start_address: Option<unsafe extern "system" fn(*mut c_void) -> u32>,
        arglist: *mut c_void,
        initflag: c_uint,
        thrdaddr: *mut c_uint,
    ) -> usize;
}

pub const MYTHREAD_RET_VALUE: *mut c_void = core::ptr::null_mut();

#[cfg(unix)]
pub type mythread = libc::pthread_t;
#[cfg(windows)]
pub type mythread = HANDLE;
#[cfg(unix)]
pub type mythread_mutex = libc::pthread_mutex_t;
#[cfg(windows)]
pub type mythread_mutex = CRITICAL_SECTION;

#[cfg(unix)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mythread_cond {
    pub cond: libc::pthread_cond_t,
    // Clock ID (CLOCK_REALTIME or CLOCK_MONOTONIC) associated with
    // the condition variable.
    pub clk_id: libc::clockid_t,
}

#[cfg(windows)]
pub type mythread_cond = CONDITION_VARIABLE;
#[cfg(unix)]
pub type mythread_condtime = libc::timespec;

#[cfg(windows)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mythread_condtime {
    pub start: u32,
    pub timeout: u32,
}

#[cfg(all(unix, not(target_os = "emscripten")))]
#[inline]
pub unsafe fn mythread_sigmask(how: c_int, set: *const libc::sigset_t, oset: *mut libc::sigset_t) {
    let _ret: c_int = unsafe { libc::pthread_sigmask(how, set, oset) };
}

// Emscripten's libc has no pthread_sigmask(); signals barely exist there.
#[cfg(all(unix, target_os = "emscripten"))]
#[inline]
pub unsafe fn mythread_sigmask(
    _how: c_int,
    _set: *const libc::sigset_t,
    _oset: *mut libc::sigset_t,
) {
}

#[cfg(windows)]
struct mythread_start_info {
    func: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
}

#[cfg(windows)]
unsafe extern "system" fn mythread_start(param: *mut c_void) -> u32 {
    let info = Box::from_raw(param.cast::<mythread_start_info>());
    let _ = (info.func)(info.arg);
    0
}

// Creates a new thread with all signals blocked.
#[cfg(unix)]
#[inline]
pub unsafe fn mythread_create(
    thread: *mut mythread,
    func: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    use core::mem::MaybeUninit;

    unsafe {
        let mut old = MaybeUninit::<libc::sigset_t>::uninit();
        let mut all = MaybeUninit::<libc::sigset_t>::uninit();
        libc::sigfillset(all.as_mut_ptr());

        mythread_sigmask(libc::SIG_SETMASK, all.as_ptr(), old.as_mut_ptr());
        let ret: c_int = libc::pthread_create(
            thread,
            core::ptr::null(),
            core::mem::transmute::<
                unsafe extern "C" fn(*mut c_void) -> *mut c_void,
                extern "C" fn(*mut c_void) -> *mut c_void,
            >(func),
            arg,
        );
        mythread_sigmask(libc::SIG_SETMASK, old.as_ptr(), core::ptr::null_mut());

        ret
    }
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_create(
    thread: *mut mythread,
    func: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    let info = Box::into_raw(Box::new(mythread_start_info { func, arg }));
    let ret = unsafe {
        _beginthreadex(
            core::ptr::null_mut(),
            0,
            Some(mythread_start),
            info.cast::<c_void>(),
            0,
            core::ptr::null_mut(),
        )
    };
    if ret == 0 {
        unsafe {
            let _ = Box::from_raw(info);
        }
        -1
    } else {
        unsafe {
            *thread = ret as HANDLE;
        }
        0
    }
}

#[cfg(unix)]
#[inline]
pub fn mythread_join(thread: mythread) -> c_int {
    unsafe { libc::pthread_join(thread, core::ptr::null_mut()) }
}

#[cfg(windows)]
#[inline]
pub fn mythread_join(thread: mythread) -> c_int {
    let mut ret = 0;
    unsafe {
        if WaitForSingleObject(thread, INFINITE) != WAIT_OBJECT_0 {
            ret = -1;
        }
        if CloseHandle(thread) == 0 {
            ret = -1;
        }
    }
    ret
}

#[cfg(unix)]
#[inline]
pub unsafe fn mythread_mutex_init(mutex: *mut mythread_mutex) -> c_int {
    unsafe { libc::pthread_mutex_init(mutex, core::ptr::null()) }
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_mutex_init(mutex: *mut mythread_mutex) -> c_int {
    unsafe {
        InitializeCriticalSection(mutex);
    }
    0
}

#[cfg(unix)]
#[inline]
pub unsafe fn mythread_mutex_destroy(mutex: *mut mythread_mutex) {
    let _ret: c_int = unsafe { libc::pthread_mutex_destroy(mutex) };
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_mutex_destroy(mutex: *mut mythread_mutex) {
    unsafe {
        DeleteCriticalSection(mutex);
    }
}

#[cfg(unix)]
#[inline]
pub unsafe fn mythread_mutex_lock(mutex: *mut mythread_mutex) {
    let _ret: c_int = unsafe { libc::pthread_mutex_lock(mutex) };
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_mutex_lock(mutex: *mut mythread_mutex) {
    unsafe {
        EnterCriticalSection(mutex);
    }
}

#[cfg(unix)]
#[inline]
pub unsafe fn mythread_mutex_unlock(mutex: *mut mythread_mutex) {
    let _ret: c_int = unsafe { libc::pthread_mutex_unlock(mutex) };
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_mutex_unlock(mutex: *mut mythread_mutex) {
    unsafe {
        LeaveCriticalSection(mutex);
    }
}

// Initializes a condition variable.
//
// Using CLOCK_MONOTONIC instead of the default CLOCK_REALTIME makes the
// timeout in pthread_cond_timedwait() work correctly also if system time
// is suddenly changed. Unfortunately CLOCK_MONOTONIC isn't available
// everywhere while the default CLOCK_REALTIME is, so the default is
// used if CLOCK_MONOTONIC isn't available.
#[cfg(unix)]
#[inline]
pub unsafe fn mythread_cond_init(mycond: *mut mythread_cond) -> c_int {
    unsafe {
        #[cfg(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        {
            use core::mem::MaybeUninit;

            let mut ts = MaybeUninit::<libc::timespec>::uninit();
            let mut condattr = MaybeUninit::<libc::pthread_condattr_t>::uninit();

            // POSIX doesn't seem to *require* that pthread_condattr_setclock()
            // will fail if given an unsupported clock ID. Test that
            // CLOCK_MONOTONIC really is supported using clock_gettime().
            if libc::clock_gettime(libc::CLOCK_MONOTONIC, ts.as_mut_ptr()) == 0
                && libc::pthread_condattr_init(condattr.as_mut_ptr()) == 0
            {
                let mut ret: c_int =
                    libc::pthread_condattr_setclock(condattr.as_mut_ptr(), libc::CLOCK_MONOTONIC);
                if ret == 0 {
                    ret = libc::pthread_cond_init(
                        ::core::ptr::addr_of_mut!((*mycond).cond),
                        condattr.as_ptr(),
                    );
                }

                libc::pthread_condattr_destroy(condattr.as_mut_ptr());

                if ret == 0 {
                    (*mycond).clk_id = libc::CLOCK_MONOTONIC;
                    return 0;
                }
            }

            // If anything above fails, fall back to the default CLOCK_REALTIME.
        }

        (*mycond).clk_id = libc::CLOCK_REALTIME;
        libc::pthread_cond_init(::core::ptr::addr_of_mut!((*mycond).cond), core::ptr::null())
    }
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_cond_init(cond: *mut mythread_cond) -> c_int {
    unsafe {
        InitializeConditionVariable(cond);
    }
    0
}

#[cfg(unix)]
#[inline]
pub unsafe fn mythread_cond_destroy(cond: *mut mythread_cond) {
    let _ret: c_int =
        unsafe { libc::pthread_cond_destroy(::core::ptr::addr_of_mut!((*cond).cond)) };
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_cond_destroy(_cond: *mut mythread_cond) {}

#[cfg(unix)]
#[inline]
pub unsafe fn mythread_cond_signal(cond: *mut mythread_cond) {
    let _ret: c_int = unsafe { libc::pthread_cond_signal(::core::ptr::addr_of_mut!((*cond).cond)) };
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_cond_signal(cond: *mut mythread_cond) {
    unsafe {
        WakeConditionVariable(cond);
    }
}

#[cfg(unix)]
#[inline]
pub unsafe fn mythread_cond_wait(cond: *mut mythread_cond, mutex: *mut mythread_mutex) {
    let _ret: c_int =
        unsafe { libc::pthread_cond_wait(::core::ptr::addr_of_mut!((*cond).cond), mutex) };
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_cond_wait(cond: *mut mythread_cond, mutex: *mut mythread_mutex) {
    unsafe {
        let _ = SleepConditionVariableCS(cond, mutex, INFINITE);
    }
}

// Waits on a condition or until a timeout expires. If the timeout expires,
// non-zero is returned, otherwise zero is returned.
#[cfg(unix)]
#[inline]
pub unsafe fn mythread_cond_timedwait(
    cond: *mut mythread_cond,
    mutex: *mut mythread_mutex,
    condtime: *const mythread_condtime,
) -> c_int {
    unsafe {
        libc::pthread_cond_timedwait(::core::ptr::addr_of_mut!((*cond).cond), mutex, condtime)
    }
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_cond_timedwait(
    cond: *mut mythread_cond,
    mutex: *mut mythread_mutex,
    condtime: *const mythread_condtime,
) -> c_int {
    let (start, timeout_ms) = unsafe { ((*condtime).start, (*condtime).timeout) };
    let elapsed = unsafe { GetTickCount().wrapping_sub(start) };
    let timeout = if elapsed >= timeout_ms {
        0
    } else {
        timeout_ms - elapsed
    };
    let ret = unsafe { SleepConditionVariableCS(cond, mutex, timeout) };
    i32::from(ret == 0)
}

// Sets condtime to the absolute time that is timeout_ms milliseconds
// in the future. The type of the clock to use is taken from cond.
#[cfg(unix)]
#[inline]
pub unsafe fn mythread_condtime_set(
    condtime: *mut mythread_condtime,
    cond: *const mythread_cond,
    timeout_ms: u32,
) {
    use core::mem::MaybeUninit;

    unsafe {
        (*condtime).tv_sec = (timeout_ms / 1000) as libc::time_t;
        (*condtime).tv_nsec = ((timeout_ms % 1000) * 1_000_000) as _;

        // Zeroed rather than uninit: on failure the timeout stays relative to
        // the epoch and the wait expires immediately, which beats reading
        // uninitialized memory.
        let mut now = MaybeUninit::<libc::timespec>::zeroed();
        let _ret: c_int = libc::clock_gettime((*cond).clk_id, now.as_mut_ptr());
        debug_assert_eq!(_ret, 0, "clock_gettime failed");
        let now = now.assume_init();

        (*condtime).tv_sec += now.tv_sec;
        (*condtime).tv_nsec += now.tv_nsec;

        // tv_nsec must stay in the range [0, 999_999_999].
        if (*condtime).tv_nsec >= 1_000_000_000 {
            (*condtime).tv_nsec -= 1_000_000_000;
            (*condtime).tv_sec += 1;
        }
    }
}

#[cfg(windows)]
#[inline]
pub unsafe fn mythread_condtime_set(
    condtime: *mut mythread_condtime,
    _cond: *const mythread_cond,
    timeout_ms: u32,
) {
    unsafe {
        (*condtime).start = GetTickCount();
        (*condtime).timeout = timeout_ms;
    }
}

// Targets without native threads (wasm32-unknown-unknown, wasm32-wasip1).
// These placeholders keep the multithreaded coders compiling; thread
// creation always fails, so their initialization paths return an error
// instead of ever exercising the no-op lock/wait operations.
#[cfg(not(any(unix, windows)))]
mod unsupported {
    use crate::types::{c_int, c_void};

    pub type mythread = usize;
    pub type mythread_mutex = u8;

    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct mythread_cond {
        _unused: u8,
    }

    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct mythread_condtime {
        _unused: u8,
    }

    #[inline]
    pub fn mythread_create(
        _thread: *mut mythread,
        _func: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
        _arg: *mut c_void,
    ) -> c_int {
        -1
    }

    #[inline]
    pub fn mythread_join(_thread: mythread) -> c_int {
        -1
    }

    #[inline]
    pub fn mythread_mutex_init(_mutex: *mut mythread_mutex) -> c_int {
        0
    }

    #[inline]
    pub fn mythread_mutex_destroy(_mutex: *mut mythread_mutex) {}

    #[inline]
    pub fn mythread_mutex_lock(_mutex: *mut mythread_mutex) {}

    #[inline]
    pub fn mythread_mutex_unlock(_mutex: *mut mythread_mutex) {}

    #[inline]
    pub fn mythread_cond_init(_cond: *mut mythread_cond) -> c_int {
        0
    }

    #[inline]
    pub fn mythread_cond_destroy(_cond: *mut mythread_cond) {}

    #[inline]
    pub fn mythread_cond_signal(_cond: *mut mythread_cond) {}

    #[inline]
    pub fn mythread_cond_wait(_cond: *mut mythread_cond, _mutex: *mut mythread_mutex) {}

    #[inline]
    pub fn mythread_cond_timedwait(
        _cond: *mut mythread_cond,
        _mutex: *mut mythread_mutex,
        _condtime: *const mythread_condtime,
    ) -> c_int {
        1
    }

    #[inline]
    pub fn mythread_condtime_set(
        _condtime: *mut mythread_condtime,
        _cond: *const mythread_cond,
        _timeout_ms: u32,
    ) {
    }
}

#[cfg(not(any(unix, windows)))]
pub use unsupported::*;
