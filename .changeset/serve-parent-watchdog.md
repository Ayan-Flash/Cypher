---
"@cypher/cli": patch
"cypher-cli": patch
---

Shut down the headless `cypher serve` process automatically when the editor client that launched it exits without a clean signal, preventing orphaned CLI processes.
