| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-MGMT-001 | The device shall serve the management UI as static files. | Inspection | 003 | 0.1.0 |
| ENK-MGMT-002 | The management UI shall load nothing from outside the device. | Network capture while using every page | 003 | 0.1.0 |
| ENK-MGMT-003 | The management UI shall work completely without service workers, camera access, `crypto.subtle`, the clipboard API, persistent storage, and Web Bluetooth. | Test on a page served over HTTP | 003 | 0.1.0 |
| ENK-MGMT-004 | The management UI and the daemon shall be released as one unit with one version number, shown in the UI. | Inspection | 003 | 0.1.0 |
| ENK-MGMT-005 | After an update, the UI shown to the operator shall be the one that belongs to the running daemon. A UI from before the update shall not be used. | Test: update, reopen | 003 | 0.1.0 |
| ENK-MGMT-006 | The management API shall not depend on the client being the UI that the device serves. | Test with a separate client | 003 | 0.1.0 |
| ENK-MGMT-007 | The management UI shall be reachable at one IPv4 address, the same on the management and the setup network. The address shall change only when ENK-TOPO-003 forces the subnet to change. | Test | 003 | 0.1.0 |
| ENK-MGMT-008 | The device should also answer under one fixed name on those networks. Whether it does is decided by measurement M-1. | RFC 006 | 003 | 0.1.0 |
| ENK-MGMT-009 | The address shall be given to the operator as text and as a QR code, in the install guide and in the management UI. | Inspection | 003 | 0.1.0 |
