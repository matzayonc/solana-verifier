use solana_program::program_error::ProgramError;
pub use swiftness_stark::types::{Felt, StarkProof};

use crate::Cache;
use crate::verify::stark_verify::StarkVerifyTask;
use crate::verify::stark_verify::table_decommit::{TableDecommitTarget, TableDecommitTask};
use crate::verify::verify_output::VerifyOutputTask;
use crate::{intermediate::Intermediate, verify::VerifyProofTask};

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum Tasks {
    #[default]
    VerifyProofWithoutStark = 1,
    StarkVerify = 2,
    VerifyOutput = 3,
    TableDecommit(TableDecommitTarget) = 4,
}

pub type TaskResult = Result<Vec<Tasks>, ()>;

pub type RawTask = [u8; 4];

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
            Tasks::VerifyOutput => Box::new(VerifyOutputTask::view(proof, cache, intermediate)),
            Tasks::TableDecommit(target) => {
                Box::new(TableDecommitTask::view(target, proof, cache, intermediate))
            }
        }
    }
}

impl TryFrom<&RawTask> for Tasks {
    type Error = ProgramError;

    fn try_from(value: &RawTask) -> Result<Self, Self::Error> {
        let [variant, tail @ ..] = value;

        Ok(match variant {
            1 => Tasks::VerifyProofWithoutStark,
            2 => Tasks::StarkVerify,
            3 => Tasks::VerifyOutput,
            4 => Tasks::TableDecommit(TableDecommitTarget::try_from(tail[0])?),
            _ => return Err(ProgramError::Custom(2)),
        })
    }
}

impl From<Tasks> for RawTask {
    fn from(task: Tasks) -> Self {
        match task {
            Tasks::VerifyProofWithoutStark => [1, 0, 0, 0],
            Tasks::StarkVerify => [2, 0, 0, 0],
            Tasks::VerifyOutput => [3, 0, 0, 0],
            Tasks::TableDecommit(target) => [4, target as u8, 0, 0],
        }
    }
}
