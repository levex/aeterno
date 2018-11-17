# aeterno-master <=> aeterno-sys protocol description

This document outlines and describes in detail the communication protocol used
by the Aeterno init system's `-sys` component and its corresponding `-master`
component.

## The `HELO` command

The most basic command - used to retrieve information about the running
`aeterno-sys` instance.

*connection opened by `MASTER` to `SYS`*
- MASTER -> SYS: `HELO\n`
- SYS -> MASTER: `Aeterno vMAJ.MIN.PAT - RELMONTH RELYEAR`
*connection closed*

### Explanation of replies

Aeterno follows semantic versioning, so `MAJ` refers to a major version. Each
major version bump indicates a non-backwards compatible change. We will try to
maintain backwards compatibility for as long *as it makes sense*.

Similarly, `MIN` and `PAT` refer to minor and patch versions of Aeterno. Each
minor version bump is a backwards-compatible change with _new_ features, while
a `PAT` version bump is a backwards compatible bugfix.

## The `START` command

## The `STOP` command
