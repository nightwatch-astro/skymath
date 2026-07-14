# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""User handler — T016, T019."""
from services.user_service import UserService


class UserHandler:
    def __init__(self, service: UserService = None):
        self._svc = service or UserService()

    def create(self, request):
        body = request.get("body", {})
        user = self._svc.register(body["name"], body["email"])
        return {"status": 201, "body": user.to_dict()}

    def update(self, request):
        body = request.get("body", {})
        return {"status": 200, "body": body}
