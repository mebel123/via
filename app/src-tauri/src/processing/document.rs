use crate::agents::agent::Agent;
use crate::agents::context_relation::ContextRelationAgent;
use crate::agents::person_relation::PersonRelationAgent;
use crate::pipeline::context::RecordContext;
use crate::pipeline::entities::EntityExtractionStep;
use crate::pipeline::pipeline::Pipeline;
use crate::pipeline::transcription::TranscriptionStep;

pub async fn process_document(ctx: &RecordContext) -> Result<(), String> {
    let openai_api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();

    ctx.emit("transcription", "Transkription gestartet", 5);

    Pipeline::new()
        .add_step(TranscriptionStep {
            openai_api_key: openai_api_key.clone(),
        })
        .run(ctx)
        .await?;

    ctx.emit("entities", "Entities extrahiert", 30);

    Pipeline::new()
        .add_step(EntityExtractionStep {
            openai_api_key: openai_api_key.clone(),
        })
        .run(ctx)
        .await?;

    ctx.emit("relations", "Personenrelationen analysiert", 55);

    PersonRelationAgent {
        openai_api_key: openai_api_key.clone(),
    }
        .run_document(ctx)
        .await
        .map_err(|e| e.to_string())?;

    ctx.emit("relations", "Kontextrelationen analysiert", 75);

    ContextRelationAgent {
        openai_api_key,
    }
        .run_document(ctx)
        .await
        .map_err(|e| e.to_string())?;

    ctx.emit("done", "Verarbeitung abgeschlossen", 100);

    Ok(())
}
