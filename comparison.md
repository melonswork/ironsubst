# Environment Substitution Comparison

## Variable state: UNSET

| Tool | `${UNSET_VAR}`<br>Bare | `${UNSET_VAR-fb}`<br>Def (-) | `${UNSET_VAR:-fb}`<br>Def (:-) | `${UNSET_VAR=fb}`<br>Assign (=) | `${UNSET_VAR:=fb}`<br>Assign (:=) | `${UNSET_VAR:=}`<br>Empty Assign (:=) | `${UNSET_VAR+alt}`<br>Subst (+) | `${UNSET_VAR:+alt}`<br>Subst (:+) | `${UNSET_VAR?err}`<br>Error (?) | `${UNSET_VAR:?err}`<br>Error (:?) |
|------|------------------------|------------------------------|--------------------------------|---------------------------------|-----------------------------------|---------------------------------------|---------------------------------|-----------------------------------|---------------------------------|-----------------------------------|
| **ironsubst (default)** | `""` | `fb` | `fb` | `fb` | `fb` | `""` | `""` | `""` | ERR(1): `UNSET_VAR: err` | ERR(1): `UNSET_VAR: err` |
| **ironsubst --require-values** | ERR(1): `variable ${UNSET_VAR} ...` | `fb` | `fb` | `fb` | `fb` | `""` | `""` | `""` | ERR(1): `UNSET_VAR: err` | ERR(1): `UNSET_VAR: err` |
| **ironsubst --require-nonempty-values** | `""` | `fb` | `fb` | `fb` | `fb` | `""` | `""` | `""` | ERR(1): `UNSET_VAR: err` | ERR(1): `UNSET_VAR: err` |
| **a8m/envsubst (default)** | `""` | `fb` | `fb` | `fb` | `fb` | `""` | `""` | `""` | `""` | `""` |
| **a8m/envsubst -no-unset** | ERR(1): `variable ${UNSET_VAR} ...` | `fb` | `fb` | `fb` | `fb` | ERR(1): `variable ${UNSET_VAR} ...` | `""` | `""` | ERR(1): `variable ${UNSET_VAR} ...` | ERR(1): `variable ${UNSET_VAR} ...` |
| **a8m/envsubst -no-empty** | `""` | `fb` | `fb` | `fb` | `fb` | `""` | `""` | `""` | `""` | `""` |
| **gettext envsubst** | `""` | `${UNSET_VAR-fb}` | `${UNSET_VAR:-fb}` | `${UNSET_VAR=fb}` | `${UNSET_VAR:=fb}` | `${UNSET_VAR:=}` | `${UNSET_VAR+alt}` | `${UNSET_VAR:+alt}` | `${UNSET_VAR?err}` | `${UNSET_VAR:?err}` |
| **bash (default)** | `""` | `fb` | `fb` | `fb` | `fb` | `""` | `""` | `""` | ERR(127): `bash: line 1: UNSET_VA...` | ERR(127): `bash: line 1: UNSET_VA...` |
| **bash (set -u)** | ERR(127): `bash: line 1: UNSET_VA...` | `fb` | `fb` | `fb` | `fb` | `""` | `""` | `""` | ERR(127): `bash: line 1: UNSET_VA...` | ERR(127): `bash: line 1: UNSET_VA...` |

## Variable state: EMPTY

| Tool | `${EMPTY_VAR}`<br>Bare | `${EMPTY_VAR-fb}`<br>Def (-) | `${EMPTY_VAR:-fb}`<br>Def (:-) | `${EMPTY_VAR=fb}`<br>Assign (=) | `${EMPTY_VAR:=fb}`<br>Assign (:=) | `${EMPTY_VAR:=}`<br>Empty Assign (:=) | `${EMPTY_VAR+alt}`<br>Subst (+) | `${EMPTY_VAR:+alt}`<br>Subst (:+) | `${EMPTY_VAR?err}`<br>Error (?) | `${EMPTY_VAR:?err}`<br>Error (:?) |
|------|------------------------|------------------------------|--------------------------------|---------------------------------|-----------------------------------|---------------------------------------|---------------------------------|-----------------------------------|---------------------------------|-----------------------------------|
| **ironsubst (default)** | `""` | `""` | `fb` | `""` | `fb` | `""` | `alt` | `""` | `""` | ERR(1): `EMPTY_VAR: err` |
| **ironsubst --require-values** | `""` | `""` | `fb` | `""` | `fb` | `""` | `alt` | `""` | `""` | ERR(1): `EMPTY_VAR: err` |
| **ironsubst --require-nonempty-values** | ERR(1): `variable ${EMPTY_VAR} ...` | ERR(1): `variable ${EMPTY_VAR} ...` | `fb` | ERR(1): `variable ${EMPTY_VAR} ...` | `fb` | `""` | `alt` | `""` | `""` | ERR(1): `EMPTY_VAR: err` |
| **a8m/envsubst (default)** | `""` | `""` | `fb` | `""` | `fb` | `""` | `alt` | `alt` | `""` | `""` |
| **a8m/envsubst -no-unset** | `""` | `""` | `fb` | `""` | `fb` | `""` | `alt` | `alt` | `""` | `""` |
| **a8m/envsubst -no-empty** | ERR(1): `variable ${EMPTY_VAR} ...` | ERR(1): `variable ${EMPTY_VAR} ...` | `fb` | ERR(1): `variable ${EMPTY_VAR} ...` | `fb` | ERR(1): `variable ${EMPTY_VAR} ...` | `alt` | `alt` | ERR(1): `variable ${EMPTY_VAR} ...` | ERR(1): `variable ${EMPTY_VAR} ...` |
| **gettext envsubst** | `""` | `${EMPTY_VAR-fb}` | `${EMPTY_VAR:-fb}` | `${EMPTY_VAR=fb}` | `${EMPTY_VAR:=fb}` | `${EMPTY_VAR:=}` | `${EMPTY_VAR+alt}` | `${EMPTY_VAR:+alt}` | `${EMPTY_VAR?err}` | `${EMPTY_VAR:?err}` |
| **bash (default)** | `""` | `""` | `fb` | `""` | `fb` | `""` | `alt` | `""` | `""` | ERR(127): `bash: line 1: EMPTY_VA...` |
| **bash (set -u)** | `""` | `""` | `fb` | `""` | `fb` | `""` | `alt` | `""` | `""` | ERR(127): `bash: line 1: EMPTY_VA...` |

