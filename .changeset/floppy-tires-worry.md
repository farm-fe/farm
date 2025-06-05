---
"@farmfe/core": patch
---

Added origin validation to HMR server

BREAKING CHANGE: The HMR server now rejects all connections with unrecognized `Origin` headers. Clients need to update their configured ports and hosts if they want external apps to be able to connect to the HMR server.
