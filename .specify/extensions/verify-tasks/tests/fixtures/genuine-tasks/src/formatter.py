# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""
Output formatters — T005, T006.
Both functions referenced by src/pipeline.py and src/app.py.
"""
from datetime import datetime


def format_currency(amount: float, currency: str = "USD") -> str:
    """Return amount formatted as a currency string, e.g. '$1,234.56'."""
    symbols = {"USD": "$", "EUR": "€", "GBP": "£"}
    sym = symbols.get(currency, currency + " ")
    return f"{sym}{amount:,.2f}"


def format_date(dt: datetime, fmt: str = "%Y-%m-%d") -> str:
    """Return dt formatted according to fmt."""
    return dt.strftime(fmt)
