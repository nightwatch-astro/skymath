# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""User service — T011."""
import uuid
from models.user import User
from repos.user_repo import UserRepository


class UserService:
    def __init__(self, repo: UserRepository = None):
        self._repo = repo or UserRepository()

    def register(self, name: str, email: str) -> User:
        user = User(id=str(uuid.uuid4()), name=name, email=email)
        self._repo.save(user)
        return user
