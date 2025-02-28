use swiftness::swiftness_fri::FriVerifyCache;
use swiftness::types::StarkProof;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

pub struct StarkVerifyLayerAssignNextTask<'a> {
    cache: &'a mut FriVerifyCache,
}

impl Task for StarkVerifyLayerAssignNextTask<'_> {
    // fri_verify_layers(
    fn execute(&mut self) {
        let FriVerifyCache {
            fri_queries,
            next_layer_cache,
            ..
        } = self.cache;

        fri_queries.flush();
        fri_queries.extend(next_layer_cache.next_queries.as_slice());
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> StarkVerifyLayerAssignNextTask<'a> {
    pub fn view(
        _proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        _intermediate: &'a mut Intermediate,
    ) -> Self {
        Self {
            cache: &mut cache.legacy.stark.fri,
        }
    }
}
