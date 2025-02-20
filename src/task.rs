use solana_program::program_error::ProgramError;
use swiftness_air::layout::recursive::Layout;
pub use swiftness_stark::types::{Cache, Felt, StarkProof};

#[derive(Debug, Clone)]
#[repr(C)]
pub enum Job {
    Job { jobs: Vec<Job> },
    Task { task: Task },
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum Task {
    #[default]
    VerifyProof = 1,
}

struct VerifyProofView<'a> {
    proof: &'a mut StarkProof,
    cache: &'a mut Cache,
}

pub trait TaskTrait {
    fn execute(&mut self);
}

impl<'a> TaskTrait for VerifyProofView<'a> {
    fn execute(&mut self) {
        let security_bits = self.proof.config.security_bits();
        let _res = self
            .proof
            .verify::<Layout>(self.cache, security_bits)
            .unwrap();
    }
}

impl Task {
    pub fn view<'a>(
        &'a mut self,
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
    ) -> Box<dyn TaskTrait + 'a> {
        match self {
            Task::VerifyProof => Box::new(VerifyProofView { proof, cache }),
        }
    }
}

impl TryFrom<u8> for Task {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Task::VerifyProof,
            _ => return Err(ProgramError::Custom(2)),
        })
    }
}
