# Known Issues

## Flaky rate-limit test under parallel test runs

The `test_rate_limit_enforced` test shares in-process counter state with
other workers when run under `pytest-xdist -n auto`. It fails non-deterministically
depending on which worker resets the counter. Marked with `@pytest.mark.flaky`
as a short-term workaround. nref-eMTdGQL0dbZnjEseUYxnPui

Proper fix: replace the in-process dict with a Redis-backed counter, which
also unblocks horizontal scaling (see nref-dzWVZ98hJTPoQxPgtBsKo8V).
