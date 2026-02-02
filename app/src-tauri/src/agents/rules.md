Hier ist eine passende rules.md für den Ordner agents, klar und bewusst normativ formuliert:

# Agent Rules

Agents are responsible for generating **new evidence** from individual documents.

They operate under the following principles:

• Agents work **document-scoped**  
Each agent processes exactly one document at a time.  
No global state is assumed.

• Agents **produce evidence**  
Their output is evidence records, not final knowledge.  
Evidence may later be accepted, merged, downgraded, or rejected.

• Agents are allowed to **guess**  
Agents may infer, hypothesize, or propose relations based on context.  
All uncertainty must be expressed via confidence values.

• Agents are **short-lived and context-bound**  
They do not persist state across runs.  
They rely solely on the current document and its extracted signals.

• Agents do **not resolve identity or truth**  
They never decide whether two entities are the same.  
They never normalize names globally.  
This responsibility belongs to resolvers.

In short:  
Agents generate hypotheses.  
They do not decide what is true.

Wenn du möchtest, schreiben wir direkt daneben ein resolvers/rules.md, das exakt das Gegenstück definiert.
