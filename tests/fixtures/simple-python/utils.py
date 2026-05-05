# simple-python fixture: utils.py
# Imports: 1 | Exports: 1
import re


def helper(path):
    return re.sub(r"/$", "", path)
