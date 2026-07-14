# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""User model — T001, T004, T005."""
from dataclasses import dataclass, field, asdict
from datetime import datetime


@dataclass
class User:
    id: str
    name: str
    email: str
    created_at: datetime = field(default_factory=datetime.utcnow)

    def to_dict(self):
        d = asdict(self)
        d["created_at"] = self.created_at.isoformat()
        return d
