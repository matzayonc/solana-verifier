use swiftness::commit::powers_array;
use swiftness::commit::stark_commit;
use swiftness::config::StarkConfig;
use swiftness::oods::verify_oods;
use swiftness::swiftness_fri::fri::fri_commit;
use swiftness::types::CacheStark;
use swiftness::types::Felt;
use swiftness::types::StarkCommitment;
use swiftness::types::StarkProof;
use swiftness::types::StarkUnsentCommitment;
use swiftness_air::Transcript;
use swiftness_air::domains::StarkDomains;
use swiftness_air::layout::LayoutTrait;
use swiftness_air::layout::recursive_with_poseidon::Layout;
use swiftness_air::public_memory::PublicInput;
use swiftness_air::swiftness_commitment::table::commit::table_commit;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::TaskResult;

pub struct StarkCommitTask<'a> {
    result: &'a mut StarkCommitment,
    cache: &'a mut CacheStark,
    transcript: &'a mut Transcript,
    public_input: &'a PublicInput,
    unsent_commitment: &'a StarkUnsentCommitment,
    config: &'a StarkConfig,
    stark_domains: &'a StarkDomains,
}

impl<'a> Task for StarkCommitTask<'a> {
    // stark_commit()
    fn execute(&mut self) -> TaskResult {
        let StarkCommitTask {
            result,
            cache,
            transcript,
            public_input,
            unsent_commitment,
            config,
            stark_domains,
        } = self;

        let traces_commitment =
            Layout::traces_commit(transcript, &unsent_commitment.traces, config.traces.clone());

        // Generate interaction values after traces commitment.
        let composition_alpha = transcript.random_felt_to_prover();
        powers_array(
            cache
                .powers_array
                .powers_array
                .unchecked_slice_mut(Layout::N_CONSTRAINTS),
            Felt::ONE,
            composition_alpha,
            Layout::N_CONSTRAINTS as u32,
        );
        let traces_coefficients = cache
            .powers_array
            .powers_array
            .unchecked_slice(Layout::N_CONSTRAINTS);

        // Read composition commitment.
        let composition_commitment = table_commit(
            transcript,
            unsent_commitment.composition,
            config.composition.clone(),
        );

        // Generate interaction values after composition.
        let interaction_after_composition = transcript.random_felt_to_prover();

        // Read OODS values.
        transcript.read_felt_vector_from_prover(&unsent_commitment.oods_values.to_vec());

        // // Check that the trace and the composition agree at oods_point.
        verify_oods::<Layout>(
            cache.commitment.verify_oods.inner(),
            unsent_commitment.oods_values.as_slice(),
            &traces_commitment.interaction_elements,
            public_input,
            &traces_coefficients,
            &interaction_after_composition,
            &stark_domains.trace_domain_size,
            &stark_domains.trace_generator,
        )
        .unwrap();

        // Generate interaction values after OODS.
        let oods_alpha = transcript.random_felt_to_prover();

        cache.powers_array.powers_array.flush();
        let n = Layout::MASK_SIZE + Layout::CONSTRAINT_DEGREE;
        powers_array(
            cache.powers_array.powers_array.unchecked_slice_mut(n),
            Felt::ONE,
            oods_alpha,
            n as u32,
        );
        let oods_coefficients = cache.powers_array.powers_array.unchecked_slice(n);

        // Read fri commitment.
        let fri_commitment = fri_commit(
            transcript,
            unsent_commitment.fri.clone(),
            config.fri.clone(),
        );

        // Proof of work commitment phase.
        unsent_commitment
            .proof_of_work
            .commit(transcript, &config.proof_of_work)
            .unwrap();

        let StarkCommitment {
            traces,
            composition,
            interaction_after_composition: interaction,
            oods_values,
            interaction_after_oods,
            fri,
        } = result;

        // Return commitment.
        *traces = traces_commitment;
        *composition = composition_commitment;
        *interaction = interaction_after_composition;
        *fri = fri_commitment;

        oods_values.overwrite(unsent_commitment.oods_values.as_slice());
        interaction_after_oods.overwrite(&oods_coefficients);

        Ok(vec![])
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
        }
    }
}
