use swiftness::swiftness_fri::ComputeNextLayerCache;
use swiftness::swiftness_fri::FriVerifyCache;

use swiftness::types::StarkProof;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

use super::layer::StarkVerifyLayerTask;

pub mod next_inner;

pub struct ComputeNextTask<'a> {
    pub parent: StarkVerifyLayerTask<'a>,
}

impl Task for ComputeNextTask<'_> {
    // compute_next_layer(
    fn execute(&mut self) -> Vec<Tasks> {
        // Original

        let StarkVerifyLayerTask { cache, .. } = &mut self.parent;

        let FriVerifyCache {
            fri_queries: queries,
            next_layer_cache,
            ..
        } = cache;

        let ComputeNextLayerCache {
            next_queries,
            verify_indices,
            verify_y_values,
            ..
        } = next_layer_cache;

        next_queries.flush();
        verify_indices.flush();
        verify_y_values.flush();

        if queries.is_empty() {
            return vec![];
        }

        self.children()
    }

    fn children(&self) -> Vec<Tasks> {
        vec![Tasks::ComputeNextInner(self.parent.layer_index)]
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
