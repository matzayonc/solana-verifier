use swiftness::oods::OodsEvaluationInfo;
use swiftness::oods::eval_oods_boundary_poly_at_points;
use swiftness::queries::queries_to_points;
use swiftness::stark::CacheCommitment;
use swiftness::swiftness_fri::fri::fri_verify;
use swiftness::swiftness_fri::types;
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
use table_decommit::TableDecommitTarget;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::TaskResult;
use crate::task::Tasks;

pub mod table_decommit;

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
        // stark_verify::<Layout>(
        //     self.cache,
        //     self.n_original_columns,
        //     self.n_interaction_columns,
        //     self.public_input,
        //     self.queries,
        //     self.commitment,
        //     self.witness,
        //     self.stark_domains,
        // )
        // .unwrap();

        let StarkVerifyTask {
            cache,
            n_original_columns,
            n_interaction_columns,
            public_input,
            queries,
            commitment,
            witness,
            stark_domains,
        } = self;

        let CacheCommitment {
            points, eval_oods, ..
        } = &mut cache.commitment;

        // Compute query points.
        let points = queries_to_points(
            points.unchecked_slice_mut(queries.len()),
            queries,
            stark_domains,
        );

        // Evaluate the FRI input layer at query points.
        let eval_info = OodsEvaluationInfo {
            oods_values: &commitment.oods_values.as_slice().to_vec(),
            oods_point: &commitment.interaction_after_composition,
            trace_generator: &stark_domains.trace_generator,
            constraint_coefficients: &commitment.interaction_after_oods.as_slice().to_vec(),
        };
        let oods_poly_evals = eval_oods_boundary_poly_at_points::<Layout>(
            eval_oods,
            *n_original_columns,
            *n_interaction_columns,
            public_input,
            &eval_info,
            &points,
            &witness.traces_decommitment,
            &witness.composition_decommitment,
        );

        // Decommit FRI.
        let fri_decommitment = types::DecommitmentRef {
            values: oods_poly_evals,
            points,
        };
        fri_verify(
            &mut cache.fri,
            queries,
            &commitment.fri,
            &fri_decommitment,
            &mut witness.fri_witness,
        )
        .map_err(|_| ())?;

        Ok(vec![
            Tasks::TableDecommit(TableDecommitTarget::Original),
            Tasks::TableDecommit(TableDecommitTarget::Interaction),
            Tasks::TableDecommit(TableDecommitTarget::Composition),
        ])
    }
}

impl<'a> StarkVerifyTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        StarkVerifyTask {
            cache: &mut cache.legacy.stark,
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
