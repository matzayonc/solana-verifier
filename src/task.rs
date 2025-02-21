use solana_program::program_error::ProgramError;
pub use swiftness_stark::types::{Cache, Felt, StarkProof};

use crate::verify::stark_verify::StarkVerifyTask;
use crate::{intermediate::Intermediate, verify::VerifyProofTask};

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum Tasks {
    #[default]
    VerifyProofWithoutStark = 1,
    StarkVerify = 2,
}

pub type TaskResult = Result<Vec<Tasks>, ()>;

pub trait Task {
    fn execute(&mut self) -> TaskResult;
}

impl Tasks {
    pub fn view<'a>(
        self,
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Box<dyn Task + 'a> {
        match self {
            Tasks::VerifyProofWithoutStark => {
                Box::new(VerifyProofTask::view(proof, cache, intermediate))
            }
            Tasks::StarkVerify => Box::new(StarkVerifyTask::view(proof, cache, intermediate)),
        }
    }
}

impl TryFrom<u8> for Tasks {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Tasks::VerifyProofWithoutStark,
            2 => Tasks::StarkVerify,
            _ => return Err(ProgramError::Custom(2)),
        })
    }
}
