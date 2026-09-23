---
name: fact-review
description: Catch factually untrue operational claims in C1 engineering-blog drafts. Actor cannot produce the effect; two objects treated as one; product path misstated. Use on every post, every fleet iteration, or when a scene names a C1 connector, grant list, session path, or SAML consumer doing something that actor does not do.
---

# Fact review

One axis: is the sentence true of the named actor and object. An engineer who believes the sentence would be wrong about the product.

## Common Mistakes

- Scoring missing WHY, blank referents, or ethos. Those are other personas.
- Flagging subjunctive C1 prevention as untrue. `would` of a refused path is allowed when the cause is true of that actor.
- Inventing product behavior. Verify against the sentence's own later exhibit, C1 docs linked in the post, or the named connector's job in this file.
- Treating a later correction as making the earlier untrue sentence OK.

Worked fail: `Suppose the Okta AWS Federation connector would have recorded Alice's federated AWS login as AdministratorAccess.` That connector records IAM roles from Okta AWS-app assignment profiles. Wrapping extra Role is recorded by a first-Assertion SAML consumer of the login XML. Those actors are not interchangeable.

## Pass

Read-only. Per live `DRAFT.md` (or gist file if published):

1. List operational claims (records, reads, writes, signs, assumes, certifies, parses, evals, serves, opens, grants).
2. For each: name the actor, the object, the effect. Is that effect true of that actor?
3. Quote the untrue sentence. Quote the true fact (same post, linked doc, or this skill's worked fail). Rewrite so the actor matches the effect, or drop the sentence.

## Before finishing

- Every untrue claim has a quote, a true fact, and a rewrite
- Subjunctive refused paths were not flagged solely for being hypothetical
- No WHY or referent flags
