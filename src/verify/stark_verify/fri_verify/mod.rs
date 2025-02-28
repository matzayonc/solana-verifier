use swiftness::swiftness_fri::FriVerifyCache;
use swiftness::swiftness_fri::config::Config;
use swiftness::swiftness_fri::first_layer::gather_first_layer_queries;
use swiftness::swiftness_fri::fri::Error;
use swiftness::swiftness_fri::types;
use swiftness::types::Felt;
use swiftness::types::StarkProof;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

use super::StarkVerifyIntermediate;

pub mod fri_verify_layers;
pub mod last_layer;

pub struct StarkVerifyFriTask<'a> {
    config: &'a Config,
    cache: &'a mut FriVerifyCache,
    commitment: &'a types::Commitment,
    queries: &'a [Felt],
    decommitment: types::DecommitmentRef<'a>,
}

impl Task for StarkVerifyFriTask<'_> {
    // fri_verify(
    fn execute(&mut self) -> Vec<Tasks> {
        // Original

        let StarkVerifyFriTask {
            cache,
            decommitment,
            queries,
            ..
        } = self;

        // fri_verify(cache, queries, commitment, decommitment, witness).unwrap();

        if queries.len() != decommitment.values.len() {
            Result::<(), Error>::Err(Error::InvalidLength {
                expected: queries.len(),
                actual: decommitment.values.len(),
            })
            .unwrap();
        }

        // Compute first FRI layer queries.
        gather_first_layer_queries(
            &mut cache.fri_queries,
            queries,
            decommitment.values,
            decommitment.points,
        );

        self.children()
    }

    fn children(&self) -> Vec<Tasks> {
        vec![
            Tasks::StarkVerifyLayersTask,
            Tasks::StarkVerifyLastLayerTask,
        ]
    }
}

impl<'a> StarkVerifyFriTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        let StarkVerifyIntermediate {
            points,
            evaluations,
        } = &intermediate.stark_verify;

        // Decommit FRI.
        let decommitment: types::DecommitmentRef<'_> = types::DecommitmentRef {
            values: evaluations.as_slice(),
            points: points.as_slice(),
        };

        let cache = &mut cache.legacy.stark.fri;
        let commitment = &intermediate.verify.stark_commitment.fri;
        let _witness = &mut proof.witness.fri_witness;

        StarkVerifyFriTask {
            config: &proof.config.fri,
            cache,
            queries: intermediate.verify.queries.as_slice(),
            commitment,
            decommitment,
        }
    }
}
