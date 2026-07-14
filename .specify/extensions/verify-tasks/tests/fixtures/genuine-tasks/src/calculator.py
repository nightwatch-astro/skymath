# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""
Calculator — T001, T002.
All methods implemented and referenced by src/runner.py and src/app.py.
"""


class Calculator:
    """Simple arithmetic calculator."""

    def add(self, a: float, b: float) -> float:
        """Return a + b."""
        return a + b

    def subtract(self, a: float, b: float) -> float:
        """Return a - b."""
        return a - b

    def multiply(self, a: float, b: float) -> float:
        """Return a * b."""
        return a * b

    def divide(self, a: float, b: float) -> float:
        """Return a / b. Raises ZeroDivisionError if b is zero."""
        if b == 0:
            raise ZeroDivisionError("Cannot divide by zero")
        return a / b
