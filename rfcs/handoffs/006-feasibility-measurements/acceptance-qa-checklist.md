# Acceptance checklist — RFC 006 Feasibility measurements

**Entry point:** [README.md](./README.md)

A report is accepted when every row applies.

## Every report

| # | Check |
|---|---|
| 1 | Names RFC 006 and this handoff. |
| 2 | Follows [report-template.md](./report-template.md). |
| 3 | States the question exactly as RFC 006 states it. |
| 4 | States the versions of OpenWrt, of every installed package, of every tool, and of every device, phone, and browser involved. |
| 5 | Describes the setup so that another person can repeat the measurement. |
| 6 | Reports every step of the procedure, including steps that failed or could not be carried out. |
| 7 | Gives measured values with units. Where a threshold exists, shows it beside the value without judging. |
| 8 | Separates what was observed from what is concluded. |
| 9 | Lists what contradicts an RFC, with the RFC and the requirement named. |
| 10 | Uses the vocabulary of RFC 001. |
| 11 | Contains no credential, no personal data, and no hardware address of a device that is not under test. |
| 12 | Is listed in `docs/src/measurements/index.md` and in `docs/src/SUMMARY.md`. |
| 13 | No code or image written for the measurement was added to the repository. |

## Per measurement

| Measurement | The report contains |
|---|---|
| M-1 | The table of 15 observations for every phone; screenshots of rows 6, 13, 14 |
| M-2 | The table program × (size, size in squashfs, idle memory, memory after load); dependency lists; password check times or the reason for their absence |
| M-3 | Image size beside stock size and limit; free overlay; memory per state; events of the 24-hour run |
| M-4 | The per-client table; results for interface count, isolation of the open BSS, and the five network names |
| M-5 | The list of needed methods; results for confirm, no confirm, and restart inside the timeout; durations without service |
| M-6 | Results of every step; whether the run is unattended; total time |
| M-7 | Per participant and per device results; whether a phone alone was enough; proposed changes to the guide |
| M-8 | The 15 rows with tool and command; captures for every deviation |

## Review outcome

The architect records one of these per report in `docs/src/measurements/index.md`:

| Outcome | Meaning |
|---|---|
| Accepted | The report is complete. What it changes is recorded. |
| Returned | Rows of this checklist are not met. They are named. |
| Superseded | The measurement was repeated. The newer report counts. |
