use swiftness::funvec;
use swiftness::types::StarkProof;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

use super::StarkVerifyFriTask;

pub mod assign_next;
pub mod layer;

pub struct StarkVerifyLayersTask<'a> {
    parent: StarkVerifyFriTask<'a>,
}

impl Task for StarkVerifyLayersTask<'_> {
    // fri_verify_layers(
    fn execute(&mut self) {
        // Original

        // let StarkVerifyFriTask {
        //     cache,
        //     commitment,
        //     witness,
        //     ..
        // } = &mut self.parent;

        // // Compute fri_group.
        // let fri_group: &[Felt; 16] = &get_fri_group();
        // let fri_step_sizes = commitment.config.fri_step_sizes.as_slice();

        // // Prepare params
        // let n_layers = commitment.config.n_layers - 1;
        // let eval_points = commitment.eval_points.as_slice();
        // let commitment = commitment.inner_layers.as_slice();
        // let layer_witness = witness.layers.as_slice_mut();
        // let step_sizes = &fri_step_sizes[1..fri_step_sizes.len()];

        // Verify inner layers.
        // let _last_queries = fri_verify_layers(
        //     cache,
        //     fri_group,
        //     n_layers,
        //     commitment,
        //     layer_witness,
        //     eval_points,
        //     step_sizes,
        // );
    }

    fn children(&self) -> Vec<Tasks> {
        let n_layers = funvec::cast_felt(&self.parent.config.n_layers) as usize;
        (0..(n_layers - 1))
            .map(|i| Tasks::StarkVerifyFriLayer(i))
            .collect()
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
