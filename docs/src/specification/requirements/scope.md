| ID | Requirement | Verification | RFC | Release |
|---|---|---|---|---|
| ENK-SCOPE-001 | Enkrnett shall use OpenWrt for all wireless execution, and shall reach hostapd/wpad only through OpenWrt. | Design review | 001 | 0.1.0 |
| ENK-SCOPE-002 | Setup, configuration, and status shall work without any service outside the venue. | Test without Internet | 001 | 0.1.0 |
| ENK-SCOPE-003 | An operator shall be able to install, set up, and run Enkrnett without knowledge of networking, OpenWrt, or Wi-Fi security. | Usability test with non-experts | 001 | 1.0.0 |
| ENK-SCOPE-004 | Whatever Enkrnett tells an operator or a guest about protection shall be accurate and bounded. It shall state what is protected and what is not. | Review of every status and message against the security RFCs | 001 | 0.2.0 |
| ENK-SCOPE-005 | No supported function shall require the operator to use a command line. | Test | 001 | 1.0.0 |
| ENK-SCOPE-006 | When a later RFC amends an accepted RFC, the amended RFC shall receive, in the commit that accepts the amending RFC, an **Amended by** line in its header naming the amending RFC, and a one-line pointer at each amended passage. The amended text itself is rewritten only in the consolidated specification. | Inspection at acceptance | 016 | now (process) |
