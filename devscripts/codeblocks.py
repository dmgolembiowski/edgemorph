#!/usr/bin/env python3
import sys

if __name__ == "__main__":
    if not sys.argv[1]:
        print("No file supplied")
        sys.exit(1)
    else:
        tokens = []
        tokens.append("<pre><code>")
        with open(sys.argv[1], "r") as f:
            lines = f.readlines()
        for line in lines:
            tokens.append(line)
            tokens.append("<br>")
        tokens.pop()
        tokens.append("</code></pre>")
        print(''.join([t.strip("\n") for t in tokens]))
