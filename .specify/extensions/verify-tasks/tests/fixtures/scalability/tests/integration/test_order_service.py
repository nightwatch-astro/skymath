# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Integration tests for OrderService.place_order — T043."""
from services.order_service import OrderService


def test_place_order_and_retrieve():
    svc = OrderService()
    order = svc.place_order("user-1", [{"product_id": "p-1", "quantity": 2, "unit_price": 10.0}])
    assert order.user_id == "user-1"
    assert len(order.items) == 1

    orders = svc.get_user_orders("user-1")
    assert any(o.id == order.id for o in orders)
