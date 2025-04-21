use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

/**
A mutable pointer that we promise to use safely.
# Safety
- Modifying the pointer in any way could cause undefined behavior unless you're sure there cannot be race conditions.
- If only one thread is acting on the pointer you can interact with the pointer in any way you want without worry, but then you may as well just use a pointer instead of this struct.
*/
#[derive(Debug, Copy, Clone)]
pub struct ThreadSafeMutPtr<T> {
    ptr: *mut T,
}

impl<T> ThreadSafeMutPtr<T> {
    /**
    Returns a new `ThreadSafeMutPtr`.
    # Safety
    - Calling this function is perfectly safe,
      but using the `ThreadSafeMutPtr` it returns is not.
    - You MUST know that the way you use the pointer CANNOT cause race conditions.
    */
    pub const unsafe fn new(ptr: *mut T) -> Self {
        Self { ptr }
    }

    /**
    Returns a new `ThreadSafeMutPtr`
    # Safety
    Has the same safety implications as `ThreadSafeMutPtr::new`
    */
    pub const unsafe fn from_mut_ref(r: &mut T) -> Self {
        unsafe { Self::new(std::ptr::from_mut(r)) }
    }

    /**
    Casts the inner pointer as a mutable reference

    # Panics
    If the inner pointer is null this panics

    # Safety

    */
    #[must_use]
    pub const unsafe fn into_mut_ref<'a>(self) -> Option<&'a mut T> {
        unsafe { self.into_inner().as_mut() }
    }

    /**
    Casts the inner pointer as a mutable reference
    # Safety
    The inner pointer must be known to be non-null
    */
    #[must_use]
    pub const unsafe fn into_mut_ref_unchecked<'a>(self) -> &'a mut T {
        unsafe { &mut *self.into_inner() }
    }

    /// Returns the inner pointer
    #[must_use]
    pub const fn into_inner(self) -> *mut T {
        self.ptr
    }

    /**
    Shorthand for `self.ptr.add(amount)`
    # Safety
    Probably super unsafe but we don't care :3
    */
    #[must_use]
    pub const unsafe fn add(self, amount: usize) -> Self {
        unsafe { Self::new(self.ptr.add(amount)) }
    }

    /// Dereferences the pointer, replacing the value at that address with `new_value`;
    pub const fn write(&mut self, new_value: T) {
        unsafe { self.ptr.write(new_value) };
    }
}

unsafe impl<T> Send for ThreadSafeMutPtr<T> {}
unsafe impl<T> Sync for ThreadSafeMutPtr<T> {}

/**
A data-race safe `Vec<bool>` where elements default to true and once set to false remain false forever.
# Notes On Safety
For simplicity the internal `Vec`'s length should not grow
*/
#[derive(Debug, Default, Clone)]
pub struct SieveVecBool {
    vec: Vec<bool>,
}

impl From<Vec<bool>> for SieveVecBool {
    fn from(vec: Vec<bool>) -> Self {
        Self { vec }
    }
}

impl SieveVecBool {
    /// Returns an empty `SieveVecBool`
    #[must_use]
    pub const fn new() -> Self {
        Self { vec: Vec::new() }
    }

    /// Returns the inner `Vec`
    #[must_use]
    pub fn into_inner(self) -> Vec<bool> {
        self.vec
    }

    /**
    Returns a `ThreadSafeMutPtr` pointing to the start of the inner `Vec`.
    # Safety
    Has the same safety implications as `ThreadSafeMutPtr::new`
    */
    unsafe fn ptr_to_start_of_vec(&mut self) -> ThreadSafeMutPtr<bool> {
        unsafe { ThreadSafeMutPtr::new(self.vec.as_mut_ptr()) }
    }

    /**
    Sets the element at `index` to false in the `Vec`.
    # Safety
    `index` must be known to be a valid index into the inner `Vec`
    */
    pub unsafe fn set_false_unchecked(&mut self, index: usize) {
        *unsafe { self.vec.get_unchecked_mut(index) } = false;
    }

    /**
    Calls `set_false` on all the indices in the range given its `start`, `stop`, and `step`.
    This differs from `set_step_range_to_false` by performing its operations in parallel, which could be faster depending on your use case.

    # Safety
    all elements in `(start..stop).step_by(step_size)` must be valid indices into the `Vec`.
    */
    pub unsafe fn set_step_range_to_false_par(
        &mut self,
        start: usize,
        stop: usize,
        step_size: usize,
    ) {
        let ptr: ThreadSafeMutPtr<bool> = unsafe { self.ptr_to_start_of_vec() };

        let range = (start..stop).step_by(step_size);
        range.par_bridge().for_each(move |index| unsafe {
            let mut p = ptr.add(index);
            p.write(false);
        });
    }

    /**
    Calls `set_false` on all the indices in the range given its `start`, `stop`, and `step`.

    # Safety
    all elements in `(start..stop).step_by(step_size)` must be valid indices into the `Vec`.
    */
    pub unsafe fn set_step_range_to_false(&mut self, start: usize, stop: usize, step_size: usize) {
        let mut index = start;
        while index < stop {
            unsafe { self.set_false_unchecked(index) };
            index += step_size;
        }
    }

    /**
    Produces the multiples of `n`, setting all those indices in the inner Vec to false.
    # Safety
    Has the same safety implications as `set_step_range_to_false`
    */
    pub unsafe fn set_multiples_to_false(&mut self, n: usize) {
        unsafe { self.set_step_range_to_false(n, self.vec.len(), n) };
    }

    /**
    Produces the multiples of `n`, setting all those indices in the inner Vec to false.
    Differs from `set_multiples_to_false` by being parallel, which could be faster depending on your use case.

    # Safety
    Has the same safety implications as `set_step_range_to_false`
    */
    pub unsafe fn set_multiples_to_false_par(&mut self, n: usize) {
        unsafe { self.set_step_range_to_false_par(n, self.vec.len(), n) };
    }

    /**
    Calls `self.set_multiples_to_false` for all the items in `iter`.
    # Panics
    Panics if a null pointer is dereferenced

    # Safety
    */
    pub unsafe fn set_multiples_of_slice_to_false_par(&mut self, slice: &[usize]) {
        let self_ptr: ThreadSafeMutPtr<Self> = unsafe { ThreadSafeMutPtr::from_mut_ref(self) };
        slice.into_par_iter().for_each(move |&index| {
            unsafe {
                self_ptr
                    .clone()
                    .into_mut_ref_unchecked()
                    .set_multiples_to_false(index);
            };
        });
    }
}
