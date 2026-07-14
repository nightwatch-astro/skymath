# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Logging middleware — T022."""
import sys


class LoggingMiddleware:
    def __call__(self, request, next_handler):
        print(f"[LOG] {request.get('method', 'GET')} {request.get('path', '/')}", file=sys.stdout)
        response = next_handler(request)
        print(f"[LOG] → {response.get('status', 200)}", file=sys.stdout)
        return response
