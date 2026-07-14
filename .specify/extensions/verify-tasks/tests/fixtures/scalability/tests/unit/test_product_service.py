# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Unit tests for ProductService.create_product — T042."""
from services.product_service import ProductService


def test_create_product():
    svc = ProductService()
    p = svc.create_product("Widget", 9.99)
    assert p.name == "Widget"
    assert p.price == 9.99
    assert p.id
