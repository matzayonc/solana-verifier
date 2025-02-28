use swiftness::funvec::FunVec;
use swiftness::swiftness_fri::ComputeNextLayerCache;
use swiftness::swiftness_fri::FriVerifyCache;
use swiftness::swiftness_fri::group::FRI_GROUP;
use swiftness::swiftness_fri::layer::FriLayerComputationParams;
use swiftness::swiftness_fri::layer::compute_next_layer;
use swiftness::types::Felt;
use swiftness::types::StarkProof;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;
use crate::verify::stark_verify::table_decommit::TableDecommitCache;
use crate::verify::stark_verify::table_decommit::TableDecommitTarget;
use crate::verify::stark_verify::table_decommit::TableDecommitTask;

pub struct StarkVerifyLayerTask<'a> {
    pub layer_index: usize,
    pub cache: &'a mut FriVerifyCache,
    pub table_cache: &'a mut TableDecommitCache,
    pub context: Option<StarkVerifyLayerContext<'a>>,
}

pub struct StarkVerifyLayerContext<'a> {
    pub target_layer_witness_leaves: &'a mut FunVec<Felt, 512>,
    pub target_layer_witness_table_withness: &'a swiftness_air::Witness,
    pub target_commitment: &'a swiftness_air::Commitment,
    pub params: FriLayerComputationParams<'a>,
}

impl Task for StarkVerifyLayerTask<'_> {
    // fri_verify_layers(
    fn execute(&mut self) -> Vec<Tasks> {
        // Original

        let Self { cache, context, .. } = self;

        let FriVerifyCache {
            fri_queries,
            next_layer_cache,
            ..
        } = cache;

        let Some(StarkVerifyLayerContext {
            target_layer_witness_leaves,
            params,
            ..
        }) = context
        else {
            panic!("Not enough data in context");
        };

        self.children()
    }

    fn children(&self) -> Vec<Tasks> {
        vec![
            Tasks::ComputeNextLayer(self.layer_index),
            Tasks::StarkVerifyLayerDecommitmentMont(self.layer_index),
            Tasks::TableDecommit(TableDecommitTarget::Fri(self.layer_index as u8)),
            Tasks::StarkVerifyLayerAssignNext,
        ]
    }
}

impl<'a> StarkVerifyLayerTask<'a> {
    pub fn view(
        layer_index: usize,
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        let Cache { legacy, table } = cache;
        let cache = &mut legacy.stark.fri;
        let commitment = &intermediate.verify.stark_commitment.fri;
        let witness = &mut proof.witness.fri_witness;

        let fri_step_sizes = commitment.config.fri_step_sizes.as_slice();

        let eval_points = commitment.eval_points.as_slice();
        let commitment_layer = commitment.inner_layers.as_slice();
        let layer_witness = witness.layers.as_slice_mut();

        let context = if fri_step_sizes.len() != 0 {
            let step_sizes = &fri_step_sizes[1..fri_step_sizes.len()];

            let target_layer_witness = layer_witness.get_mut(layer_index).unwrap();
            let target_layer_witness_leaves = &mut target_layer_witness.leaves;
            let target_layer_witness_table_withness = &target_layer_witness.table_witness;
            let target_commitment = commitment_layer.get(layer_index).unwrap();

            // Params.
            let coset_size = Felt::TWO.pow_felt(step_sizes.get(layer_index).unwrap());
            let params = FriLayerComputationParams {
                coset_size,
                fri_group: &FRI_GROUP,
                eval_point: *eval_points.get(layer_index).unwrap(),
            };

            let context = StarkVerifyLayerContext {
                target_layer_witness_leaves,
                target_layer_witness_table_withness,
                target_commitment,
                params,
            };

            Some(context)
        } else {
            None
        };

        StarkVerifyLayerTask {
            layer_index,
            cache,
            table_cache: table,
            context,
        }
    }
}

impl<'a> Into<TableDecommitTask<'a>> for StarkVerifyLayerTask<'a> {
    fn into(self) -> TableDecommitTask<'a> {
        let StarkVerifyLayerTask {
            cache,
            context,
            table_cache,
            ..
        } = self;

        let FriVerifyCache {
            next_layer_cache,
            decommitment,
            ..
        } = cache;

        let Some(StarkVerifyLayerContext {
            target_layer_witness_table_withness,
            target_commitment,
            ..
        }) = context
        else {
            panic!("Not enough data in context");
        };

        let ComputeNextLayerCache { verify_indices, .. } = next_layer_cache;

        TableDecommitTask {
            cache: table_cache,
            commitment: &target_commitment,
            queries: verify_indices.as_slice(),
            decommitment: decommitment,
            witness: &target_layer_witness_table_withness,
        }
    }
}
