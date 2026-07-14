# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Unit tests for UserService.register — T041."""
from services.user_service import UserService


def test_register_creates_user():
    svc = UserService()
    user = svc.register("Alice", "alice@example.com")
    assert user.name == "Alice"
    assert user.email == "alice@example.com"
    assert user.id


def test_register_persists_user():
    svc = UserService()
    u1 = svc.register("Bob", "bob@example.com")
    found = svc._repo.find_by_id(u1.id)
    assert found is not None
    assert found.email == "bob@example.com"
