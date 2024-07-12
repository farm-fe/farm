---
"create-farm-plugin": patch
---

Add plugins hooks:
* process_genereated_resources: process genererated resources after transform file name and before adding the resources to resources_map
* handle_entry_resource: handle entry resources after all resources are generated and before finalize resources
