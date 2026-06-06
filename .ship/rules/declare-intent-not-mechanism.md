# Declare Intent, Don't Drive Mechanism

The platform layer (L1) declares **what it needs**; the substrate (L0) owns
**how**. L0 is Ship — its ports are the capabilities the system is made of, not
insulation around core utilities.

## The criterion for a valid port
A port exposes a **capability** (the result a caller wants) and owns the
**mechanism** (the procedure). Method names are **domain needs**, not IO verbs.

Read the call site to judge it:
- `config.get()` · `artifacts.store(x)` · `defs.list()` · `sui_compiler.ensure()`
  → **valid** — a request for a result.
- `fs.read(path)` · `fs.list_dir(dir)` · `cmd.spawn("git", …)`
  → **smell** — a procedure against a primitive, *even if the primitive is
  dressed as a "port."*

## Why "raw access" alone was the wrong frame
The defect is not `std::fs` appearing in L1. The defect is L1 being
**imperative when it should be compositional** — encoding the procedure ("open
`<dir>`, read these files, parse, assemble") instead of declaring the result ("I
need the ship configuration"). Raw fs is just the loudest tell.

## Consequence for generic brokers
A generic `FsBroker` / `ExecutionContext` is **fine as an L0-internal
mechanism** and an **instant smell the moment L1 drives it imperatively** with
specific paths/commands — because L1 still owns the procedure. The rule is not
"ban the broker from L1"; it is **"L1 declares results, never procedures."**

## Naming follows intent, not nature
Group capabilities by what callers want, not by what things are. There is no
"toolchain" bucket — a view that compiles asks for the **sui compiler**. A
future second binary earns its own named capability; it does not retroactively
justify a "bundled binaries" dump. Nature-over-intent grouping is the same
error as a UI-driven service.

## Test before adding/using a port
1. Does the method name a domain need, or an IO/process verb?
2. Does the caller receive a result, or orchestrate a procedure?
3. If L1 imports a primitive (`fs`, `process`, a broker) — what *result* did it
   actually want? Move the procedure behind a capability that owns it.
