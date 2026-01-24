---
paths:
  - "crates/tss-gui/**/*.rs"
---

# GUI Architecture Rules (tss-gui)

## MANDATORY: Deliberation First

Before ANY change to this crate:

1. State the problem
2. Present 2-3 approaches with pros/cons
3. Wait for explicit approval

This applies even to "obvious" GUI fixes. **No exceptions.**

**Example:** If a button doesn't work, don't just fix the handler.
Ask: "Is this the right UX? Should the button exist? Is the handler in the right place?"

---

## Elm Architecture - Non-Negotiable

- State changes ONLY happen in `update()` - views are pure functions
- NEVER modify state in view code
- Use `Task::perform` for async operations - no channels or polling
- Follow the handler pattern: each feature has a dedicated handler implementing `MessageHandler`

---

## Before Modifying

ASK before:

- Adding new message types to the main Message enum
- Creating new handlers
- Changing state structure in AppState or ViewState
- Modifying the theme system

---

## Patterns to Follow

- Look at existing handlers in `handler/` for patterns
- Check `component/` for reusable UI components
- Use services in `service/` for background tasks

---

## Common Mistakes to Avoid

1. **Putting logic in views** - Views must be pure. Move logic to handlers.
2. **Creating new state when existing state works** - Check ViewState first.
3. **Not using Task::perform for async** - Never use channels or polling.
4. **Ignoring existing patterns** - Always check how similar features are implemented.