use swiftness::swiftness_fri::FriVerifyCache;
use swiftness::swiftness_fri::fri::Error;
use swiftness::swiftness_fri::last_layer::verify_last_layer;
use swiftness::types::Felt;
use swiftness::types::StarkProof;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

use super::StarkVerifyFriTask;

pub struct StarkVerifyLastLayerTask<'a> {
    parent: StarkVerifyFriTask<'a>,
}

impl Task for StarkVerifyLastLayerTask<'_> {
    // fri_verify(
    fn execute(&mut self) -> Vec<Tasks> {
        // Original

        let StarkVerifyFriTask {
            cache, commitment, ..
        } = &mut self.parent;

        let FriVerifyCache { fri_queries, .. } = cache;

        if Felt::from(commitment.last_layer_coefficients.len())
            != Felt::TWO.pow_felt(&commitment.config.log_last_layer_degree_bound)
        {
            Result::<(), Error>::Err(Error::InvalidValue).unwrap();
        };

        verify_last_layer(
            fri_queries.as_slice(),
            commitment.last_layer_coefficients.as_slice(),
        )
        .map_err(|_| Error::LastLayerVerificationError)
        .unwrap();

        self.children()
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> StarkVerifyLastLayerTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        Self {
            parent: StarkVerifyFriTask::view(proof, cache, intermediate),
        }
    }
}
