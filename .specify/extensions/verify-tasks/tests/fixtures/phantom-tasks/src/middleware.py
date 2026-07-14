# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""
Logging middleware.
PHANTOM T009: LoggingMiddleware class exists but __call__ method is missing — behavioral gap.
"""


class LoggingMiddleware:
    """Logs incoming requests. INCOMPLETE — __call__ not implemented."""

    def __init__(self, app):
        self.app = app
