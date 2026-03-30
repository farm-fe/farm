---
name: self-improving-agent
description: A universal self-improving agent that learns from ALL skill experiences. Uses multi-memory architecture (semantic + episodic + working) to continuously evolve the codebase. Auto-triggers on skill completion/error with hooks-based self-correction.
allowed-tools: Read, Write, Edit, Bash, Grep, Glob, WebSearch
metadata:
  hooks:
    before_start:
      - trigger: session-logger
        mode: auto
        context: "Start {skill_name}"
    after_complete:
      - trigger: create-pr
        mode: ask_first
        condition: skills_modified
        reason: "Submit improvements to repository"
      - trigger: session-logger
        mode: auto
        context: "Self-improvement cycle complete"
    # Note: on_error intentionally only logs to session to avoid infinite recursion
    # Self-correction is triggered by other skills (debugger, code-reviewer) completing their work
    on_error:
      - trigger: session-logger
        mode: auto
        context: "Error captured in {skill_name}"
---

# Self-Improving Agent

> "An AI agent that learns from every interaction, accumulating patterns and insights to continuously improve its own capabilities." — Based on 2025 lifelong learning research

## Overview

This is a **universal self-improvement system** that learns from ALL skill experiences, not just PRDs. It implements a complete feedback loop with:

- **Multi-Memory Architecture**: Semantic + Episodic + Working memory
- **Self-Correction**: Detects and fixes skill guidance errors
- **Self-Validation**: Periodically verifies skill accuracy
- **Hooks Integration**: Auto-triggers on skill events (before_start, after_complete, on_error)
- **Evolution Markers**: Traceable changes with source attribution

## Research-Based Design

Based on 2025 research:

