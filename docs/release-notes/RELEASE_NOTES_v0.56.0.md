# TCHLauncher 0.56.0

Local Windows x64 preview. This milestone is not published on GitHub.
The installer is unsigned. GitHub retains the published 0.54 preview.

## Changes since 0.55.0

- The server library uses pages of 50 cards with the existing search, filters,
  sorting, grid/list layouts and actions. Page changes announce their position
  and request result-count focus. Small libraries keep their existing controls.
- Application, scheduled-task and crash-restart event subscriptions retry failed
  registration. Recovery clears notices and reconciles missed state, logs or
  backup/task data through the existing readers.
- Late library reads cannot overwrite newer process events or acknowledged
  creation, rename and deletion. Updated snapshot metadata remains available.
- General actions, settings saves, manual backups, deletion and restore share
  immediate ownership so repeated callbacks cannot send duplicate operations
  before controls update. Installation jobs retain their independent task state.
- Closing a deletion dialog whose opener disappeared requests focus on the
  current page heading. Cancelling preserves normal focus return.
- Heap-only memory charts show heap headline, peak and description instead of
  resident RAM. CPU readings and missing-data states remain unchanged.
- Idle task snapshots reuse unchanged state. Backup latest-copy and reason
  derivation reuses unchanged inventories. Thirty-six superseded sizing/spacing
  declarations are removed while retaining final and conditional CSS rules.

## Validation and limits

Focused actual-component, helper, effect and callback checks cover these changes
with controlled native/read/focus boundaries. A 253-server fixture bounds live
library cards to 50 per page, preserving action identity, filters, sort and layout.
Source checks preserve final unconditional declarations and conditional rules.
Temporary probes add no retained tests or dependencies.

These checks do not establish rendered visual quality, native keyboard/event
delivery, physical-router behavior or a new Minecraft client session. No CPU,
RAM, frame-rate or Internet-latency improvement is claimed. Remaining manual
validation stays maintainer-owned and nonblocking. GitHub retains preview 0.54.

Packaged source: `44aeb3d4ce3b4e3b3a911a0da6d63c525f931856`. Five retained frontend tests, TypeScript,
production frontend, optimized offline Windows NSIS packaging, executable/setup
versions and embedded icons pass. The installer is 6,882,989 bytes. Generated-cache
cleanup is deferred while other builds remain active. The 0.55/0.56 installers
and executable keep their verified hashes. The obsolete local 0.54 setup is verified
and removed. Checksums stay local. These counts are file lengths, not allocated
disk space or runtime memory. Native interaction/gameplay is not repeated.
