import pytest
from unittest.mock import patch
from backend.api import app


@pytest.fixture
def client():
    app.config["JWT_SECRET"] = "test-secret"
    app.config["TESTING"] = True
    with app.test_client() as c:
        yield c


def make_token(user_id=1):
    import jwt
    return jwt.encode({"sub": user_id}, "test-secret", algorithm="HS256")


def test_list_users_requires_auth(client):
    resp = client.get("/users?org_id=1")
    assert resp.status_code == 401


def test_list_users_returns_members(client):
    token = make_token()
    with patch("backend.api.db.get_users") as mock_get:
        mock_get.return_value = [
            {"id": 1, "name": "Alice", "email": "alice@example.com", "roles": ["admin"]},
            {"id": 2, "name": "Bob",   "email": "bob@example.com",   "roles": ["member"]},
        ]
        resp = client.get("/users?org_id=42", headers={"Authorization": f"Bearer {token}"})
    assert resp.status_code == 200
    assert len(resp.get_json()) == 2


def test_get_user_not_found(client):
    token = make_token()
    with patch("backend.api.db.get_user_by_id", return_value=None):
        resp = client.get("/users/9999", headers={"Authorization": f"Bearer {token}"})
    assert resp.status_code == 404


# This test is flaky under parallel pytest-xdist runs: the in-process rate
# counter is shared state and resets don't happen between workers.
# nref-eMTdGQL0dbZnjEseUYxnPui
@pytest.mark.flaky(reruns=3)
def test_rate_limit_enforced(client):
    token = make_token(user_id=99)
    headers = {"Authorization": f"Bearer {token}"}
    with patch("backend.api.db.get_users", return_value=[]):
        for _ in range(100):
            client.get("/users?org_id=1", headers=headers)
        resp = client.get("/users?org_id=1", headers=headers)
    assert resp.status_code == 429
