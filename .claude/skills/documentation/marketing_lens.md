# Documentation Skill: Marketing Lens

## Purpose

Enhance technical documentation to appeal to executives and decision-makers without changing tone, losing precision, or alienating the primary developer audience.

## Core Philosophy

**Every technical fact can be framed with its business consequence.**

The strategy is not to add marketing language, but to make the business implications of technical choices explicit. Developers understand these implications intuitively; executives need them stated.

```
Before: "The system normalizes data into a consistent shape"
After:  "The system normalizes data into a consistent shape so
         you can ask one question across all systems"
```

---

## What NOT To Do

| Anti-pattern | Why It Fails |
|--------------|--------------|
| Add "For Executives" callout boxes | Breaks tone, signals "skip this" to devs |
| Add marketing superlatives | Loses credibility with both audiences |
| Add unsubstantiated claims | Violates precision |
| Remove technical depth | Alienates primary audience |
| Create separate "executive" docs | Maintenance burden, content drift |
| "Get this wrong" consequence framing | Negative tone, creates anxiety |
| Time/effort/cost estimates | Speculation, sets wrong expectations |
| Buzzwords ("enterprise-ready", "scalable") | Empty calories, no information |
| Vague compliance claims | Must name specific standards |

---

## Techniques

### Technique 1: Persona Callouts

Name who cares about a capability when introducing it.

**Pattern:**
```
[Capability]. [Who benefits and how].
```

**Example:**
```
Before:
"The integration uses YAML configuration instead of custom code."

After:
"The integration uses YAML configuration instead of custom code.
This means your ops or DBA teams can own it directly—no
engineering queue."
```

---

### Technique 2: Compliance Framing

Reframe "best practices" as compliance controls where true. Be specific.

**Important distinctions:**
- **Compliance requirement** - What a standard mandates (e.g., "SOC 2 requires access logging")
- **Compliance control** - How you implement the requirement (e.g., "audit logs with 90-day retention")

**Rule:** When referencing compliance, name the specific standard(s): SOC 2, ISO 27001, HIPAA, PCI-DSS, GDPR, FedRAMP, etc. Vague "compliance" claims have no information content.

**Pattern:**
```
[Practice]—required for [specific standard].
```

**Examples:**
```
Before:
"Never commit credentials to source control."

After:
"Never commit credentials to source control—this is a SOC 2
requirement (CC6.1: logical access controls)."

Before:
"Log all access attempts."

After:
"Log all access attempts—required for SOC 2 (CC7.2) and
ISO 27001 (A.12.4.1)."
```

---

### Technique 3: Operational Context

Add operational implications to mode/choice comparisons.

**Example:**
```
Before:
"One-shot mode: Run, perform a task, and exit."
"Daemon mode: Run continuously and process tasks..."

After:
"One-shot mode: Run, perform a task, and exit. Simple to test,
simple to debug, no persistent infrastructure."
"Daemon mode: Run continuously and process tasks... Enables
real-time updates and enforcement."
```

---

### Technique 4: Capability Expansion

State what capabilities enable for the organization.

**Example:**
```
Before:
"The system produces an access graph with three main node/edge types:"

After:
"This graph powers access reviews, certification campaigns,
provisioning workflows, and compliance reporting.
The system produces an access graph..."
```

---

### Technique 5: Scope Illustration

Show the range of what's possible by listing concrete use cases.

**Example:**
```
Before:
"## REST API Integration"

After:
"## REST API Integration

Use this for any REST API: internal microservices, SaaS platforms
without pre-built integrations, or legacy systems with HTTP endpoints."
```

---

## Validation Checklist

Apply to each enhancement:

- [ ] **Tone preserved?** Still reads like technical documentation
- [ ] **Precision maintained?** No superlatives, no unsubstantiated claims
- [ ] **Developer value intact?** Primary audience still gets what they need
- [ ] **Executive visibility added?** Business implication is now explicit
- [ ] **Strictly additive?** No content removed
- [ ] **No buzzwords?** Specific capabilities, not vague adjectives
- [ ] **Compliance specific?** Standards named, not generic "compliance"

---

## What This Enables

After applying the marketing lens, docs:

1. **Support "send to my team" workflow** - Executive can forward to engineers
2. **Answer "why this matters" implicitly** - Business implications stated
3. **Signal organizational scope** - Specific compliance controls, multi-team use cases
4. **Maintain developer primacy** - Technical precision unchanged
5. **Name stakeholders** - Executives see who benefits

---

## Limitations

This technique will NOT:
- Add competitive positioning
- Add case studies
- Add compliance matrices
- Add time/cost estimates

It maximizes existing content's dual-audience value within precision constraints.

---

## Integration with Other Skills

- **dry_witted_engineering.md** - Marketing lens is additive to dry-witted, not a replacement. Technical precision comes first; business framing wraps it.
- **tone_matrixing.md** - Marketing lens is one dimension in the tone matrix (Marketing presence: Zero -> Light -> Heavy)
- **gradual_exploration_process.md** - Apply marketing lens during Step 4 (Draft) or Step 8 (Publish)
