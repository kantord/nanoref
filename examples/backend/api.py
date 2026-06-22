from flask import Flask, request, jsonify, g
from functools import wraps
import jwt
import time
import db

app = Flask(__name__)

# Rate-limit state lives in-process; not suitable for multi-replica deploy.
# See architecture note before scaling horizontally.
# nref-dzWVZ98hJTPoQxPgtBsKo8V
_rate_counters: dict[str, tuple[int, float]] = {}
RATE_LIMIT = 100  # requests per minute per token


def rate_limited(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        token = request.headers.get("Authorization", "")
        now = time.time()
        count, window_start = _rate_counters.get(token, (0, now))
        if now - window_start > 60:
            count, window_start = 0, now
        if count >= RATE_LIMIT:
            return jsonify({"error": "rate limit exceeded"}), 429
        _rate_counters[token] = (count + 1, window_start)
        return f(*args, **kwargs)
    return wrapper


def require_auth(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        auth = request.headers.get("Authorization", "")
        if not auth.startswith("Bearer "):
            return jsonify({"error": "unauthorized"}), 401
        try:
            # Secret is read from env at startup; rotation requires restart.
            # Incident history: nref-EpbgIhJ9p89YzVSHr9dfRB3
            payload = jwt.decode(auth[7:], app.config["JWT_SECRET"], algorithms=["HS256"])
            g.user_id = payload["sub"]
        except jwt.InvalidTokenError:
            return jsonify({"error": "invalid token"}), 401
        return f(*args, **kwargs)
    return wrapper


@app.route("/users")
@require_auth
@rate_limited
def list_users():
    org_id = request.args.get("org_id", type=int)
    if org_id is None:
        return jsonify({"error": "org_id required"}), 400
    # This hits the N+1 path in db.get_users — see nref--vLAibCPJ1GyIbMWBdQ2dyO
    users = db.get_users(org_id)
    return jsonify(users)


@app.route("/users/<int:user_id>")
@require_auth
def get_user(user_id: int):
    user = db.get_user_by_id(user_id)
    if user is None:
        return jsonify({"error": "not found"}), 404
    return jsonify(user)


# TODO: migrate to async (FastAPI + asyncpg) once load tests confirm the
# throughput ceiling. Blocking on DB driver evaluation. nref--jXcxRfLqlOyuO7ZBrvbAJg
if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8080)
