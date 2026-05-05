# simple-python-relative fixture: pkg/__init__.py
# Exercises Python relative-import specifiers: "." and ".models"
from . import models
from .models import User


def make_user(name: str) -> "User":
    return User(name=name)
