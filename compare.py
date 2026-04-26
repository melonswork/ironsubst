#!/usr/bin/env python3
import subprocess
import os
import sys

# Build ironsubst first
subprocess.run(["cargo", "build", "-q"], check=True)

# Define tools
tools = [
    ("ironsubst (default)", ["./target/debug/ironsubst"]),
    ("ironsubst --require-values", ["./target/debug/ironsubst", "--require-values"]),
    (
        "ironsubst --require-nonempty-values",
        ["./target/debug/ironsubst", "--require-nonempty-values"],
    ),
    ("a8m/envsubst (default)", ["mise", "exec", "envsubst", "--", "envsubst"]),
    (
        "a8m/envsubst -no-unset",
        ["mise", "exec", "envsubst", "--", "envsubst", "-no-unset"],
    ),
    (
        "a8m/envsubst -no-empty",
        ["mise", "exec", "envsubst", "--", "envsubst", "-no-empty"],
    ),
    ("bash (default)", ["bash", "-c"]),
    ("bash (set -u)", ["bash", "-c"]),
]

# Gettext envsubst — optional, via env var
gettext_path = os.environ.get("GETTEXT_ENVSUBST_PATH")
if gettext_path:
    tools.insert(6, ("gettext envsubst", [gettext_path]))

# Define states and variables
states = {"UNSET": "UNSET_VAR", "EMPTY": "EMPTY_VAR", "SET": "SET_VAR"}

# The environment used for the test
env = os.environ.copy()
env.pop("UNSET_VAR", None)
env["EMPTY_VAR"] = ""
env["SET_VAR"] = "val"

# Define operators/placeholders (suffix to add to the var name)
operators = {
    "Bare": "",
    "Def (-)": "-fb",
    "Def (:-)": ":-fb",
    "Assign (=)": "=fb",
    "Assign (:=)": ":=fb",
    "Empty Assign (:=)": ":=",
    "Subst (+)": "+alt",
    "Subst (:+)": ":+alt",
    "Error (?)": "?err",
    "Error (:?)": ":?err",
}


def format_output(stdout, stderr, rc):
    out = (stdout + stderr).strip()
    out = out.replace("\n", " ")

    if rc != 0:
        if out:
            short_err = out if len(out) < 25 else out[:22] + "..."
            return f"ERR({rc}): `{short_err}`"
        return f"ERR({rc})"
    else:
        if not out:
            return '`""`'
        return f"`{out}`"


with open("comparison.md", "w") as f:
    f.write("# Environment Substitution Comparison\n\n")

    if not gettext_path:
        f.write(
            "> **Note:** gettext envsubst is not tested. "
            "Set `GETTEXT_ENVSUBST_PATH` to the binary path to include it.\n\n"
        )

    for state_name, var_name in states.items():
        f.write(f"## Variable state: {state_name}\n\n")

        headers = ["Tool"] + [
            f"`${{{var_name}{op}}}`<br>{name}" for name, op in operators.items()
        ]
        f.write("| " + " | ".join(headers) + " |\n")
        f.write("|-" + "-|-".join(["-" * len(h) for h in headers]) + "-|\n")

        for tool_name, tool_cmd in tools:
            row = [f"**{tool_name}**"]

            for op_name, op_val in operators.items():
                payload = f"${{{var_name}{op_val}}}"

                if tool_cmd[0] == "bash":
                    if "set -u" in tool_name:
                        cmd = ["bash", "-c", f'set -u; echo -n "{payload}"']
                    else:
                        cmd = ["bash", "-c", f'echo -n "{payload}"']

                    try:
                        res = subprocess.run(
                            cmd, env=env, capture_output=True, text=True
                        )
                        cell = format_output(res.stdout, res.stderr, res.returncode)
                    except Exception as e:
                        cell = f"ERR: {e}"
                else:
                    try:
                        res = subprocess.run(
                            tool_cmd,
                            env=env,
                            input=payload,
                            capture_output=True,
                            text=True,
                        )
                        cell = format_output(res.stdout, res.stderr, res.returncode)
                    except Exception as e:
                        cell = f"ERR: {e}"

                row.append(cell)

            f.write("| " + " | ".join(row) + " |\n")
        f.write("\n")

    # String manipulation operators — only meaningful with a set, non-empty variable.
    # envsubst (both a8m and gettext) does not support any of these.
    f.write("## String Manipulation Operators\n\n")
    f.write(
        "> These operators are supported by ironsubst and bash but **not** by "
        "a8m/envsubst (which outputs the expression verbatim).  \n"
        "> Tested with `SET_VAR=val`.\n\n"
    )

    string_ops = [
        ("`${#SET_VAR}`",        "${#SET_VAR}"),
        ("`${SET_VAR#v}`",       "${SET_VAR#v}"),
        ("`${SET_VAR##v*}`",     "${SET_VAR##v*}"),
        ("`${SET_VAR%l}`",       "${SET_VAR%l}"),
        ("`${SET_VAR%%*l}`",     "${SET_VAR%%*l}"),
        ("`${SET_VAR:1}`",       "${SET_VAR:1}"),
        ("`${SET_VAR:0:2}`",     "${SET_VAR:0:2}"),
    ]

    string_tools = [
        ("ironsubst (default)", ["./target/debug/ironsubst"]),
        ("a8m/envsubst (default)", ["mise", "exec", "envsubst", "--", "envsubst"]),
        ("bash (default)", ["bash", "-c"]),
    ]
    if gettext_path:
        string_tools.insert(2, ("gettext envsubst", [gettext_path]))

    headers = ["Expression"] + [t[0] for t in string_tools]
    f.write("| " + " | ".join(headers) + " |\n")
    f.write("|-" + "-|-".join(["-" * len(h) for h in headers]) + "-|\n")

    for op_label, payload in string_ops:
        row = [f"**{op_label}**"]
        for tool_name, tool_cmd in string_tools:
            if tool_cmd[0] == "bash":
                cmd = ["bash", "-c", f'echo -n "{payload}"']
                try:
                    res = subprocess.run(cmd, env=env, capture_output=True, text=True)
                    cell = format_output(res.stdout, res.stderr, res.returncode)
                except Exception as e:
                    cell = f"ERR: {e}"
            else:
                try:
                    res = subprocess.run(
                        tool_cmd,
                        env=env,
                        input=payload,
                        capture_output=True,
                        text=True,
                    )
                    cell = format_output(res.stdout, res.stderr, res.returncode)
                except Exception as e:
                    cell = f"ERR: {e}"
            row.append(cell)
        f.write("| " + " | ".join(row) + " |\n")
    f.write("\n")

    f.write("---\n*Generated by `compare.py`*\n")
