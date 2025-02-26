use swiftness::stark::Error;
use swiftness_air::{
    Transcript,
    domains::StarkDomains,
    layout::{GenericLayoutTrait, LayoutTrait, recursive_with_poseidon::Layout},
};
pub use swiftness_stark::types::StarkProof;

use crate::{
    Cache,
    intermediate::{Intermediate, VerifyIntermediate},
    task::{Task, Tasks},
};

pub mod generate_queries;
pub mod stark_commit;
pub mod stark_verify;
pub mod verify_output;

#[derive(Debug)]
pub struct VerifyProofTask<'a> {
    proof: &'a mut StarkProof,
    intermediate: &'a mut VerifyIntermediate,
}

impl Task for VerifyProofTask<'_> {
    // let _res = self.proof.verify::<Layout>(self.cache, security_bits);
    fn execute(&mut self) {
        let security_bits = self.proof.config.security_bits();

        let VerifyIntermediate {
            n_original_columns,
            n_interaction_columns,
            stark_domains,
            transcript,
            ..
        } = self.intermediate;

        *n_original_columns = Layout::get_num_columns_first(&self.proof.public_input)
            .ok_or(Error::ColumnMissing)
            .unwrap();

        *n_interaction_columns = Layout::get_num_columns_second(&self.proof.public_input)
            .ok_or(Error::ColumnMissing)
            .unwrap();

        self.proof
            .config
            .validate(
                security_bits,
                (*n_original_columns).into(),
                (*n_interaction_columns).into(),
            )
            .unwrap();

        // Validate the public input.
        *stark_domains = StarkDomains::new(
            self.proof.config.log_trace_domain_size,
            self.proof.config.log_n_cosets,
        );

        Layout::validate_public_input(&self.proof.public_input, stark_domains).unwrap();

        // Compute the initial hash seed for the Fiat-Shamir transcript.
        // Construct the transcript.
        *transcript = Transcript::new(
            self.proof
                .public_input
                .get_hash(self.proof.config.n_verifier_friendly_commitment_layers),
        );
    }

    fn children(&self) -> Vec<Tasks> {
        vec![
            Tasks::StarkCommit,
            Tasks::GenerateQueries,
            Tasks::StarkVerify,
            Tasks::VerifyOutput,
        ]
    }
}

impl<'a> VerifyProofTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        _cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        VerifyProofTask {
            proof,
            intermediate: &mut intermediate.verify,
        }
    }
}
