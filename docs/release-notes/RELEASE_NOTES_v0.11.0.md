# TCHLauncher 0.11.0 - End themes and interface refinement

## Appearance

- Add **Settings** below **Automatic startup** in the server rail. Choose **End**,
  **Obsidian** or **Nether** from a dedicated page with previews. Changes apply
  immediately and are remembered on this computer.
- Make End the default: near-black and dark gray surfaces with violet interaction
  accents. Obsidian offers neutral graphite/silver; Nether offers warm charcoal and
  restrained ember accents. Semantic status and console-category colors stay distinct.
- Add a Minecraft/reading typography preference. Extend locally bundled Monocraft
  to server sections, rail actions, server names, property labels, capacity controls,
  player tabs and disclosures. Long explanations, input, paths and logs remain readable.
- Apply the Minecraft raised-control treatment to the server section bar. Resting
  actions stay quiet, while hover/focus brighten the selected theme's accents.
- Align collapsed rail actions to consistent square hit areas and centered icons.
  Expanded labels, server tooltips, context menus and pin/delete workflows remain available.

## Library and runtime

- Replace the square black logo backdrop with a transparent exterior cutout while
  retaining the dark central emblem. Regenerate application/tray icons and favicon.
- Recompose the empty library around a larger create action and two compact import
  actions, with dedicated transparent End island, world-map chest and modpack artwork.
  Creation, folder import and `.mrpack` import keep their existing native workflows.
- Rebuild the Java panel with proper padding, clearer runtime identity and real
  compatibility status. Keep installation/reinstallation and recheck actions visible.
  Move the complete executable location into its own disclosure and retain verified
  custom-runtime selection with confirmation.
- Keep theme changes local to appearance preferences. They do not rewrite server
  files, runtime selection, EULA consent, authentication settings or player identities.

## Verification

- TypeScript and production Vite compilation passed.
- 16 frontend tests passed, including all six theme/font save-and-reload combinations,
  corrupt preference recovery and denied storage handling.
- Offline Rust suite: 110 passed, 18 external-resource tests intentionally excluded.
- 160 palette and log-label text combinations passed 4.5:1 contrast; minimum 4.64:1.
- Production logo and inventory illustrations verified as 256px PNGs with transparency.
- Existing reduced-motion behavior, native radio keyboard semantics and focus outlines
  are retained. Asset sources, exact prompts and generation mode are documented in
  [the asset README](../../src/assets/README.md).
- Windows NSIS x64 installer generated from this worktree; SHA-256 stored beside it.

## Remaining checks

Native appearance and keyboard review remain manual by the user's request. No
computer-use tools, browser automation or subagents were used. Minecraft client
gameplay, Linux packaging, public-network automation and reliable TPS telemetry
remain outside this preview's verification. The broader original specification is
still in development; see [NEXT_TODOS.md](../../NEXT_TODOS.md).

## Windows package

- Installer: `TCHLauncher_0.11.0_x64-setup.exe` (5.54 MiB, 5,810,681 bytes).
- Native application product/file version: `0.11.0`.
- SHA-256: `08d692d3ea0076ffb199b51d95ae2746e2bbd6655caecdf1c6830ea9f4c7f920`.
