---
"cypher-cli": patch
---

Fix performance-degrading recursive directory search command loops (like `ls -R` / `dir /s`) and optimize initial chat greeting responses to bypass LLM and tool loops.
