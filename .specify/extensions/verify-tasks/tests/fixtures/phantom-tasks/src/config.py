# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""
Application configuration.
Genuine implementation — T004.
"""
from dataclasses import dataclass


@dataclass
class AppConfig:
    """Holds application configuration values."""
    host: str = "0.0.0.0"
    port: int = 8080
    debug: bool = False
