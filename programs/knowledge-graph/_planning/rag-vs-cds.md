# RAG vs CDS — Why This Is Not a RAG System

## Short Answer

You're describing a **Cognitive Document System (CDS)**.
RAG is a subset of what you're building. The vector layer is RAG-like.
Everything else — the tool-use navigation, the embedded prompts, the attention trace,
the self-orientation — goes beyond RAG and into cognitive architecture territory.

---

## The RAG Model

```
Human query
    ↓
Embed query as vector
    ↓
Find similar documents (cosine similarity)
    ↓
Inject retrieved documents as context
    ↓
LLM generates response
    ↓
End
```

Key properties:
- **Passive** — documents sit and wait to be retrieved
- **Query-driven** — retrieval is triggered by an external query
- **Stateless** — no memory of what was retrieved before
- **Documents are inert** — they have no instructions, no self-knowledge

---

## The CDS Model (What You're Building)

```
AI decides to read a document
    ↓
AI calls file-selector tool with file number
    ↓
file-selector returns content + appends to ticker
    ↓
AI reads embedded prompt in the document
    ↓
AI reads the document's neighbors (via file-selector calls)
    ↓
context-builder writes ctx-NNNN.md (what this doc is, how it relates to neighbors)
    ↓
AI continues navigating based on what it found
    ↓
(loops until exploration goal is met)
```

Key properties:
- **Active** — the AI chooses what to read based on goal and prior reads
- **Navigation-driven** — movement through the graph is intentional, not retrieval
- **Stateful** — the ticker logs every read; the AI knows where it's been
- **Documents are active** — each doc has an embedded prompt that fires on read

---

## The Layers

### Layer 1 — Storage (shared with RAG)
Files in `Data/` with 5D vectors. Proximity in 5D space = semantic relatedness.
This is the RAG layer. The indexer computes vectors; file-selector can do proximity queries.

### Layer 2 — Navigation (beyond RAG)
The AI uses `file-selector` as a tool — an explicit function call.
It doesn't receive documents automatically. It chooses which document to read next.
This is **agentic navigation**, not retrieval.

### Layer 3 — Active Documents (far beyond RAG)
Each document has an embedded prompt. When the AI reads it, the prompt says:
"Look at my neighbors. Describe how I relate to them. Write ctx-NNNN.md."
Documents are not passive data. They are **instructions that execute on read**.

### Layer 4 — Attention Trace (no equivalent in RAG)
The global ticker logs every read: timestamp, file, session, reason.
Over time, this becomes a **map of what the AI has paid attention to**.
High-frequency files = central nodes. Navigation paths = implicit edges.
The ticker IS the graph's edge weights, emerging from use.

### Layer 5 — Self-Organization (no equivalent in RAG)
Documents update their own context files (ctx-NNNN.md) based on neighbor reads.
The graph describes itself. No human writes the relationships — they emerge from
the AI running embedded prompts and writing context files.

---

## What This Is Closest To

| System | How CDS is similar |
|--------|-------------------|
| **MemGPT** | External memory accessed via tool-use; AI chooses what to load |
| **GraphRAG** (Microsoft) | Documents connected as graph nodes, not just similar documents |
| **Cognitive architectures** | Active memory management, attention trace, self-describing nodes |
| **Zettelkasten** | Every note links to others; notes describe their relationships |

The best single-sentence description:
**"A Zettelkasten where notes are active agents, the AI navigates by tool-use, and access patterns emerge as graph structure."**

---

## What Makes This System Uniquely Powerful

1. **5D vectors are interpretable.** You can look at a file's `[0.8, 0.2, 0.1, 0.9, 0.9]`
   and understand it without running a query: specific, intuitive, foundational, central, verified.

2. **The ticker is an emergent knowledge graph.** You don't have to define relationships
   between documents. Relationships emerge from navigation patterns.
   Documents the AI visits together in a session are implicitly related.

3. **Documents teach the AI what to do.** The embedded prompt is a micro-agent inside
   each document. Different documents can have different prompts — some say "expand outward
   (read all neighbors)", some say "go deeper (read my children)", some say "go up (read my parent)".

4. **The system grows without restructuring.** Add a new file with an empty vector.
   The indexer positions it. Context-builder orients it. No schema changes.
   The file slot right into the graph.

---

## The Question You Should Ask Instead of "Is This RAG?"

**"Is this system capable of building its own understanding of itself?"**

Answer: Yes. Here's the loop:
```
File is created (vector = null, neighbors = [])
    ↓
Indexer assigns vector and finds k nearest neighbors
    ↓
AI reads file via file-selector
    ↓
Embedded prompt fires: read neighbors, write ctx-NNNN.md
    ↓
ctx-NNNN.md now describes this document's place in the graph
    ↓
Next AI reads both file and ctx-NNNN.md
    ↓
AI has richer context without any human annotation
```

**The system writes its own documentation by being used.**
