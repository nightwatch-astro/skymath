# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""
Input validators — T003, T004.
Both functions referenced by src/pipeline.py.
"""
import re


def validate_email(email: str) -> bool:
    """Return True if email matches a basic RFC-5322 pattern."""
    pattern = r"^[^@\s]+@[^@\s]+\.[^@\s]+$"
    return bool(re.match(pattern, email))


def validate_phone(phone: str) -> bool:
    """Return True if phone is a plausible E.164 or local number."""
    digits = re.sub(r"[\s\-\(\)\+]", "", phone)
    return digits.isdigit() and 7 <= len(digits) <= 15
