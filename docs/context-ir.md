# Context IR

The versioned protocol models blocks with kind, source, content, preserve mode, scoring dimensions, token estimate, lifecycle and status. Kinds include `objective`, `constraint`, `acceptance`, `code`, `file_reference`, `error`, `tool_result`, `decision`, `fact`, `completed_work`, `next_action`, `project_map` and `conversation`.

`exact` and `semantic` blocks bypass ordinary pruning. `compressible` and `droppable` blocks are budgeted. Uncertainty keeps content. v0.1 estimates use a conservative byte heuristic and explicitly set `exact=false`; every decision can appear in a machine-readable trace.
