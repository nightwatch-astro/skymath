# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""
Application entry point — T010.
Imports and uses FileStore, Pipeline, and Calculator.
"""
from calculator import Calculator
from storage import FileStore
from pipeline import Pipeline


def run():
    # Use Calculator
    calc = Calculator()
    result = calc.multiply(6, 7)

    # Use Pipeline
    pipe = Pipeline()
    pipe_result = pipe.process("user@example.com", result, "USD")

    # Use FileStore
    store = FileStore()
    store.save("result", pipe_result)
    loaded = store.load("result")

    print("Result:", loaded)


if __name__ == "__main__":
    run()
