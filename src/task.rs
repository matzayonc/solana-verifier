use solana_program::program_error::ProgramError;
pub use swiftness_stark::types::{Felt, StarkProof};

use crate::Cache;
use crate::verify::generate_queries::GenerateQueriesTask;
use crate::verify::stark_commit::{
    StarkCommitAssignTask, StarkCommitFriTask, StarkCommitOodsCoefTask, StarkCommitTask,
};
use crate::verify::stark_verify::StarkVerifyTask;
use crate::verify::stark_verify::fri_verify::StarkVerifyFriTask;
use crate::verify::stark_verify::fri_verify::fri_verify_layers::StarkVerifyLayersTask;
use crate::verify::stark_verify::fri_verify::fri_verify_layers::assign_next::StarkVerifyLayerAssignNextTask;
use crate::verify::stark_verify::fri_verify::fri_verify_layers::compute_next_layer::ComputeNextTask;
use crate::verify::stark_verify::fri_verify::fri_verify_layers::decommitment_mont::StarkVerifyLayerDecommitmentMontTask;
use crate::verify::stark_verify::fri_verify::fri_verify_layers::layer::StarkVerifyLayerTask;
use crate::verify::stark_verify::fri_verify::last_layer::StarkVerifyLastLayerTask;
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
    StarkCommit = 5,
    StarkCommitOodsCoef = 6,
    StarkCommitFri = 7,
    StarkCommitAssign = 8,
    GenerateQueries = 9,
    StarkVerifyFri = 10,
    StarkVerifyLayersTask = 11,
    StarkVerifyLastLayerTask = 12,
    StarkVerifyFriLayer(usize) = 13,
    StarkVerifyLayerAssignNext = 14,
    StarkVerifyLayerDecommitmentMont(usize) = 15,
    ComputeNextLayer(usize) = 16,
}

pub type RawTask = [u8; 4];

pub trait Task {
    fn execute(&mut self) -> Vec<Tasks>;
    fn children(&self) -> Vec<Tasks>;
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
            Tasks::StarkCommit => Box::new(StarkCommitTask::view(proof, cache, intermediate)),
            Tasks::GenerateQueries => {
                Box::new(GenerateQueriesTask::view(proof, cache, intermediate))
            }
            Tasks::StarkCommitOodsCoef => {
                Box::new(StarkCommitOodsCoefTask::view(proof, cache, intermediate))
            }
            Tasks::StarkCommitFri => Box::new(StarkCommitFriTask::view(proof, cache, intermediate)),
            Tasks::StarkCommitAssign => {
                Box::new(StarkCommitAssignTask::view(proof, cache, intermediate))
            }
            Tasks::StarkVerifyFri => Box::new(StarkVerifyFriTask::view(proof, cache, intermediate)),
            Tasks::StarkVerifyLayersTask => {
                Box::new(StarkVerifyLayersTask::view(proof, cache, intermediate))
            }
            Tasks::StarkVerifyLastLayerTask => {
                Box::new(StarkVerifyLastLayerTask::view(proof, cache, intermediate))
            }
            Tasks::StarkVerifyFriLayer(i) => {
                Box::new(StarkVerifyLayerTask::view(i, proof, cache, intermediate))
            }
            Tasks::StarkVerifyLayerAssignNext => Box::new(StarkVerifyLayerAssignNextTask::view(
                proof,
                cache,
                intermediate,
            )),
            Tasks::StarkVerifyLayerDecommitmentMont(i) => Box::new(
                StarkVerifyLayerDecommitmentMontTask::view(i, proof, cache, intermediate),
            ),
            Tasks::ComputeNextLayer(i) => {
                Box::new(ComputeNextTask::view(i, proof, cache, intermediate))
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
            4 => Tasks::TableDecommit(TableDecommitTarget::try_from([tail[0], tail[1]])?),
            5 => Tasks::StarkCommit,
            6 => Tasks::StarkCommitOodsCoef,
            7 => Tasks::StarkCommitFri,
            8 => Tasks::StarkCommitAssign,
            9 => Tasks::GenerateQueries,
            10 => Tasks::StarkVerifyFri,
            11 => Tasks::StarkVerifyLayersTask,
            12 => Tasks::StarkVerifyLastLayerTask,
            13 => Tasks::StarkVerifyFriLayer(tail[0] as usize),
            14 => Tasks::StarkVerifyLayerAssignNext,
            15 => Tasks::StarkVerifyLayerDecommitmentMont(tail[0] as usize),
            16 => Tasks::ComputeNextLayer(tail[0] as usize),
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
            Tasks::TableDecommit(target) => match target {
                TableDecommitTarget::Invalid => [4, 0, 0, 0],
                TableDecommitTarget::Original => [4, 1, 0, 0],
                TableDecommitTarget::Interaction => [4, 2, 0, 0],
                TableDecommitTarget::Composition => [4, 3, 0, 0],
                TableDecommitTarget::Fri(i) => [4, 4, i as u8, 0],
            },
            Tasks::StarkCommit => [5, 0, 0, 0],
            Tasks::StarkCommitOodsCoef => [6, 0, 0, 0],
            Tasks::StarkCommitFri => [7, 0, 0, 0],
            Tasks::StarkCommitAssign => [8, 0, 0, 0],
            Tasks::GenerateQueries => [9, 0, 0, 0],
            Tasks::StarkVerifyFri => [10, 0, 0, 0],
            Tasks::StarkVerifyLayersTask => [11, 0, 0, 0],
            Tasks::StarkVerifyLastLayerTask => [12, 0, 0, 0],
            Tasks::StarkVerifyFriLayer(i) => [13, i as u8, 0, 0],
            Tasks::StarkVerifyLayerAssignNext => [14, 0, 0, 0],
            Tasks::StarkVerifyLayerDecommitmentMont(i) => [15, i as u8, 0, 0],
            Tasks::ComputeNextLayer(i) => [16, i as u8, 0, 0],
        }
    }
}
