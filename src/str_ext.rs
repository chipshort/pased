pub trait StrExt {
    fn subslice_offset(&self, inner: &str) -> Option<usize>;
}

impl StrExt for &str {
    fn subslice_offset(&self, inner: &str) -> Option<usize> {
        let self_beg = self.as_ptr();
        let inner = inner.as_ptr();
        if inner < self_beg || inner > self_beg.wrapping_add(self.len()) {
            None
        } else {
            // SAFETY: we just checked that `inner` is inside `self`
            Some(unsafe { inner.offset_from(self_beg) } as usize)
        }
    }
}
