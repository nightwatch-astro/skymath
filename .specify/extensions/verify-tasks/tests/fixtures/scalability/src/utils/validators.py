# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Input validators — T039, T040."""
import re
import uuid


def validate_email(email: str) -> bool:
    """Return True if email matches a basic RFC-5322 pattern."""
    return bool(re.match(r"^[^@\s]+@[^@\s]+\.[^@\s]+$", email))


def validate_uuid(value: str) -> bool:
    """Return True if value is a valid UUID."""
    try:
        uuid.UUID(str(value))
        return True
    except ValueError:
        return False
