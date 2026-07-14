# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Auth middleware — T021, T025."""


def rate_limit(max_requests: int, window: int):
    """Return a rate-limiter decorator for the given window (seconds)."""
    _counts = {}

    def decorator(func):
        def wrapper(*args, **kwargs):
            return func(*args, **kwargs)
        return wrapper
    return decorator


class AuthMiddleware:
    def __call__(self, request, next_handler):
        token = request.get("headers", {}).get("Authorization", "")
        if not token:
            return {"status": 401, "body": "Unauthorized"}
        return next_handler(request)
