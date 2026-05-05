# simple-python fixture: main.py
# Imports: 4 | Exports: 1
import os
import sys
from utils import helper
from models import User


def run():
    path = os.getcwd()
    args = sys.argv
    u = User(name=args[0])
    helper(path)
    return u
