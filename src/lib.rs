use rayon::iter::{ParallelBridge, ParallelIterator};

/**
A mutable pointer to a bool that we promise to use safely.
# Safety
- Modifying the pointer in any way could cause undefined behavior unless you're sure there cannot be race conditions.
- If only one thread is acting on the pointer you can interact with the pointer in any way you want without worry, but then you may as well just use a pointer instead of this struct.
*/
#[derive(Debug, Copy, Clone)]
pub struct ThreadSafeBoolPtr {
    ptr: *mut bool,
}

impl ThreadSafeBoolPtr {
    /**
    Returns a new `ThreadSafeBoolPtr`.
    # Safety
    - Calling this function is perfectly safe,
      but using the `ThreadSafeBoolPtr` it returns is not.
    - You MUST know that the way you use the pointer CANNOT cause race conditions.
    */
    #[allow(unsafe_code)]
    pub const unsafe fn new(ptr: *mut bool) -> Self {
        Self { ptr }
    }
    /// Returns the inner pointer
    #[must_use]
    pub const fn into_inner(self) -> *mut bool {
        self.ptr
    }

    /// Shorthand for `self.ptr.add(amount)`
    /// # Safety
    /// Probably super unsafe but we don't care :3
    #[allow(unsafe_code)]
    #[must_use]
    pub const unsafe fn add(self, amount: usize) -> Self {
        unsafe { Self::new(self.ptr.add(amount)) }
    }

    /// Dereferences the pointer, replacing the bool at that memory address with `false`.
    #[allow(unsafe_code)]
    pub const fn write_false(&mut self) {
        unsafe { self.ptr.write(false) };
    }
}
#[allow(unsafe_code)]
unsafe impl Send for ThreadSafeBoolPtr {}
#[allow(unsafe_code)]
unsafe impl Sync for ThreadSafeBoolPtr {}

/// A data-race safe Vec<bool> where elements default to true and once set to false remain false forever.
/// # Notes On Safety
/// For simplicity the internal Vec's length should not grow
struct SieveVecBool {
    vec: Vec<bool>,
}

impl From<Vec<bool>> for SieveVecBool {
    fn from(vec: Vec<bool>) -> Self {
        Self { vec }
    }
}

impl SieveVecBool {
    /// Returns an empty `SieveVecBool`
    pub const fn new() -> Self {
        Self { vec: Vec::new() }
    }

    /// Returns the inner `Vec`
    pub fn into_inner(self) -> Vec<bool> {
        self.vec
    }

    /**
    Sets the element at `index` to false in the `Vec`.
    # Safety
    `index` must be known to be a valid index into the inner `Vec`
    */
    #[allow(unsafe_code)]
    pub unsafe fn set_false_unchecked(&mut self, index: usize) {
        *unsafe { self.vec.get_unchecked_mut(index) } = false;
    }

    /**
    Calls `set_false` on all the indices in the range given its `start`, `stop`, and `step`
    # Safety
    all elements in `(start..stop).step_by(step_size)` must be valid indices into the `Vec`.
    */
    #[allow(unsafe_code)]
    pub unsafe fn set_step_range_to_false(&mut self, start: usize, stop: usize, step_size: usize) {
        let ptr: ThreadSafeBoolPtr = unsafe { ThreadSafeBoolPtr::new(self.vec.as_mut_ptr()) };

        let range = (start..stop).step_by(step_size);
        range.par_bridge().for_each(move |index| unsafe {
            let mut p = ptr.add(index);
            p.write_false();
        });
    }
}
