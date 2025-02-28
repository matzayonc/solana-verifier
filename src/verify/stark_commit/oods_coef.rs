use swiftness::commit::powers_array;
use swiftness::types::Felt;
use swiftness::types::StarkProof;
use swiftness_air::layout::LayoutTrait;
use swiftness_air::layout::recursive_with_poseidon::Layout;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

use super::StarkCommitTask;

pub struct StarkCommitOodsCoefTask<'a> {
    parent: StarkCommitTask<'a>,
}

impl Task for StarkCommitOodsCoefTask<'_> {
    // stark_commit() - last part
    fn execute(&mut self) -> Vec<Tasks> {
        let StarkCommitTask {
            intermediate,
            transcript,
            cache,
            ..
        } = &mut self.parent;
        // Generate interaction values after OODS.
        intermediate.oods_alpha = transcript.random_felt_to_prover();

        cache.powers_array.powers_array.flush();
        let n = Layout::MASK_SIZE + Layout::CONSTRAINT_DEGREE;
        powers_array(
            cache.powers_array.powers_array.unchecked_slice_mut(n),
            Felt::ONE,
            intermediate.oods_alpha,
            n as u32,
        );

        self.children()
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> StarkCommitOodsCoefTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        StarkCommitOodsCoefTask {
            parent: StarkCommitTask::view(proof, cache, intermediate),
        }
    }
}
