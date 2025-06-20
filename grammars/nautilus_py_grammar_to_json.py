#!/usr/bin/env python
# -*- coding: utf-8 -*-

#
# american fuzzy lop++ - grammar mutator
# --------------------------------------
#
# Written by Shengtuo Hu
#
# Copyright 2020 AFLplusplus Project. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at:
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# A grammar-based custom mutator written for GSoC '20.
#

import re
import sys
import json

OLD_KEY_RE = r'\{(?P<key>[a-zA-Z0-9_]+?)\}'
NEW_KEY_RE = r'(?P<key><[a-zA-Z0-9_]+?>)'


class Context(object):
    data = {}

    def __init__(self):
        super(Context, self).__init__()

    def rule(self, key, rule):
        key = '<%s>' % (key)
        if self.data.get(key) is None:
            self.data[key] = []

        rule = re.sub(OLD_KEY_RE, r'<\g<key>>', rule)  # key

        rule = re.sub(r'\\{', '{', rule)
        rule = re.sub(r'\\}', '}', rule)
        rule = re.sub(r'\\\\', '\\\\', rule)

        rule = re.split(NEW_KEY_RE, rule)
        rule = list(filter(None, rule))  # remove empty strings
        self.data[key].append(rule)


def main():
    if len(sys.argv) > 2:
        input_file = sys.argv[1]
        output_file = sys.argv[2]

    ctx = Context()
    with open(input_file, 'r') as fp:
        exec(fp.read())

    with open(output_file, 'w') as fp:
        json.dump(ctx.data, fp, indent=4)


if __name__ == '__main__':
    main()

