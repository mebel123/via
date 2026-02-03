use crate::pipeline::context::RecordContext;
use crate::pipeline::evidences::EvidencesPipeline;
use crate::pipeline::knowledge_builder::KnowledgeBuilder;
use crate::pipeline::pipeline::Pipeline;
use crate::pipeline::signals::SignalsPipeline;
use crate::resolvers::context::ResolverContext;
use crate::resolvers::OrgIdentityResolver;
use crate::resolvers::resolver::ResolverRunner;

pub async fn update_global_state(ctx: &RecordContext) -> Result<(), String> {
    Pipeline::new()
        .add_step(SignalsPipeline)
        .run(ctx)
        .await?;

    Pipeline::new()
        .add_step(EvidencesPipeline)
        .run(ctx)
        .await?;

    Pipeline::new()
        .add_step(KnowledgeBuilder)
        .run(ctx)
        .await?;


    let data_root = ctx
        .base_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("invalid data root")?
        .to_path_buf();

    let resolver_ctx = ResolverContext { data_root };

    ResolverRunner::run_all(
        &resolver_ctx,
        &[
            Box::new(OrgIdentityResolver),
            // später:
            // Box::new(PersonIdentityResolver),
        ],
    )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}