| Research | Key Insight | Application |
|----------|-------------|-------------|
| [SimpleMem](https://arxiv.org/html/2601.02553v1) | Efficient lifelong memory | Pattern accumulation system |
| [Multi-Memory Survey](https://dl.acm.org/doi/10.1145/3748302) | Semantic + Episodic memory | World knowledge + experiences |
| [Lifelong Learning](https://arxiv.org/html/2501.07278v1) | Continuous task stream learning | Learn from every skill use |
| [Evo-Memory](https://shothota.medium.com/evo-memory-deepminds-new-benchmark) | Test-time lifelong learning | Real-time adaptation |

## The Self-Improvement Loop

```
┌─────────────────────────────────────────────────────────────────┐
│                    UNIVERSAL SELF-IMPROVEMENT                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Skill Event → Extract Experience → Abstract Pattern → Update  │
│        │                  │                │         │          │
│        ▼                  ▼                ▼         ▼          │
│   ┌─────────────────────────────────────────────────────┐       │
│   │              MULTI-MEMORY SYSTEM                      │       │
│   ├─────────────────────────────────────────────────────┤       │
│   │  Semantic Memory   │  Episodic Memory  │ Working Memory │  │
│   │  (Patterns/Rules)  │  (Experiences)    │  (Current)     │  │
│   │  memory/semantic/  │  memory/episodic/ │  memory/working/│  │
│   └─────────────────────────────────────────────────────┘       │
│                                                                 │
│   ┌─────────────────────────────────────────────────────┐       │
│   │              FEEDBACK LOOP                            │       │
│   │  User Feedback → Confidence Update → Pattern Adapt   │       │
│   └─────────────────────────────────────────────────────┘       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## When This Activates

### Automatic Triggers (via hooks)

| Event | Trigger | Action |
|-------|---------|--------|
| **before_start** | Any skill starts | Log session start |
| **after_complete** | Any skill completes | Extract patterns, update skills |
| **on_error** | Bash returns non-zero exit | Capture error context, trigger self-correction |

### Manual Triggers

- User says "自我进化", "self-improve", "从经验中学习"
- User says "分析今天的经验", "总结教训"
- User asks to improve a specific skill

## Evolution Priority Matrix

Trigger evolution when new reusable knowledge appears:

| Trigger | Target Skill | Priority | Action |
|---------|--------------|----------|--------|
| New PRD pattern discovered | prd-planner | High | Add to quality checklist |
| Architecture tradeoff clarified | architecting-solutions | High | Add to decision patterns |
| API design rule learned | api-designer | High | Update template |
| Debugging fix discovered | debugger | High | Add to anti-patterns |
| Review checklist gap | code-reviewer | High | Add checklist item |
| Perf/security insight | performance-engineer, security-auditor | High | Add to patterns |
| UI/UX spec issue | prd-planner, architecting-solutions | High | Add visual spec requirements |
| React/state pattern | debugger, refactoring-specialist | Medium | Add to patterns |
| Test strategy improvement | test-automator, qa-expert | Medium | Update approach |
| CI/deploy fix | deployment-engineer | Medium | Add to troubleshooting |

## Multi-Memory Architecture

### 1. Semantic Memory (`memory/semantic-patterns.json`)

Stores **abstract patterns and rules** reusable across contexts:

```json
{
  "patterns": {
    "pattern_id": {
      "id": "pat-2025-01-11-001",
      "name": "Pattern Name",
      "source": "user_feedback|implementation_review|retrospective",
      "confidence": 0.95,
      "applications": 5,
      "created": "2025-01-11",
      "category": "prd_structure|react_patterns|async_patterns|...",
      "pattern": "One-line summary",
      "problem": "What problem does this solve?",
      "solution": { ... },
      "quality_rules": [ ... ],
      "target_skills": [ ... ]
    }
  }
}
```

### 2. Episodic Memory (`memory/episodic/`)

Stores **specific experiences and what happened**:

```
memory/episodic/
├── 2025/
│   ├── 2025-01-11-prd-creation.json
│   ├── 2025-01-11-debug-session.json
│   └── 2025-01-12-refactoring.json
```

```json
{
  "id": "ep-2025-01-11-001",
  "timestamp": "2025-01-11T10:30:00Z",
  "skill": "debugger",
  "situation": "User reported data not refreshing after form submission",
  "root_cause": "Empty callback in onRefresh prop",
  "solution": "Implement actual refresh logic in callback",
  "lesson": "Always verify callbacks are not empty functions",
  "related_pattern": "callback_verification",
  "user_feedback": {
    "rating": 8,
    "comments": "This was exactly the issue"
  }
}
```

### 3. Working Memory (`memory/working/`)

Stores **current session context**:

```
memory/working/
├── current_session.json   # Active session data
├── last_error.json        # Error context for self-correction
└── session_end.json       # Session end marker
```

## Self-Improvement Process

### Phase 1: Experience Extraction

After any skill completes, extract:

```yaml
What happened:
  skill_used: {which skill}
  task: {what was being done}
  outcome: {success|partial|failure}

Key Insights:
  what_went_well: [what worked]
  what_went_wrong: [what didn't work]
  root_cause: {underlying issue if applicable}

User Feedback:
  rating: {1-10 if provided}
  comments: {specific feedback}
```

### Phase 2: Pattern Abstraction

Convert experiences to reusable patterns:

| Concrete Experience | Abstract Pattern | Target Skill |
|--------------------|------------------|--------------|
| "User forgot to save PRD notes" | "Always persist thinking to files" | prd-planner |
| "Code review missed SQL injection" | "Add security checklist item" | code-reviewer |
| "Callback was empty, didn't work" | "Verify callback implementations" | debugger |
| "Net APY position ambiguous" | "UI specs need exact relative positions" | prd-planner |

**Abstraction Rules:**

```yaml
If experience_repeats 3+ times:
  pattern_level: critical
  action: Add to skill's "Critical Mistakes" section

If solution_was_effective:
  pattern_level: best_practice
  action: Add to skill's "Best Practices" section

If user_rating >= 7:
  pattern_level: strength
  action: Reinforce this approach

If user_rating <= 4:
  pattern_level: weakness
  action: Add to "What to Avoid" section
```

### Phase 3: Skill Updates

Update the appropriate skill files with **evolution markers**:

```markdown
<!-- Evolution: 2025-01-12 | source: ep-2025-01-12-001 | skill: debugger -->

## Pattern Added (2025-01-12)

**Pattern**: Always verify callbacks are not empty functions

**Source**: Episode ep-2025-01-12-001

**Confidence**: 0.95

### Updated Checklist
- [ ] Verify all callbacks have implementations
- [ ] Test callback execution paths
```

**Correction Markers** (when fixing wrong guidance):

```markdown
<!-- Correction: 2025-01-12 | was: "Use callback chain" | reason: caused stale refresh -->

## Corrected Guidance

Use direct state monitoring instead of callback chains:
```typescript
// ✅ Do: Direct state monitoring
const prevPendingCount = usePrevious(pendingCount);
```
```

### Phase 4: Memory Consolidation

1. **Update semantic memory** (`memory/semantic-patterns.json`)
2. **Store episodic memory** (`memory/episodic/YYYY-MM-DD-{skill}.json`)
3. **Update pattern confidence** based on applications/feedback
4. **Prune outdated patterns** (low confidence, no recent applications)

## Self-Correction (on_error hook)

Triggered when:
- Bash command returns non-zero exit code
- Tests fail after following skill guidance
- User reports the guidance produced incorrect results

**Process:**

```markdown
## Self-Correction Workflow

1. Detect Error
   - Capture error context from working/last_error.json
   - Identify which skill guidance was followed

2. Verify Root Cause
   - Was the skill guidance incorrect?
   - Was the guidance misinterpreted?
   - Was the guidance incomplete?

3. Apply Correction
   - Update skill file with corrected guidance
   - Add correction marker with reason
   - Update related patterns in semantic memory

4. Validate Fix
   - Test the corrected guidance
   - Ask user to verify
```

**Example:**

```markdown
<!-- Correction: 2025-01-12 | was: "useMemo for claimable ids" | reason: stale data at click time -->

## Self-Correction: Click-Time Computation

**Issue**: Using useMemo for claimable IDs caused stale data
**Fix**: Compute at click time for always-fresh data
**Pattern**: click_time_vs_open_time_computation
```

## Self-Validation

Use the validation template in `references/appendix.md` when reviewing updates.

## Hooks Integration

### Wiring Hooks in Claude Code Settings

Add to Claude Code settings (`~/.claude/settings.json`):

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash|Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "bash ${SKILLS_DIR}/self-improving-agent/hooks/pre-tool.sh \"$TOOL_NAME\" \"$TOOL_INPUT\""
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "bash ${SKILLS_DIR}/self-improving-agent/hooks/post-bash.sh \"$TOOL_OUTPUT\" \"$EXIT_CODE\""
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "bash ${SKILLS_DIR}/self-improving-agent/hooks/session-end.sh"
          }
        ]
      }
    ]
  }
}
```

Replace `${SKILLS_DIR}` with your actual skills path.

## Additional References

See `references/appendix.md` for memory structure, workflow diagrams, metrics, feedback templates, and research links.

## Best Practices

### DO

- ✅ Learn from EVERY skill interaction
- ✅ Extract patterns at the right abstraction level
- ✅ Update multiple related skills
- ✅ Track confidence and apply counts
- ✅ Ask for user feedback on improvements
- ✅ Use evolution/correction markers for traceability
- ✅ Validate guidance before applying broadly

### DON'T

- ❌ Over-generalize from single experiences
- ❌ Update skills without confidence tracking
- ❌ Ignore negative feedback
- ❌ Make changes that break existing functionality
- ❌ Create contradictory patterns
- ❌ Update skills without understanding context

## Quick Start

After any skill completes, this agent automatically:

1. **Analyzes** what happened
2. **Extracts** patterns and insights
3. **Updates** relevant skill files
4. **Logs** to memory for future reference
5. **Reports** summary to user

## References

- [SimpleMem: Efficient Lifelong Memory for LLM Agents](https://arxiv.org/html/2601.02553v1)
- [A Survey on the Memory Mechanism of Large Language Model Agents](https://dl.acm.org/doi/10.1145/3748302)
- [Lifelong Learning of LLM based Agents](https://arxiv.org/html/2501.07278v1)
- [Evo-Memory: DeepMind's Benchmark](https://shothota.medium.com/evo-memory-deepminds-new-benchmark)
- [Let's Build a Self-Improving AI Agent](https://medium.com/@nomannayeem/lets-build-a-self-improving-ai-agent-that-learns-from-your-feedback-722d2ce9c2d9)
