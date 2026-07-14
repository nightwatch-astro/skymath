# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""CORS middleware — T023."""


class CorsMiddleware:
    def __call__(self, request, next_handler):
        response = next_handler(request)
        if isinstance(response, dict):
            headers = response.setdefault("headers", {})
            headers["Access-Control-Allow-Origin"] = "*"
        return response
