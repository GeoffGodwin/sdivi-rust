# simple-python fixture: models.py
# Imports: 1 | Exports: 1
from typing import Optional


class User:
    def __init__(self, name: str, email: Optional[str] = None):
        self.name = name
        self.email = email
