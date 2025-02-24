use bytemuck::Pod;
use bytemuck::Zeroable;
use solana_program::program_error::ProgramError;
use swiftness::stark::CacheCommitment;
use swiftness::types::Felt;
use swiftness::types::StarkProof;
use swiftness_air::Commitment;
use swiftness_air::Decommitment;
use swiftness_air::Witness;
use swiftness_air::swiftness_commitment::table::decommit::table_decommit;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::TaskResult;

pub struct TableDecommitTask<'a> {
    pub cache: &'a mut TableDecommitCache,
    pub commitment: &'a Commitment,
    pub queries: &'a [Felt],
    pub decommitment: &'a Decommitment,
    pub witness: &'a Witness,
}

#[derive(Debug, Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct TableDecommitCache {
    pub commitment: CacheCommitment, // TODO: minimize this;
}

impl<'a> Task for TableDecommitTask<'a> {
    fn execute(&mut self) -> TaskResult {
        table_decommit(
            &mut self.cache.commitment,
            self.commitment,
            self.queries,
            self.decommitment,
            self.witness,
        )
        .map_err(|_| ())?;

        Ok(vec![])
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum TableDecommitTarget {
    #[default]
    Invalid = 0,
    Original = 1,
    Interaction = 2,
    Composition = 3,
}

impl TryFrom<u8> for TableDecommitTarget {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TableDecommitTarget::Invalid),
            1 => Ok(TableDecommitTarget::Original),
            2 => Ok(TableDecommitTarget::Interaction),
            3 => Ok(TableDecommitTarget::Composition),
            _ => Err(ProgramError::Custom(17)),
        }
    }
}

impl<'a> TableDecommitTask<'a> {
    pub fn view(
        variant: TableDecommitTarget,
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        let queries = intermediate.verify.queries.as_slice();
        let cache = &mut cache.table;

        let commitment = &intermediate.verify.stark_commitment;
        let decommitment = &proof.witness.traces_decommitment;
        let witness = &proof.witness.traces_witness;

        match variant {
            TableDecommitTarget::Original => TableDecommitTask {
                cache,
                commitment: &commitment.traces.original,
                queries,
                decommitment: &decommitment.original,
                witness: &witness.original,
            },
            TableDecommitTarget::Interaction => TableDecommitTask {
                cache,
                commitment: &commitment.traces.interaction,
                queries,
                decommitment: &decommitment.interaction,
                witness: &witness.interaction,
            },
            TableDecommitTarget::Composition => TableDecommitTask {
                cache,
                commitment: &commitment.composition,
                queries,
                decommitment: &proof.witness.composition_decommitment,
                witness: &proof.witness.composition_witness,
            },
            TableDecommitTarget::Invalid => unreachable!("TableDecommitTarget::Invalid"),
        }
    }
}
