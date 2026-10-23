# TCHLauncher 0.35.0

## Import and player identities

- Preview detected world folders, mod/plugin files, safe settings and access-file
  counts before importing an existing server. Lists are paginated; incomplete
  inspection is explicit. Original files and private permission records remain
  unchanged. Inventory collection is bounded and runs only for this preview.
- Import standard Minecraft whitelist JSON while stopped, preserving UUIDs and
  existing records. Accounts sharing a name remain separate. Changed lists retain
  an original-byte safety copy; no-op imports do not write files.
- TXT name lists use the running Minecraft session to resolve accounts. A matching
  cached name never removes a request. The review shows UUIDs/profile heads when
  available and distinguishes saved identities from submitted commands.
- Keep completed import results when a later refresh fails, with no repeated write.

## Worlds and recovery

- Treat Paper's overworld and separate Nether/End folders as one family for world
  listing, duplication, folder/ZIP export, activation and protected deletion.
- Recover interrupted family publication using trusted library records, ownership
  markers and streamed fingerprints. Conflicting or modified files are preserved
  for review; disconnected external destinations do not stop unrelated servers.
- A termination before the durable publication record may leave hidden staging
  folders. Interruption checks simulate unwinding, not an actual OS kill or
  power-loss event. Reviewed external Paper-family world import is still pending.

## Tasks, diagnostics and maintenance

- Retain last-known scheduled tasks after a failed read and block edits until a
  successful read-only retry. Translate task status, results and timing labels.
- Distinguish an absent crash report from a failed or pending read. Retry current
  and historical report reads without repeating server actions; translate diagnoses
  while preserving original technical evidence.
- Retire most tests at the maintainer's request and remove unused test helpers and
  standalone Java probes. The compact suite retains 24 cases: ten native safety
  checks, five frontend read/recovery checks and nine opt-in real-engine scenarios.
  Reduced coverage is an intentional maintenance tradeoff, not an app RAM or
  installer-size optimization. No production dependency was added.

## Verification boundaries

The full original manager objective remains open. Native visual, keyboard and
screen-reader review; clean install/upgrade/uninstall; real client admission and
physical router/Internet-play checks remain outstanding. The friend-hosted RAM,
missing-content, Minehut identity and connection-stall cases still need evidence
from the affected host. Linux and client launching remain deferred.

This is a local unsigned Windows preview. Generating an installer does not
install the application or publish a GitHub release.

## Windows package

Installer: `TCHLauncher_0.35.0_x64-setup.exe`, 6,787,166 bytes / 6.47 MiB.
Executable: 24,374,784 bytes. Both report version 0.35.0 and both embedded icons
match the supplied logo. The generated NSIS script includes all four legal
resources. The license inventory has 342 locked Cargo packages, four frontend
runtime packages and three asset/native/installer entries, with no missing local
texts. Only the application's own version changed in Cargo.lock.

TypeScript, five frontend checks, ten native checks and the optimized Windows
NSIS build passed. Nine real Minecraft integrations were not rerun. Initial
JavaScript is 469.35 kB and CSS is 218.61 kB; these are bundle measurements, not
RAM or network performance evidence. The optimized native build took 6m 31s after
cache cleanup; removing compiler caches trades disk space for rebuild time.

SHA-256: `9b6f544369bc801eed5f052f1f19a5bb0eb7cfd499a5e6e8f40b18e72fcce3c2`.

Adjacent checksum files were generated. Installer 0.34.0 remains as fallback;
obsolete 0.33.0 installer/checksum files were removed. After verification,
3,962,938,326 bytes of regenerable compiler caches and 18,806,610 bytes of obsolete
installer/debug artifacts were removed. The executable and both retained
installer hashes were unchanged by cleanup. This concerns workspace build
storage; it does not shrink the installed application or user server data.
