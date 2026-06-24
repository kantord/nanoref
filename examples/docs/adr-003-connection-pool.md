# ADR-003: Use a threaded connection pool for PostgreSQL

**Status:** Accepted
**Date:** 2024-08-22
**Deciders:** backend-team

## Context

The service started with a single persistent connection opened at import time.
Under load testing at >20 req/s the connection became a bottleneck: Flask
worker threads serialised on it and P99 latency climbed linearly with
concurrency.

## Decision

Switch to `psycopg2.pool.ThreadedConnectionPool`. Pool parameters (min/max
connections) are externalised to `app.yaml` so they can be tuned per
environment without a code change. nref-B7Uc71yVYaKQehxHdpJ55dg

Chosen over alternatives:
- **SQLAlchemy connection pool** — heavier dependency, ORM not needed here
- **pgBouncer sidecar** — operational overhead not justified at current scale
- **asyncpg** — requires full async rewrite, deferred for now

## Consequences

- Each gunicorn worker holds up to `pool_size` connections; total DB
  connections = `workers × pool_size`. The PostgreSQL `max_connections`
  limit must be set accordingly.
- Connection pool metrics are not yet exposed via `/metrics`. Adding
  `pool.closed` / `pool.used` gauges is a follow-up item.

## Rate limiting note

In-process rate limiting (nref-dzWVZ98hJTPoQxPgtBsKo8V) shares the same
single-process assumption as this pooling approach. Both decisions need
revisiting before horizontal scaling.
