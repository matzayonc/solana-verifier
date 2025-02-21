use swiftness::types::CacheStark;
use swiftness::types::Felt;
use swiftness::types::StarkCommitment;
use swiftness::types::StarkWitness;
use swiftness_air::domains::StarkDomains;
use swiftness_air::layout::recursive::Layout;
use swiftness_air::layout::recursive::global_values::InteractionElements;
use swiftness_air::public_memory::PublicInput;
use swiftness_stark::verify::stark_verify;

use crate::task::TaskTrait;

struct StarkVerifyTask<'a> {
    cache: &'a mut CacheStark,
    n_original_columns: u32,
    n_interaction_columns: u32,
    public_input: &'a PublicInput,
    queries: &'a [Felt],
    commitment: &'a StarkCommitment<InteractionElements>,
    witness: &'a mut StarkWitness,
    stark_domains: &'a StarkDomains,
}

impl<'a> TaskTrait for StarkVerifyTask<'a> {
    fn execute(&mut self) {
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
    }
}
