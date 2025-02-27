use swiftness::swiftness_fri::ComputeNextLayerCache;
use swiftness::swiftness_fri::FriVerifyCache;
use swiftness::swiftness_fri::group::get_fri_group;
use swiftness::swiftness_fri::layer::FriLayerComputationParams;
use swiftness::swiftness_fri::layer::compute_next_layer;
use swiftness::types::Felt;
use swiftness::types::StarkProof;
use swiftness_air::swiftness_commitment::table::decommit::MONTGOMERY_R;
use swiftness_air::swiftness_commitment::table::decommit::table_decommit;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

use super::StarkVerifyFriTask;

pub struct StarkVerifyLayerTask<'a> {
    parent: StarkVerifyFriTask<'a>,
    layer_index: usize,
}

impl Task for StarkVerifyLayerTask<'_> {
    // fri_verify_layers(
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

        // Prepare params
        // let n_layers = commitment.config.n_layers - 1;
        let eval_points = commitment.eval_points.as_slice();
        let commitment = commitment.inner_layers.as_slice();
        let layer_witness = witness.layers.as_slice_mut();
        let step_sizes = &fri_step_sizes[1..fri_step_sizes.len()];

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

        let FriVerifyCache {
            fri_queries,
            next_layer_cache,
            decommitment,
            ..
        } = cache;

        let i = self.layer_index;

        // let len: usize = funvec::cast_felt(&n_layers) as usize;
        // for i in 0..len {
        let target_layer_witness = layer_witness.get_mut(i).unwrap();
        let target_layer_witness_leaves = &mut target_layer_witness.leaves;
        let target_layer_witness_table_withness = &target_layer_witness.table_witness;
        let target_commitment = commitment.get(i).unwrap();

        // Params.
        let coset_size = Felt::TWO.pow_felt(step_sizes.get(i).unwrap());
        let params = FriLayerComputationParams {
            coset_size: &coset_size,
            fri_group,
            eval_point: eval_points.get(i).unwrap(),
        };

        // Compute next layer queries.
        compute_next_layer(
            next_layer_cache,
            fri_queries,
            target_layer_witness_leaves,
            params,
        )
        .unwrap();
        let ComputeNextLayerCache {
            next_queries,
            verify_indices,
            verify_y_values,
            ..
        } = next_layer_cache;

        decommitment.values.flush();
        decommitment.montgomery_values.flush();
        decommitment.values.extend(verify_y_values.as_slice());
        for i in 0..verify_y_values.len() {
            decommitment
                .montgomery_values
                .push(verify_y_values.get(i).unwrap() * MONTGOMERY_R);
        }

        // Table decommitment.
        let _ = table_decommit(
            &mut cache.commitment,
            &target_commitment,
            verify_indices.as_slice(),
            &decommitment,
            &target_layer_witness_table_withness,
        );

        fri_queries.flush();
        fri_queries.extend(next_queries.as_slice());
        // }
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> StarkVerifyLayerTask<'a> {
    pub fn view(
        layer_index: usize,
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        StarkVerifyLayerTask {
            parent: StarkVerifyFriTask::view(proof, cache, intermediate),
            layer_index,
        }
    }
}
