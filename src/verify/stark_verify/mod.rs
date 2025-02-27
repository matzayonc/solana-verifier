use swiftness::funvec::FunVec;
use swiftness::oods::OodsEvaluationInfo;
use swiftness::oods::eval_oods_boundary_poly_at_points;
use swiftness::queries::queries_to_points;
use swiftness::stark::CacheCommitment;
use swiftness::types::CacheStark;
use swiftness::types::Felt;
use swiftness::types::StarkCommitment;
use swiftness::types::StarkProof;
use swiftness::types::StarkWitness;
use swiftness_air::domains::StarkDomains;
use swiftness_air::layout::recursive_with_poseidon::Layout;
use swiftness_air::public_memory::PublicInput;
use table_decommit::TableDecommitTarget;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

pub mod fri_verify;
pub mod table_decommit;

pub struct StarkVerifyTask<'a> {
    pub cache: &'a mut CacheStark,
    pub n_original_columns: u32,
    pub n_interaction_columns: u32,
    pub public_input: &'a PublicInput,
    pub queries: &'a [Felt],
    pub commitment: &'a StarkCommitment,
    pub witness: &'a mut StarkWitness,
    pub stark_domains: &'a StarkDomains,
    pub intermediate: &'a mut StarkVerifyIntermediate,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StarkVerifyIntermediate {
    pub points: FunVec<Felt, 256>,
    pub evaluations: FunVec<Felt, 256>,
}

impl Task for StarkVerifyTask<'_> {
    // stark_verify::<Layout>(
    fn execute(&mut self) {
        let StarkVerifyTask {
            cache,
            n_original_columns,
            n_interaction_columns,
            public_input,
            queries,
            commitment,
            witness,
            stark_domains,
            intermediate,
        } = self;

        let CacheCommitment { eval_oods, .. } = &mut cache.commitment;
        let StarkVerifyIntermediate {
            points,
            evaluations,
        } = intermediate;
        let points = points.to_size_uninitialized(queries.len());

        // Compute query points.
        let points = queries_to_points(points, queries, stark_domains);

        // Evaluate the FRI input layer at query points.
        let eval_info = OodsEvaluationInfo {
            oods_values: commitment.oods_values.as_slice(),
            oods_point: &commitment.interaction_after_composition,
            trace_generator: &stark_domains.trace_generator,
            constraint_coefficients: commitment.interaction_after_oods.as_slice(),
        };
        let evaluations = evaluations.to_size_uninitialized(points.len());
        let _oods_poly_evals = eval_oods_boundary_poly_at_points::<Layout>(
            eval_oods,
            evaluations,
            *n_original_columns,
            *n_interaction_columns,
            public_input,
            &eval_info,
            points,
            &witness.traces_decommitment,
            &witness.composition_decommitment,
        );
    }

    fn children(&self) -> Vec<Tasks> {
        vec![
            Tasks::TableDecommit(TableDecommitTarget::Original),
            Tasks::TableDecommit(TableDecommitTarget::Interaction),
            Tasks::TableDecommit(TableDecommitTarget::Composition),
            Tasks::StarkVerifyFri,
        ]
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
            queries: intermediate.verify.queries.as_slice(),
            commitment: &intermediate.verify.stark_commitment,
            witness: &mut proof.witness,
            stark_domains: &intermediate.verify.stark_domains,
            intermediate: &mut intermediate.stark_verify,
        }
    }
}
