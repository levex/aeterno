# aeterno-master <=> aeterno-sys protocol description

This document outlines and describes in detail the communication protocol used
by the Aeterno init system's `-sys` component and its corresponding `-master`
component.

## Protocol overview

## The `HELO` command

The most basic command - used to retrieve information about the running
`aeterno-sys` instance.

### Example

*connection opened by `MASTER` to `SYS`*
- MASTER -> SYS: `HELO\n`
- SYS -> MASTER: `Aeterno MAJ.MIN.PAT - TEXT`
*connection closed*

### Explanation of replies

Aeterno follows semantic versioning, so `MAJ` refers to a major version. Each
major version bump indicates a non-backwards compatible change. We will try to
maintain backwards compatibility for as long *as it makes sense*.

Similarly, `MIN` and `PAT` refer to minor and patch versions of Aeterno. Each
minor version bump is a backwards-compatible change with _new_ features, while
a `PAT` version bump is a backwards compatible bugfix.

Finally, `TEXT` is a blob (i.e., a hostname) identifying the system in question
via some user-defined value.

## The `START` command

This command is responsible for starting a process. When the `sys` instance
encounters this command, the process is immediately created and started.

This command takes an unlimited number of arguments, as long as the total length 
of the command is less than `CMDLINE_MAX` (as defined by the operating system),  
the execution will be passed _directly_ to the OS. Usually, the argument at
position zero is the actual binary that will be started and the rest of the
arguments (if any) will become the arguments to the binary started.

If starting the process failed, the command returns an Error condition, where
the value of the error is the `errno` returned by the underlying system call to
`execve`. Consult the manual page for `execve(2)` and the corresponding
`errno.h` for details on why the call failed.

If starting the process succeeded, the command returns with an Ok condition,
where the value of this is the process identifier (usually the `pid`) of the
process just started.

### Example:

*connection opened by `MASTER` to `SYS`*
- MASTER -> SYS: `START /bin/echo hello world\n`
- SYS -> MASTER: `OK 1234`
- MASTER -> SYS: `START /bin/does/not/exist\n`
- SYS -> MASTER: `ERR 8`
*connection closed*

