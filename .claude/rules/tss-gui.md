---
paths:
  - "crates/tss-gui/**/*.rs"
---

# GUI Architecture Rules (tss-gui)

## Elm Architecture - Non-Negotiable

- State changes ONLY happen in `update()` - views are pure functions
- NEVER modify state in view code
- Use `Task::perform` for async operations - no channels or polling
- Follow the handler pattern: each feature has a dedicated handler implementing `MessageHandler`

## Before Modifying

ASK before:
- Adding new message types to the main Message enum
- Creating new handlers
- Changing state structure in AppState or ViewState
- Modifying the theme system

## Patterns to Follow

- Look at existing handlers in `handler/` for patterns
- Check `component/` for reusable UI components
- Use services in `service/` for background tasks