use swiftness::swiftness_fri::fri::fri_verify_layers;
use swiftness::swiftness_fri::group::get_fri_group;
use swiftness::types::Felt;
use swiftness::types::StarkProof;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

use super::StarkVerifyFriTask;

pub struct StarkVerifyLayersTask<'a> {
    parent: StarkVerifyFriTask<'a>,
}

impl Task for StarkVerifyLayersTask<'_> {
    // fri_verify(
    fn execute(&mut self) {
        // Original

        let StarkVerifyFriTask {
            cache,
            commitment,
            witness,
            ..
        } = &mut self.parent;

        // Compute fri_group.
        let fri_group: &[Felt; 16] = &get_fri_group();

        let fri_step_sizes = commitment.config.fri_step_sizes.as_slice();

        // Verify inner layers.
        let _last_queries = fri_verify_layers(
            cache,
            fri_group,
            commitment.config.n_layers - 1,
            commitment.inner_layers.as_slice(),
            witness.layers.as_slice_mut(),
            commitment.eval_points.as_slice(),
            &fri_step_sizes[1..fri_step_sizes.len()],
            // fri_queries,
        );
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> StarkVerifyLayersTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        StarkVerifyLayersTask {
            parent: StarkVerifyFriTask::view(proof, cache, intermediate),
        }
    }
}
