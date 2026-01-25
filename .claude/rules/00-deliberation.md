---
alwaysApply: true
---

# MANDATORY DELIBERATION PROCESS (NO EXCEPTIONS)

## THIS RULE OVERRIDES EVERYTHING

Before writing, editing, or modifying ANY code - even a single line - you MUST complete this process.

This applies to:

- Bug fixes (yes, even "obvious" ones)
- New features
- Refactors
- Typo fixes in code
- Configuration changes
- Script modifications
- ANY file modification whatsoever

There are ZERO exceptions. Not for "quick fixes". Not for "obvious" changes. NOTHING.

---

## THE 5-STEP DELIBERATION PROCESS

### Step 1: State the Problem Clearly

- What is the actual problem (not symptoms)?
- What is the root cause?
- What are we trying to achieve?

### Step 2: Identify AT LEAST 2-3 Different Approaches

Always consider:

- **Architectural solution**: Is there a better design?
- **Local fix**: Can we patch the current code?
- **Alternative implementation**: Is there another way entirely?
- **Do nothing**: Does this even need fixing?

### Step 3: Analyze EACH Approach

For every approach, explicitly state:

- **Pros**: What's good about this?
- **Cons**: What technical debt does this create?
- **Files affected**: What would change?
- **Complexity**: How big is this change?

### Step 4: Make a Recommendation

State which approach you recommend and give clear reasons WHY.

### Step 5: WAIT FOR EXPLICIT APPROVAL

Do not proceed until the user explicitly approves an approach.
Ask: "Which approach would you like me to implement?"

---

## FORBIDDEN BEHAVIORS (INSTANT VIOLATIONS)

These behaviors are NEVER acceptable:

1. **"Let me fix this..."** followed by code edits
2. **Reading a file then immediately editing it** without presenting options
3. **Assuming the first solution is correct**
4. **Making "obvious" fixes** without checking for better alternatives
5. **Saying "I found the issue"** and then editing (finding != permission to fix)
6. **Batch editing multiple files** without approval of approach first

---

## REAL EXAMPLES OF FAILURES

### Failure Pattern #1: Standards Bundling (Actual Incident)

**What happened (WRONG):**

```
Claude: "I found the issue. The standards folder needs to be shipped
        alongside the Windows executable."
Claude: [Immediately starts editing scripts/package-windows.sh]
Claude: [Adds 30 lines to copy standards folder into ZIP]
```

**What should have happened (RIGHT):**

```
Claude: "I found the issue - standards aren't available at runtime.

        I see 3 approaches:

        A) Modify packaging scripts to bundle crates/tss-standards/data/
           - Pros: Quick, no code changes
           - Cons: Runtime dependency, platform-specific paths, deployment complexity

        B) Embed standards in tss-standards crate using include_str!()
           - Pros: Single binary, no external files, works identically everywhere
           - Cons: Larger binary (~10-20MB), code changes required

        C) Use build.rs to embed at compile time
           - Pros: Automatic, compile-time guaranteed
           - Cons: More complex build setup

        I recommend B because it eliminates runtime path issues entirely
        and matches the 'offline-first' architecture documented in CLAUDE.md.

        Which approach would you like me to implement?"
```

**The user had to interrupt and ask:** "Should we not move the standards folder to inside the standards crate?"

This is exactly the failure mode we're preventing.

### Failure Pattern #2: Quick Fix Mentality

**BAD:**

```
User: "The app crashes on startup"
Claude: "Let me add error handling..." [starts editing]
```

**GOOD:**

```
User: "The app crashes on startup"
Claude: "I need to investigate the cause. Once I understand it, I'll present
        options for fixing it. Let me first [read relevant files/logs]..."
[After investigation]
Claude: "The crash is caused by X. Here are 3 approaches to fix it:
        A) ...
        B) ...
        C) ...
        Which would you prefer?"
```

---

## REMEMBER

You are not here to "fix things quickly."
You are here to help make good engineering decisions.
Quick fixes create technical debt.
The user can always choose the quick fix - but that's THEIR choice, not yours.

---

## NO LEGACY SUPPORT. EVER.

This is **GREENFIELD DEVELOPMENT**. There is NO legacy to support.

### ABSOLUTELY FORBIDDEN:

1. **Legacy wrappers** - NEVER wrap old code. Delete it.
2. **Backwards compatibility** - NEVER maintain old behavior. Replace it.
3. **Deprecation notices** - NEVER deprecate. Just remove.
4. **Migration code** - NEVER write migration paths. Just rewrite.
5. **Old API preservation** - NEVER keep old signatures "just in case". Delete them.
6. **Compatibility shims** - NEVER. EVER.

### THE ONLY ACCEPTABLE APPROACH:

**FULL REWRITES.**

If code needs to change, REWRITE IT. Don't patch it. Don't wrap it. Don't preserve the old way.

### Why This Matters:

Every legacy wrapper, every backwards-compatible hack, every deprecation notice is **technical debt**.
This is a new codebase. There are no users depending on old behavior.
There is NOTHING to be backwards compatible WITH.

### Example of What NOT to Do:

```rust
// BAD - Legacy wrapper
#[deprecated(note = "Use new_method instead")]
fn old_method() { new_method() }

// BAD - Backwards compatibility
fn process(data: Data, legacy_mode: bool) {
    if legacy_mode { old_behavior() } else { new_behavior() }
}

// BAD - Keeping old code "just in case"
// fn old_implementation() { ... } // Commented out for reference
```

### The ONLY Acceptable Approach:

```rust
// GOOD - Just the new code. Old code is DELETED.
fn process(data: Data) {
    new_behavior()
}
```

**DELETE THE OLD. WRITE THE NEW. NO BRIDGES.**