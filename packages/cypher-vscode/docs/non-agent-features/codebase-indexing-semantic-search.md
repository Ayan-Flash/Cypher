# Codebase Indexing & Semantic Search

**Priority:** P2

## Remaining Work

- Vector-based indexing with embeddings (local and/or cloud)
- Semantic search over the repository
- Incremental updates via file watchers and hashing
- Multiple embedding providers and storage backends
- Integration with existing CLI grep/glob for hybrid search

## Primary Implementation Anchors (cypher-legacy)

These exist in the [cypher-legacy](https://github.com/Cypher-Org/cypher-legacy) repo, not in this extension:

- `src/services/code-index/`
