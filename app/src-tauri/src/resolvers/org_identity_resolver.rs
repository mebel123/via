use anyhow::Result;
use std::time::Instant;

use crate::resolvers::context::ResolverContext;
use crate::resolvers::resolver::Resolver;
use crate::store::knowledge::KnowledgeStore;

pub struct OrgIdentityResolver;

#[async_trait::async_trait]
impl Resolver for OrgIdentityResolver {
    fn name(&self) -> &'static str {
        "ORG_IDENTITY_RESOLVER"
    }

    async fn run(&self, ctx: &ResolverContext) -> Result<()> {
        println!("▶ ResolverRunner: start {}", self.name());
        let started = Instant::now();
        let mut knowledge = KnowledgeStore::load_or_create(&ctx.data_root).await?;

        // 2. apply resolver logic
        Self::resolve(&mut knowledge)?;

        // 3. persist
        knowledge.save().await?;

        println!(
            "▶ ResolverRunner: finished {} ({} ms)",
            self.name(),
            started.elapsed().as_millis()
        );
        Ok(())
    }
}
fn normalize_org_name(s: &str) -> String {
    s.to_lowercase()
        .replace('.', "")
        .replace(',', "")
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
impl OrgIdentityResolver {
    fn resolve(knowledge: &mut KnowledgeStore) -> Result<()> {
        use crate::store::knowledge::OrganizationCluster;
        use chrono::Utc;
        use std::collections::{HashMap, HashSet};

        // normalized_name -> original variants
        let mut org_variants: HashMap<String, HashSet<String>> = HashMap::new();

        for record in knowledge.all() {
            // subject kann organization sein
            if record.subject_type == "organization" {
                let norm = normalize_org_name(&record.subject_value);
                org_variants
                    .entry(norm)
                    .or_default()
                    .insert(record.subject_value.clone());
            }

            // object kann organization sein (z. B. person -> organization)
            if record.predicate == "associated_with" {
                let norm = normalize_org_name(&record.object_value);
                org_variants
                    .entry(norm)
                    .or_default()
                    .insert(record.object_value.clone());
            }
        }

        println!(
            "▶ OrgIdentityResolver: collected {} normalized organization groups",
            org_variants.len()
        );

        // v1 Strategie: komplette Neuberechnung
        knowledge.clusters_mut().organizations.clear();

        let now = Utc::now().to_rfc3339();
        let mut created = 0;

        for (normalized, variants) in org_variants {
            if variants.len() < 2 {
                continue;
            }

            let mut variants_vec: Vec<String> = variants.into_iter().collect();
            variants_vec.sort();

            let cluster = OrganizationCluster {
                cluster_id: format!("org:{}", normalized),
                normalized,
                variants: variants_vec,
                confidence: 0.9, // heuristisch für v1
                status: "candidate".into(),
                source_agent: "ORG_IDENTITY_RESOLVER".into(),
                created_at: now.clone(),
                updated_at: None,
            };

            knowledge.clusters_mut().organizations.push(cluster);

            created += 1;
        }

        println!(
            "▶ OrgIdentityResolver: wrote {} organization clusters",
            created
        );

        Ok(())
    }
}
