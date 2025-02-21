pub mod stark_verify;

use swiftness::{commit::stark_commit, queries::generate_queries, stark::Error};
use swiftness_air::{
    Transcript,
    domains::StarkDomains,
    layout::{GenericLayoutTrait, LayoutTrait, recursive::Layout},
};
pub use swiftness_stark::types::{Cache, StarkProof};

use crate::{
    ProofAccount,
    intermediate::{Intermediate, VerifyIntermediate},
    task::{Task, Tasks},
};

#[derive(Debug)]
pub struct VerifyProofTask<'a> {
    proof: &'a mut StarkProof,
    cache: &'a mut Cache,
    intermediate: &'a mut VerifyIntermediate,
}

impl<'a> From<&'a mut ProofAccount> for VerifyProofTask<'a> {
    fn from(proof: &'a mut ProofAccount) -> Self {
        VerifyProofTask {
            proof: &mut proof.proof,
            cache: &mut proof.cache,
            intermediate: &mut proof.intermediate.verify,
        }
    }
}

impl<'a> Task for VerifyProofTask<'a> {
    fn execute(&mut self) -> Result<Vec<Tasks>, ()> {
        let security_bits = self.proof.config.security_bits();
        // let _res = self.proof.verify::<Layout>(self.cache, security_bits);

        let VerifyIntermediate {
            n_original_columns,
            n_interaction_columns,
            stark_domains,
            transcript,
            stark_commitment,
            queries,
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

        Layout::validate_public_input(&self.proof.public_input, &stark_domains).unwrap();

        // Compute the initial hash seed for the Fiat-Shamir transcript.
        // Construct the transcript.
        *transcript = Transcript::new(
            self.proof
                .public_input
                .get_hash(self.proof.config.n_verifier_friendly_commitment_layers),
        );

        let Cache { stark, .. } = self.cache;

        // STARK commitment phase.
        *stark_commitment = stark_commit::<Layout>(
            stark,
            transcript,
            &self.proof.public_input,
            &self.proof.unsent_commitment,
            &self.proof.config,
            &stark_domains,
        )
        .unwrap();

        // Generate queries.
        queries.move_to(generate_queries(
            transcript,
            self.proof.config.n_queries,
            stark_domains.eval_domain_size,
        ));

        // Moves queries to the cache to free up memory.
        // queries = self.cache.verify.queries.move_to(queries);

        Ok(vec![Tasks::StarkVerify])
    }
}

impl<'a> VerifyProofTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        VerifyProofTask {
            proof,
            cache,
            intermediate: &mut intermediate.verify,
        }
    }
}
