| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-ISO-001 | A guest client shall not be able to open a connection to any address in the site network, in any private, link-local, shared, or unique-local range, or in the uplink's own subnet. | Test from a guest client | 002 | 0.2.0 |
| ENK-ISO-002 | The device shall not forward traffic between guest clients: within one BSS, between BSSes, between radios, and at IP level. | Test from two guest clients in each combination | 002 | 0.2.0 |
| ENK-ISO-003 | A guest client shall reach no service on the device other than DHCP and DNS. | Port scan from a guest client | 002 | 0.2.0 |
| ENK-ISO-004 | A management client shall not be able to open a connection to the site network, to a guest client, or to another management client. | Test from a management client | 002 | 0.2.0 |
| ENK-ISO-005 | The device shall accept no connection from the uplink side. | Port scan from the site network | 002 | 0.2.0 |
| ENK-ISO-006 | Guest and management networks shall be separate bridges. | Inspection | 002 | 0.2.0 |
