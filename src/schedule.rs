use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Schedule<T, const N: usize>
where
    T: Pod + Zeroable + Default,
{
    data: [T; N],
    top: usize,
    finished: usize,
}

unsafe impl<T: Pod + Zeroable + Default, const N: usize> Pod for Schedule<T, N> {}
unsafe impl<T: Pod + Zeroable + Default, const N: usize> Zeroable for Schedule<T, N> {}

impl<T, const N: usize> Default for Schedule<T, N>
where
    T: Pod + Zeroable + Default,
{
    fn default() -> Self {
        Self {
            data: [T::default(); N],
            top: 0,
            finished: 0,
        }
    }
}

impl<T, const N: usize> Schedule<T, N>
where
    T: Pod + Zeroable + Default,
{
    pub fn finished(&self) -> bool {
        self.top == self.finished
    }

    pub fn next(&mut self) -> Option<&T> {
        if self.finished >= self.top {
            None
        } else {
            let value = &self.data[self.finished];
            self.finished += 1;
            Some(value)
        }
    }

    pub fn next_owned(&mut self) -> Option<T> {
        self.next().cloned()
    }

    pub fn push(&mut self, value: T) {
        self.data[self.top] = value;
        self.top += 1;
    }

    pub fn push_slice(&mut self, vec: &[T]) {
        self.data[self.top..self.top + vec.len()].copy_from_slice(vec);
        self.top += vec.len();
    }

    pub fn flush(&mut self) {
        self.finished = 0;
        self.top = 0;
    }

    pub fn from_slice(vec: &[T]) -> Self {
        let mut stack = Self::default();
        stack.data[..vec.len()].copy_from_slice(vec);
        stack.top = vec.len();
        stack.finished = 0;
        stack
    }
}
