# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Product model — T002, T004, T005."""
from dataclasses import dataclass, asdict


@dataclass
class Product:
    id: str
    name: str
    price: float
    stock: int

    def to_dict(self):
        return asdict(self)
