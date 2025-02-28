use swiftness::commit::powers_array;
use swiftness::config::StarkConfig;
use swiftness::oods::verify_oods;
use swiftness::types::CacheStark;
use swiftness::types::Felt;
use swiftness::types::StarkCommitment;
use swiftness::types::StarkProof;
use swiftness::types::StarkUnsentCommitment;
use swiftness_air::Transcript;
use swiftness_air::domains::StarkDomains;
use swiftness_air::layout::LayoutTrait;
use swiftness_air::layout::recursive_with_poseidon::Layout;
use swiftness_air::layout::recursive_with_poseidon::global_values::InteractionElements;
use swiftness_air::public_memory::PublicInput;
use swiftness_air::swiftness_commitment::table::commit::table_commit;
use swiftness_air::trace::Commitment;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

mod assign;
mod fri_commit;
mod oods_coef;

pub use assign::*;
pub use fri_commit::*;
pub use oods_coef::*;

pub struct StarkCommitTask<'a> {
    result: &'a mut StarkCommitment,
    cache: &'a mut CacheStark,
    transcript: &'a mut Transcript,
    public_input: &'a PublicInput,
    unsent_commitment: &'a StarkUnsentCommitment,
    config: &'a StarkConfig,
    stark_domains: &'a StarkDomains,
    intermediate: &'a mut StarkCommitIntermediate,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct StarkCommitIntermediate {
    traces_commitment: Commitment<InteractionElements>,
    composition_alpha: Felt,
    composition_commitment: swiftness_air::Commitment,
    interaction_after_composition: Felt,
    oods_alpha: Felt,
    fri_commitment: swiftness::swiftness_fri::types::Commitment,
}

impl Task for StarkCommitTask<'_> {
    // stark_commit()
    fn execute(&mut self) -> Vec<Tasks> {
        let StarkCommitTask {
            cache,
            transcript,
            public_input,
            unsent_commitment,
            config,
            stark_domains,
            intermediate,
            result: _result,
        } = self;

        intermediate.traces_commitment =
            Layout::traces_commit(transcript, &unsent_commitment.traces, config.traces);

        // Generate interaction values after traces commitment.
        intermediate.composition_alpha = transcript.random_felt_to_prover();
        powers_array(
            cache
                .powers_array
                .powers_array
                .unchecked_slice_mut(Layout::N_CONSTRAINTS),
            Felt::ONE,
            intermediate.composition_alpha,
            Layout::N_CONSTRAINTS as u32,
        );
        let traces_coefficients = cache
            .powers_array
            .powers_array
            .unchecked_slice(Layout::N_CONSTRAINTS);

        // Read composition commitment.
        intermediate.composition_commitment = table_commit(
            transcript,
            unsent_commitment.composition,
            config.composition,
        );

        // Generate interaction values after composition.
        intermediate.interaction_after_composition = transcript.random_felt_to_prover();

        // Read OODS values.
        transcript.read_felt_vector_from_prover(&unsent_commitment.oods_values.to_vec());

        // // Check that the trace and the composition agree at oods_point.
        verify_oods::<Layout>(
            cache.commitment.verify_oods.inner(),
            unsent_commitment.oods_values.as_slice(),
            &intermediate.traces_commitment.interaction_elements,
            public_input,
            traces_coefficients,
            &intermediate.interaction_after_composition,
            &stark_domains.trace_domain_size,
            &stark_domains.trace_generator,
        )
        .unwrap();

        self.children()
    }

    fn children(&self) -> Vec<Tasks> {
        vec![
            Tasks::StarkCommitOodsCoef,
            Tasks::StarkCommitFri,
            Tasks::StarkCommitAssign,
        ]
    }
}

impl<'a> StarkCommitTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        StarkCommitTask {
            result: &mut intermediate.verify.stark_commitment,
            cache: &mut cache.legacy.stark,
            transcript: &mut intermediate.verify.transcript,
            public_input: &proof.public_input,
            unsent_commitment: &proof.unsent_commitment,
            config: &proof.config,
            stark_domains: &intermediate.verify.stark_domains,
            intermediate: &mut intermediate.stark_commit,
        }
    }
}
