# CDISC Transpiler Skills

This directory contains custom Claude Code skills tailored for the CDISC Transpiler project.

## Available Skills

### 1. `validate-sdtm`
**Purpose**: Run SDTM validation checks and analyze conformance issues

**Use when**:
- Validating converted SDTM datasets
- Debugging validation errors or warnings
- Checking controlled terminology conformance
- Analyzing why validation is blocking XPT output

**Example**: "Why is validation failing for the DM domain?" or "Check the CT validation errors"

---

### 2. `check-standards`
**Purpose**: Look up SDTM and controlled terminology standards from offline CSV files

**Use when**:
- Looking up SDTM domain specifications
- Finding variable definitions and their CT codelists
- Verifying controlled terminology values
- Checking SDTM Implementation Guide requirements

**Example**: "What are the valid values for the SEX variable?" or "Show me the DM domain specification"

---

### 3. `domain-processor`
**Purpose**: Create or modify SDTM domain-specific processors

**Use when**:
- Creating a new domain processor
- Adding controlled terminology normalization
- Implementing USUBJID prefixing logic
- Adding --SEQ column generation
- Fixing domain-specific validation issues

**Example**: "Help me create an AE domain processor" or "Add CT normalization for RACE in DM"

---

### 4. `release-check`
**Purpose**: Run pre-release validation checklist before merging or releasing

**Use when**:
- Before creating a pull request
- Before merging to main branch
- Before cutting a release
- After significant refactoring

**Example**: "Run the release checklist" or "Verify code is ready for PR"

---

## How Skills Work

Skills are **automatically activated** by Claude when your request matches the skill's description. You don't need to invoke them explicitly - just ask naturally.

### Example Workflow

```
You: "I'm getting validation errors in the VS domain. Can you help debug them?"

Claude: [Automatically uses validate-sdtm skill]
        [Analyzes validation output]
        [Provides specific debugging steps]
```

## Activation

To activate these skills:

1. **Exit Claude Code** (if running)
2. **Restart Claude Code** to load the new skills
3. **Ask naturally** - Claude will use skills when relevant

## Checking Available Skills

To see all available skills:
```
What skills are available?
```

## File Structure

```
.claude/skills/
├── README.md                           # This file
├── validate-sdtm/
│   └── SKILL.md                        # Validation workflow skill
├── check-standards/
│   └── SKILL.md                        # Standards lookup skill
├── domain-processor/
│   └── SKILL.md                        # Domain development skill
└── release-check/
    └── SKILL.md                        # Release checklist skill
```

## Customization

You can customize these skills by editing the `SKILL.md` files:

- **Update description** - Change when the skill activates
- **Add commands** - Include project-specific commands
- **Add examples** - Provide domain-specific examples
- **Add references** - Link to additional documentation

## Best Practices

1. **Be specific** - The more specific your description, the better Claude knows when to use it
2. **Include examples** - Help Claude understand typical use cases
3. **Keep focused** - Each skill should have one clear purpose
4. **Test activation** - Try asking questions that should trigger the skill

## Contributing

When adding new skills:

1. Create a new directory under `.claude/skills/`
2. Add a `SKILL.md` file with YAML frontmatter
3. Include clear description and instructions
4. Test by restarting Claude Code
5. Commit to repository to share with team

## Version Control

These skills are committed to the repository so the entire team benefits. When you improve a skill, commit your changes:

```bash
git add .claude/skills/
git commit -m "docs: improve validate-sdtm skill with CT examples"
```

## Related Documentation

- Claude Code Skills Guide: Check `/help` in Claude Code
- Project Architecture: See `CLAUDE.md` in repository root
- Naming Conventions: See `docs/NAMING_CONVENTIONS.md`
- SDTM Standards: See `standards/` directory
