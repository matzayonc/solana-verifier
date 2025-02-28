use std::collections::VecDeque;

use bytemuck::{Pod, Zeroable};
use swiftness::types::StarkProof;

use crate::{Cache, intermediate::Intermediate, task::Tasks};

/// A stack-like structure to store the tasks to be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Schedule<T, const N: usize>
where
    T: Pod + Zeroable + Default,
{
    data: [T; N],
    top: usize,
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
        }
    }
}

impl<T, const N: usize> Schedule<T, N>
where
    T: Pod + Zeroable + Default + From<Tasks>,
{
    pub fn generate_tasks(
        &mut self,
        proof: &mut StarkProof,
        cache: &mut Cache,
        intermediate: &mut Intermediate,
    ) {
        let mut queue = VecDeque::new();
        queue.push_back(Tasks::VerifyProofWithoutStark);

        while let Some(task) = queue.pop_front() {
            // Add the current task to schedule
            self.push(task.into());
            let task_view = task.view(proof, cache, intermediate);
            let children = task_view.children();

            // Add the children in a stack-like manner
            for child in children.into_iter().rev() {
                queue.push_front(child);
            }
        }
    }

    pub fn finished(&self) -> bool {
        self.top == 0
    }

    pub fn next(&mut self) -> Option<T> {
        if self.top == 0 {
            None
        } else {
            self.top -= 1;
            Some(self.data[self.top])
        }
    }

    pub fn push(&mut self, value: T) {
        self.data[self.top] = value;
        self.top += 1;
    }

    /// Tasks should be in the reverse order of execution.
    pub fn push_slice(&mut self, vec: &[T]) {
        self.data[self.top..self.top + vec.len()].copy_from_slice(vec);
        self.top += vec.len();
    }

    pub fn flush(&mut self) {
        self.top = 0;
    }

    pub fn from_slice(vec: &[T]) -> Self {
        let mut stack = Self::default();
        stack.data[..vec.len()].copy_from_slice(vec);
        stack.top = vec.len();
        stack
    }

    pub fn remaining(&self) -> usize {
        self.top
    }
}
