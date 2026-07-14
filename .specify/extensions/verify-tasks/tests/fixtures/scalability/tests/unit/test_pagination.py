# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Unit tests for paginate — T044."""
from utils.pagination import paginate


def test_paginate_first_page():
    items = list(range(10))
    assert paginate(items, 1, 3) == [0, 1, 2]


def test_paginate_last_page():
    items = list(range(10))
    assert paginate(items, 4, 3) == [9]


def test_paginate_empty():
    assert paginate([], 1, 5) == []
