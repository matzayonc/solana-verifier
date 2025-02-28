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

use super::layer::StarkVerifyLayerContext;
use super::layer::StarkVerifyLayerTask;

pub struct ComputeNextTask<'a> {
    pub parent: StarkVerifyLayerTask<'a>,
}

impl Task for ComputeNextTask<'_> {
    // compute_next_layer(
    fn execute(&mut self) -> Vec<Tasks> {
        // Original

        let StarkVerifyLayerTask { cache, context, .. } = &mut self.parent;

        let FriVerifyCache {
            fri_queries,
            next_layer_cache,
            ..
        } = cache;

        let Some(StarkVerifyLayerContext {
            target_layer_witness_leaves: sibling_witness,
            params,
            ..
        }) = context
        else {
            panic!("Not enough data in context");
        };

        // Original function.
        compute_next_layer(
            next_layer_cache,
            fri_queries,
            sibling_witness,
            params.clone(),
        )
        .unwrap();

        self.children()
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> ComputeNextTask<'a> {
    pub fn view(
        layer_index: usize,
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        ComputeNextTask {
            parent: StarkVerifyLayerTask::view(layer_index, proof, cache, intermediate),
        }
    }
}
