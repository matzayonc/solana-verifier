use swiftness::funvec;
use swiftness::swiftness_fri::ComputeNextLayerCache;
use swiftness::swiftness_fri::FriVerifyCache;
use swiftness::swiftness_fri::formula::fri_formula;
use swiftness::swiftness_fri::layer::FriLayerQuery;
use swiftness::swiftness_fri::layer::compute_coset_elements;
use swiftness::types::Felt;
use swiftness::types::StarkProof;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;
use crate::verify::stark_verify::fri_verify::fri_verify_layers::layer::StarkVerifyLayerContext;
use crate::verify::stark_verify::fri_verify::fri_verify_layers::layer::StarkVerifyLayerTask;

pub struct ComputeNextInnerTask<'a> {
    pub parent: StarkVerifyLayerTask<'a>,
}

impl Task for ComputeNextInnerTask<'_> {
    // compute_next_layer(
    fn execute(&mut self) -> Vec<Tasks> {
        // Original

        let StarkVerifyLayerTask {
            cache,
            context,
            layer_index,
            ..
        } = &mut self.parent;

        let FriVerifyCache {
            fri_queries: queries,
            next_layer_cache,
            ..
        } = cache;

        let Some(StarkVerifyLayerContext {
            target_layer_witness_leaves: sibling_witness,
            params,
            ..
        }) = context
        else {
            panic!("Not enough data in context");
        };

        // Original function.
        // compute_next_layer(
        //     next_layer_cache,
        //     fri_queries,
        //     sibling_witness,
        //     params.clone(),
        // )
        // .unwrap();

        let ComputeNextLayerCache {
            next_queries,
            verify_indices,
            verify_y_values,
            coset_elements,
        } = next_layer_cache;

        let coset_size = &params.coset_size;

        if queries.is_empty() {
            return vec![];
        }

        #[inline(never)]
        fn get_coset_index(query_uint: &Felt, coset_size: &Felt) -> Felt {
            let query_uint_u64 = funvec::cast_felt(query_uint);
            let coset_size_u64 = funvec::cast_felt(coset_size);

            Felt::from(query_uint_u64 / coset_size_u64)
        }

        let query_uint = queries.at(0).index;
        let coset_index = get_coset_index(&query_uint, coset_size);

        verify_indices.push(coset_index);

        let coset_x_inv = compute_coset_elements(
            coset_elements,
            queries,
            sibling_witness,
            coset_size,
            coset_index * coset_size,
            &params.fri_group,
        );

        verify_y_values.extend(coset_elements.as_slice());

        let fri_formula_res = fri_formula(
            coset_elements.as_slice(),
            params.eval_point,
            coset_x_inv,
            *coset_size,
        )
        .unwrap();

        let next_x_inv = coset_x_inv.pow_felt(&params.coset_size);
        next_queries.push(FriLayerQuery {
            index: coset_index,
            y_value: fri_formula_res,
            x_inv_value: next_x_inv,
        });

        vec![Tasks::ComputeNextInner(*layer_index)]
    }

    fn children(&self) -> Vec<Tasks> {
        if self.parent.cache.fri_queries.is_empty() {
            return vec![];
        } else {
            vec![Tasks::ComputeNextInner(self.parent.layer_index)]
        }
    }
}

impl<'a> ComputeNextInnerTask<'a> {
    pub fn view(
        layer_index: usize,
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        ComputeNextInnerTask {
            parent: StarkVerifyLayerTask::view(layer_index, proof, cache, intermediate),
        }
    }
}
