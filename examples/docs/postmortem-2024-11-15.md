# Postmortem: API Degradation — 2024-11-15

**Severity:** SEV-2
**Duration:** 14:32–16:08 UTC (96 minutes)
**Author:** platform-oncall

## Summary

The `/users` endpoint degraded to P99 > 8 s under normal load following the
org-bulk-import feature release. Root cause was an N+1 query pattern that
had been present in the code but only became visible at scale after import
populated several orgs with thousands of members.

## Timeline

| Time (UTC) | Event |
|------------|-------|
| 14:32 | PagerDuty fires: `api_p99_latency > 2s` for 5 consecutive minutes |
| 14:41 | On-call identifies `/users` as the slow endpoint via Grafana |
| 15:03 | Code review finds N+1 in `db.get_users` — nref--vLAibCPJ1GyIbMWBdQ2dyO |
| 15:21 | Hotfix deployed: single JOIN query replaces per-user membership lookup |
| 16:08 | P99 returns to < 80 ms, incident closed |

## Root Cause

`db.get_users` fetched user rows in one query, then issued a separate
`SELECT role FROM memberships WHERE user_id = ?` for each user in a Python
loop. For orgs with O(1000) members this produced O(1000) round-trips per
request. Under concurrent traffic the database connection pool saturated,
queuing further requests and cascading into a full degradation.

## Action Items

- [x] Replace loop with a single `JOIN` query (hotfix, deployed 15:21)
- [ ] Add a query-count assertion to the integration test suite so this
      class of regression is caught before deploy
- [ ] Review other list endpoints for the same pattern

## JWT secret rotation gap

During the incident response we noticed the JWT secret has never been
rotated since initial deploy (11 months ago). This is a separate risk.
Rotation procedure needs to be documented and tested. nref-EpbgIhJ9p89YzVSHr9dfRB3
