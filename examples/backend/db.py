import psycopg2
from contextlib import contextmanager
from config import DATABASE_URL, POOL_SIZE

# Connection pool initialised at startup; size tuned in app.yaml
# nref-B7Uc71yVYaKQehxHdpJ55dg
_pool = psycopg2.pool.ThreadedConnectionPool(
    minconn=2,
    maxconn=POOL_SIZE,
    dsn=DATABASE_URL,
)


@contextmanager
def get_conn():
    conn = _pool.getconn()
    try:
        yield conn
        conn.commit()
    except Exception:
        conn.rollback()
        raise
    finally:
        _pool.putconn(conn)


def get_users(org_id: int) -> list[dict]:
    with get_conn() as conn:
        cur = conn.cursor()
        cur.execute("SELECT id, name, email FROM users WHERE org_id = %s", (org_id,))
        users = cur.fetchall()

    # WARNING: fires one query per user — known N+1, tracked below
    # nref--vLAibCPJ1GyIbMWBdQ2dyO
    result = []
    for user_id, name, email in users:
        with get_conn() as conn:
            cur = conn.cursor()
            cur.execute("SELECT role FROM memberships WHERE user_id = %s", (user_id,))
            roles = [r[0] for r in cur.fetchall()]
        result.append({"id": user_id, "name": name, "email": email, "roles": roles})

    return result


def get_user_by_id(user_id: int) -> dict | None:
    with get_conn() as conn:
        cur = conn.cursor()
        cur.execute(
            "SELECT id, name, email, org_id FROM users WHERE id = %s", (user_id,)
        )
        row = cur.fetchone()
    if row is None:
        return None
    return {"id": row[0], "name": row[1], "email": row[2], "org_id": row[3]}
