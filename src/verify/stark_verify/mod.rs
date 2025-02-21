use swiftness::types::Cache;
use swiftness::types::CacheStark;
use swiftness::types::Felt;
use swiftness::types::StarkCommitment;
use swiftness::types::StarkProof;
use swiftness::types::StarkWitness;
use swiftness_air::domains::StarkDomains;
use swiftness_air::layout::recursive::Layout;
use swiftness_air::layout::recursive::global_values::InteractionElements;
use swiftness_air::public_memory::PublicInput;
use swiftness_stark::verify::stark_verify;

use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::TaskResult;

pub struct StarkVerifyTask<'a> {
    pub cache: &'a mut CacheStark,
    pub n_original_columns: u32,
    pub n_interaction_columns: u32,
    pub public_input: &'a PublicInput,
    pub queries: &'a [Felt],
    pub commitment: &'a StarkCommitment<InteractionElements>,
    pub witness: &'a mut StarkWitness,
    pub stark_domains: &'a StarkDomains,
}

impl<'a> Task for StarkVerifyTask<'a> {
    fn execute(&mut self) -> TaskResult {
        stark_verify::<Layout>(
            self.cache,
            self.n_original_columns,
            self.n_interaction_columns,
            self.public_input,
            self.queries,
            self.commitment,
            self.witness,
            self.stark_domains,
        )
        .unwrap();

        Ok(vec![])
    }
}

impl<'a> StarkVerifyTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        StarkVerifyTask {
            cache: &mut cache.stark,
            n_original_columns: intermediate.verify.n_original_columns,
            n_interaction_columns: intermediate.verify.n_interaction_columns,
            public_input: &proof.public_input,
            queries: &intermediate.verify.queries.as_slice(),
            commitment: &intermediate.verify.stark_commitment,
            witness: &mut proof.witness,
            stark_domains: &intermediate.verify.stark_domains,
        }
    }
}
