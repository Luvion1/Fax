#!/usr/bin/env python3
import json, sys

def view(d, i=0):
    p = "  " * i
    if isinstance(d, dict):
        t = d.get("type", "Obj")
        v = d.get("value", d.get("name", ""))
        print(f"{p}▶ \x1b[38;5;75m{t}\x1b[0m" + (f" (\x1b[38;5;84m{v}\x1b[0m)" if v else ""))
        for k, val in d.items():
            if k in ["type", "name", "loc", "range", "value"]: continue
            if isinstance(val, (dict, list)):
                print(f"{p}  {k}:")
                view(val, i + 2)
            else:
                print(f"{p}  {k}: \x1b[38;5;220m{val}\x1b[0m")
    elif isinstance(d, list):
        for item in d: view(item, i)
    else: print(f"{p}{d}")

if __name__ == "__main__":
    if len(sys.argv) < 2: 
        print("Usage: faxt-inspect <file.json>")
        sys.exit(1)
    try:
        with open(sys.argv[1]) as f: 
            view(json.load(f))
    except Exception as e:
        print(f"Error: {e}")
