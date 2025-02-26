use swiftness::types::StarkCommitment;
use swiftness::types::StarkProof;
use swiftness_air::layout::LayoutTrait;
use swiftness_air::layout::recursive_with_poseidon::Layout;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

use super::StarkCommitTask;

pub struct StarkCommitAssignTask<'a> {
    parent: StarkCommitTask<'a>,
}

impl Task for StarkCommitAssignTask<'_> {
    // stark_commit() - last part
    fn execute(&mut self) {
        let StarkCommitTask {
            result,
            cache,
            unsent_commitment,
            intermediate,
            transcript,
            config,
            ..
        } = &mut self.parent;

        // Proof of work commitment phase.
        unsent_commitment
            .proof_of_work
            .commit(transcript, &config.proof_of_work)
            .unwrap();

        let n = Layout::MASK_SIZE + Layout::CONSTRAINT_DEGREE;
        let oods_coefficients = cache.powers_array.powers_array.unchecked_slice(n);

        let StarkCommitment {
            traces,
            composition,
            interaction_after_composition: interaction,
            oods_values,
            interaction_after_oods,
            fri,
        } = result;

        // Return commitment.
        *traces = intermediate.traces_commitment;
        *composition = intermediate.composition_commitment;
        *interaction = intermediate.interaction_after_composition;
        *fri = intermediate.fri_commitment;

        oods_values.overwrite(unsent_commitment.oods_values.as_slice());
        interaction_after_oods.overwrite(oods_coefficients);
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> StarkCommitAssignTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        StarkCommitAssignTask {
            parent: StarkCommitTask::view(proof, cache, intermediate),
        }
    }
}
