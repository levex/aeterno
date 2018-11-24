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

<<<<<<< HEAD
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

### Example

*connection opened by `MASTER` to `SYS`*
- MASTER -> SYS: `START /bin/echo hello world\n`
- SYS -> MASTER: `OK 1234`
- MASTER -> SYS: `START /bin/does/not/exist\n`
- SYS -> MASTER: `ERR 8`
*connection closed*

In this example, the first command will result in a process created with
the executable image being `/bin/echo` and the arguments in “C-Speak” would be:

- `argv[0]` = `/bin/echo`
- `argv[1]` = `hello`
- `argv[2]` = `world`

The second `START` command however will not result in a process being created,
since the underlying `execve(2)` system call returned an error.

### Explanation of replies

If starting the process failed, the command returns an Error condition, where
the value of the error is the `errno` returned by the underlying system call to
`execve`. Consult the manual page for `execve(2)` and the corresponding
`errno.h` for details on why the call failed.

If starting the process succeeded, the command returns with an Ok condition,
where the value of this is the process identifier (usually the `pid`) of the
process just started.

## The `STOP` command

This command is responsible for gracefully stopping a process. When the `sys`
instance receives this command, it sends an OS-defined signal that is equivalent
to gracefully stopping the process. On Linux, this is `SIGTERM` as it gives the
process a chance to clean up. This is in contrast with the `FORCESTOP` command,
that corresponds to `SIGKILL` and does *not* give the target process a chance
to clean up.

The `STOP` command takes one pk

### Example

*connection opened by `MASTER` to `SYS`*
- MASTER -> SYS: `STOP 1234`
- SYS -> MASTER: `OK 0`
- MASTER -> SYS: `STOP 0`
- SYS -> MASTER: `ERR 7`
*connection closed*

### Explanation of replies

In the example, the first `STOP` command succeeds as shown by receiving an Ok
condition with value `0`. This is because we assume that the process with
identifier `1234` exists and the underlying `kill(2)` system call has succeeded.

The second `STOP` command does not succeed, as no process with the identifier
`0` exists. In this case, an Error condition is returned, with the value being
`7`. Corresponding to `ESRCH / No such process` on the author’s system.
