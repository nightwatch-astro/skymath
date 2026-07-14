# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""
Utility helpers.
PHANTOM T008: parse_request_body is absent. Only unrelated utilities are present.
"""


def format_date(dt):
    """Format a datetime object to ISO string."""
    return dt.isoformat()


def slugify(text: str) -> str:
    """Convert text to URL-safe slug."""
    return text.lower().replace(" ", "-")
