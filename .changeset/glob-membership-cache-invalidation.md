---
"@farmfe/core": patch
---

Revalidate import.meta.glob membership when reusing persistent caches and rebuild glob importers when matching files are added or removed during development. Deliver those changes over the dev server WebSocket in compilation order and retain failed updates for retry.
