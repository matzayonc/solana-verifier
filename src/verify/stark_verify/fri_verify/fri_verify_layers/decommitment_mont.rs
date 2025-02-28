use swiftness::swiftness_fri::ComputeNextLayerCache;
use swiftness::swiftness_fri::FriVerifyCache;
use swiftness::types::StarkProof;
use swiftness_air::swiftness_commitment::table::decommit::MONTGOMERY_R;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

use super::layer::StarkVerifyLayerTask;

pub struct StarkVerifyLayerDecommitmentMontTask<'a> {
    parent: StarkVerifyLayerTask<'a>,
}

impl Task for StarkVerifyLayerDecommitmentMontTask<'_> {
    // fri_verify_layers(
    fn execute(&mut self) -> Vec<Tasks> {
        let FriVerifyCache {
            next_layer_cache,
            decommitment,
            ..
        } = self.parent.cache;

        let ComputeNextLayerCache {
            verify_y_values, ..
        } = next_layer_cache;

        decommitment.values.flush();
        decommitment.montgomery_values.flush();
        decommitment.values.extend(verify_y_values.as_slice());
        for i in 0..verify_y_values.len() {
            decommitment
                .montgomery_values
                .push(verify_y_values.get(i).unwrap() * MONTGOMERY_R);
        }

        vec![]
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> StarkVerifyLayerDecommitmentMontTask<'a> {
    pub fn view(
        layer_index: usize,
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        Self {
            parent: StarkVerifyLayerTask::view(layer_index, proof, cache, intermediate),
        }
    }
}