## Variable state: SET

| Tool | `${SET_VAR}`<br>Bare | `${SET_VAR-fb}`<br>Def (-) | `${SET_VAR:-fb}`<br>Def (:-) | `${SET_VAR=fb}`<br>Assign (=) | `${SET_VAR:=fb}`<br>Assign (:=) | `${SET_VAR:=}`<br>Empty Assign (:=) | `${SET_VAR+alt}`<br>Subst (+) | `${SET_VAR:+alt}`<br>Subst (:+) | `${SET_VAR?err}`<br>Error (?) | `${SET_VAR:?err}`<br>Error (:?) |
|------|----------------------|----------------------------|------------------------------|-------------------------------|---------------------------------|-------------------------------------|-------------------------------|---------------------------------|-------------------------------|---------------------------------|
| **ironsubst (default)** | `val` | `val` | `val` | `val` | `val` | `val` | `alt` | `alt` | `val` | `val` |
| **ironsubst --require-values** | `val` | `val` | `val` | `val` | `val` | `val` | `alt` | `alt` | `val` | `val` |
| **ironsubst --require-nonempty-values** | `val` | `val` | `val` | `val` | `val` | `val` | `alt` | `alt` | `val` | `val` |
| **a8m/envsubst (default)** | `val` | `val` | `val` | `val` | `val` | `val` | `alt` | `alt` | `val` | `val` |
| **a8m/envsubst -no-unset** | `val` | `val` | `val` | `val` | `val` | `val` | `alt` | `alt` | `val` | `val` |
| **a8m/envsubst -no-empty** | `val` | `val` | `val` | `val` | `val` | `val` | `alt` | `alt` | `val` | `val` |
| **gettext envsubst** | `val` | `${SET_VAR-fb}` | `${SET_VAR:-fb}` | `${SET_VAR=fb}` | `${SET_VAR:=fb}` | `${SET_VAR:=}` | `${SET_VAR+alt}` | `${SET_VAR:+alt}` | `${SET_VAR?err}` | `${SET_VAR:?err}` |
| **bash (default)** | `val` | `val` | `val` | `val` | `val` | `val` | `alt` | `alt` | `val` | `val` |
| **bash (set -u)** | `val` | `val` | `val` | `val` | `val` | `val` | `alt` | `alt` | `val` | `val` |

## String Manipulation Operators

> These operators are supported by ironsubst and bash but **not** by a8m/envsubst (which outputs the expression verbatim).  
> Tested with `SET_VAR=val`.

| Expression | ironsubst (default) | a8m/envsubst (default) | gettext envsubst | bash (default) |
|------------|---------------------|------------------------|------------------|----------------|
| **`${#SET_VAR}`** | `3` | `""` | `${#SET_VAR}` | `3` |
| **`${SET_VAR#v}`** | `al` | `val` | `${SET_VAR#v}` | `al` |
| **`${SET_VAR##v*}`** | `""` | `val` | `${SET_VAR##v*}` | `""` |
| **`${SET_VAR%l}`** | `va` | `val` | `${SET_VAR%l}` | `va` |
| **`${SET_VAR%%*l}`** | `""` | `val` | `${SET_VAR%%*l}` | `""` |
| **`${SET_VAR:1}`** | `al` | `val` | `${SET_VAR:1}` | `al` |
| **`${SET_VAR:0:2}`** | `va` | `val` | `${SET_VAR:0:2}` | `va` |

---
*Generated by `compare.py`*
