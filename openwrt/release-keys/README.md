# Release public keys

The public keys that may sign the release index and the images (chapter 11 §2, ENK-RLS-004,
ENK-RLS-005). Every `*.pub` file here goes into the image at `/usr/share/enkrnett/keys/`, and
the daemon accepts an index signed by any of them.

- At most two keys: the **current** key and the **designated next** key. A release signed by the
  next key makes it current; that release ships a further next key (ENK-RLS-005). The key that
  was current before is removed in the same release.
- Keys are made on the signing host with `enkrnett-release keygen` (or OpenBSD `signify -G -n`);
  only the `.pub` file is ever committed. The `.sec` file never leaves the signing host and never
  reaches a CI runner (ENK-RLS-007).
- The file format is signify's: a comment line and one Base64 line. The daemon ignores a file it
  cannot parse and logs `release_key_ignored`.

**State (2026-11-26): no key yet.** The project's release key is the owner's act on the signing
host (RFC 008); until it exists, an image carries no key and every index check answers
"could not be verified". The VM test (`tests/vm/update-check.sh`) installs its own key for the
duration of the test.
