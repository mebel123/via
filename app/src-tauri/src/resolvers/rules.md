# Resolver Rules

Resolvers are responsible for turning **evidence into knowledge**.

They operate under the following principles:

• Resolvers work **globally**  
They operate across documents, sessions, and time.  
They are not limited to a single document context.

• Resolvers **consume evidence**  
Their input is aggregated evidence from one or many agents.  
They never read raw documents directly.

• Resolvers **do not guess**  
They must be conservative.  
Every decision must be explainable based on evidence patterns.

• Resolvers **merge, normalize, and deprecate**  
They may:
- Merge multiple evidence records into one knowledge record
- Normalize names, identifiers, and roles
- Deprecate outdated or contradicted knowledge

• Resolvers manage **identity and consistency**  
They decide whether entities refer to the same real-world identity.  
They may create stable identity IDs and clusters.

• Resolvers may **trigger user interaction**  
If ambiguity cannot be resolved safely, they create tasks or questions  
instead of making assumptions.

• Resolvers are **stateful and persistent**  
Their output is durable knowledge stored in `knowledge.json`.  
Knowledge evolves over time as new evidence arrives.

In short:  
Resolvers decide what is *likely true*.  
They never invent facts, only confirm or reject hypotheses.
