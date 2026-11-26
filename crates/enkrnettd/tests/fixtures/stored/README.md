# Stored data of every earlier release (ENK-BASE-005)

What each tagged release wrote to its state directory, kept so that the current daemon is
tested against the real documents of its predecessors (`crates/enkrnettd/tests/stored_data.rs`),
not against hand-written ones.

| Directory | Made from | How |
|---|---|---|
| `0.1.0/` | tag `0.1.0` (`6d87e0f`, M1 stage 5 commit `f089392`) | The tag's daemon, `enkrnettd --mock --act-delay-ms 0`, walked by the tag's own `crates/enkrnettd/examples/claim-flow.sh` with the password `correct horse battery staple`: the claim (guest *Cafe Example*, `open_to_all`, JP), then the settings change to `encrypted_only`; the state directory copied after the change settled |
| `0.2.0/` | tag `0.2.0` (`d088d40`, commit `88b5fc0`) | The same, with the tag's daemon and script |

Both hold one file, `state.json`, format 1. The daemon of the tag wrote no other file: the pending
marker is removed when the change settles, and the update marker arrived with M3. Nothing in a
document was edited; the owner's password hash uses the mock's argon2 parameters of the time
(8 MiB, 3 iterations), which the test's daemon verifies as a device would.

A new release adds a directory here when it is tagged (release checklist, `docs/src/contributors/releasing.md`).
