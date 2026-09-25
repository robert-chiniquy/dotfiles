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
- A Wikipedia URL that is not an article, or a `#fragment` that is not a heading on that article. HTTP 200 is not enough: Wikipedia returns 200 for a missing fragment. Worked fail: `https://en.wikipedia.org/wiki/Crash_(computing)#Crash-only_software` (no such heading; the article is `Crash-only_software`). `XML_Signature_Wrapping` and `Transition_semigroup` 404. Fetch the article, confirm the heading, or drop the link.
- Invented product wording when a primary source already names the object. Worked fail: "connector draft" for Functions. Public docs: Functions start as drafts; Save draft; Test draft; Publish. Connector docs say connector, sync, version, config. Use the public name. If the object is Functions, say Functions in ethos.
- Treating a later correction as making the earlier untrue sentence OK.
- Inventing a vendor defect. Cloudflare's published 2,594-tool / 1.17M-token listing is a size measurement. It is not a missed-`if` that left `/admin` in `tools/list`. A public GitHub issue or advisory is a receipt; a constructed `if` on their generator is not.

Worked fail: `Suppose the Okta AWS Federation connector would have recorded Alice's federated AWS login as AdministratorAccess.` That connector records IAM roles from Okta AWS-app assignment profiles. Wrapping extra Role is recorded by a first-Assertion SAML consumer of the login XML. Those actors are not interchangeable.

## Pass

Read-only. Per live `DRAFT.md` (or gist file if published):

0. Every `en.wikipedia.org` URL: article exists; fragment is a heading on that page.
1. List operational claims (records, reads, writes, signs, assumes, certifies, parses, evals, serves, opens, grants).
2. For each: name the actor, the object, the effect. Is that effect true of that actor?
2b. Claim vs exhibit: count inputs, stores, states, outputs on the exhibit and in the surrounding prose. Same objects. Conjunction vs product. Guard table vs transition table. Draft store vs published store. Static catalog vs runtime eval vs test harness. `missing` / `empty` / `false` / `residual` / `diagnostic` keep one conversion order. A true isolated fact can still be false in this post's model.
3. Quote the untrue sentence. Quote the true fact (same post, linked doc, or this skill's worked fail). Rewrite so the actor matches the effect, or drop the sentence.

## Before finishing

- Every untrue claim has a quote, a true fact, and a rewrite
- Subjunctive refused paths were not flagged solely for being hypothetical
- No WHY or referent flags
