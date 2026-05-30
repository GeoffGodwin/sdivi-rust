# simple-python fixture: main.py
# Imports: 4 | Exports: 1
# Extended in M33: added requests.get (data_access) and logging.info (logging) calls.
import os
import sys
import requests
import logging
from utils import helper
from models import User


def run():
    path = os.getcwd()
    args = sys.argv
    u = User(name=args[0])
    helper(path)
    logging.info("Starting run for %s", args[0])
    resp = requests.get("http://example.com/api")
    return u
