# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Product handler — T017."""
from services.product_service import ProductService


class ProductHandler:
    def __init__(self, service: ProductService = None):
        self._svc = service or ProductService()

    def list_all(self, request):
        return {"status": 200, "body": []}

    def get(self, request):
        return {"status": 200, "body": {}}
