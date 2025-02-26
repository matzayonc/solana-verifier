use swiftness::funvec::FUNVEC_QUERIES;
use swiftness::funvec::FunVec;
use swiftness::queries::generate_queries;
use swiftness::types::Felt;
use swiftness::types::StarkProof;
use swiftness_air::Transcript;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

pub struct GenerateQueriesTask<'a> {
    queries: &'a mut FunVec<Felt, FUNVEC_QUERIES>,
    transcript: &'a mut Transcript,
    n_samples: Felt,
    query_upper_bound: Felt,
}

impl Task for GenerateQueriesTask<'_> {
    // generate_queries()
    fn execute(&mut self) {
        let GenerateQueriesTask {
            queries,
            transcript,
            n_samples,
            query_upper_bound,
        } = self;

        queries.move_to(generate_queries(transcript, *n_samples, *query_upper_bound));
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> GenerateQueriesTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        _cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        GenerateQueriesTask {
            queries: &mut intermediate.verify.queries,
            transcript: &mut intermediate.verify.transcript,
            n_samples: proof.config.n_queries,
            query_upper_bound: intermediate.verify.stark_domains.eval_domain_size,
        }
    }
}
