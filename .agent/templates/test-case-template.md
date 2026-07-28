# Test case: <name>

**Spec reference:** SPEC-XXX section N
**Invariant:** INV-NN (if applicable)
**Level:** unit | property | integration | e2e | live-fire | fuzz
**Path:** crates/<crate>/tests/<file>.rs

## Given
<Exact starting state. For core tests: the World fields that matter. For integration tests:
the fixture directory and its contents.>

## When
<Exact commands applied, or the exact tape replayed.>

## Then
<Exact assertion. For determinism tests, the expected state hash. For error tests, the exact
typed error variant.>

## Why this is not a mock
<One sentence naming the real thing under test. Required for every integration and e2e case.>
