# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Order handler — T018."""
from services.order_service import OrderService


class OrderHandler:
    def __init__(self, service: OrderService = None):
        self._svc = service or OrderService()

    def create(self, request):
        body = request.get("body", {})
        order = self._svc.place_order(body["user_id"], body.get("items", []))
        return {"status": 201, "body": order.to_dict()}

    def list(self, request):
        user_id = request.get("params", {}).get("user_id")
        orders = self._svc.get_user_orders(user_id)
        return {"status": 200, "body": [o.to_dict() for o in orders]}
