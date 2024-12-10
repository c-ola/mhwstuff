import json

f = open("Enums_Internal.hpp")
data = f.read()
tokens = data.split()
len = len(tokens)
i = 0

namespaces = {}
namespaces_rev = {}
name = ""
enum_name = ""
enum_vals = {}
enum_vals_rev = {}

while i < len:
    if tokens[i] == "namespace":
        i += 1
        name = tokens[i]

    if tokens[i] == "enum":
        i += 1
        if tokens[i] == "class":
            i += 1
        enum_name = tokens[i]
        i += 1

    if  tokens[i+1] == "=":
        enum_id = tokens[i]
        i += 2
        enum_val = int(tokens[i].strip(","))
        enum_vals[enum_id] = enum_val
        enum_vals_rev[enum_val] = enum_id

    if tokens[i] == "};":
        i += 1
        full_name = name.replace("::", ".") + "." + enum_name
        namespaces[full_name] = enum_vals
        namespaces_rev[full_name] = enum_vals_rev
        name = ""
        enum_name = ""
        enum_vals = {}
        enum_vals_rev = {}

    i += 1

print(namespaces_rev)
with open("enums.json", "w") as file:
    json.dump(namespaces_rev, file, indent=4) 

