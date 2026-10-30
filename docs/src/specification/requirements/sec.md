| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-SEC-009 | The device shall refuse every request whose `Host` header is not its own address or name. | Test, including a DNS rebinding attempt | 003 | 0.1.0 |
| ENK-SEC-010 | The management API shall accept no credential that a browser attaches by itself. It shall use neither cookies nor HTTP authentication. | Inspection; cross-site request test | 003 | 0.1.0 |
| ENK-SEC-011 | The management API shall send no header that permits cross-origin access, and shall refuse preflight requests. | Test from a page on another origin | 003 | 0.1.0 |
| ENK-SEC-012 | Every response shall forbid framing, forbid content-type guessing, send no referrer, and allow scripts, styles, images, and connections from the device only. | Inspection of response headers | 003 | 0.1.0 |
| ENK-SEC-013 | The management UI shall show every value that does not come from its own code as text. It shall never interpret such a value as markup or script. | Test with hostile values in every field that is displayed | 007 (amended by 007) | 0.1.0 |
| ENK-SEC-014 | The daemon shall pass no value that it received in a request to a command interpreter. | Inspection; test with hostile values | 007 (amended by 007) | 0.1.0 |
| ENK-SEC-015 | A response that contains the management key or a session identifier shall forbid every cache to store it. | Inspection of response headers | 007 (amended by 007) | 0.1.0 |
| ENK-SEC-016 | No log, status, or diagnosis shall contain the owner password, the management key, or a session identifier. | Search of every output after using every function | 007 (amended by 007) | 0.1.0 |
| ENK-SEC-017 | Request line and headers together shall not exceed 8 KiB (431) and a request body shall not exceed 16 KiB (413); the connection is then closed. | Test | 015 | 0.1.0 |
| ENK-SEC-018 | The daemon shall close a connection that has not delivered a complete request within 10 seconds, and shall finish or abandon any response within 30 seconds. | Test with a slow client | 015 | 0.1.0 |
| ENK-SEC-019 | The daemon shall answer every request with `Connection: close` and shall serve at most 4 connections at a time, one per worker thread, with a listen backlog of 16. | Test with 32 concurrent clients; memory within RFC 010 ENK-FOOT-007 | 015 | 0.1.0 |
| ENK-SEC-020 | The daemon shall speak HTTP/1.1 only, and shall accept no chunked request body, no `Expect`, no compression, and no `Transfer-Encoding`. Its JSON parser shall refuse duplicate keys, nesting deeper than 8, and numbers that are not integers within 64 bits. | Test | 015 | 0.1.0 |
| ENK-SEC-021 | The API shall accept no file upload. The update image is fetched by the device from the release index (RFC 013), never pushed by a client. | Inspection | 015 | 0.1.0 |
| ENK-SEC-022 | Response bodies shall not exceed 64 KiB. | Test | 015 | 0.1.0 |
