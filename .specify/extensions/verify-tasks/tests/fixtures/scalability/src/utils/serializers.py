# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Serialization helpers — T037, T038, T040."""
import json
import re
from typing import Type


def to_json(obj) -> str:
    """Serialize obj to a JSON string. Uses to_dict() if available."""
    if hasattr(obj, "to_dict"):
        return json.dumps(obj.to_dict())
    return json.dumps(obj)


def from_json(data: str, cls: Type):
    """Deserialize JSON string into an instance of cls."""
    d = json.loads(data)
    return cls(**d)


def slugify(text: str) -> str:
    """Convert text to a URL-safe lowercase slug."""
    text = text.lower().strip()
    text = re.sub(r"[^\w\s-]", "", text)
    return re.sub(r"[\s_-]+", "-", text)
