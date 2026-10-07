# Catalog visual refinement after 0.28.0

## Changes

- Rebuilt content category selection as four raised Minecraft inventory controls,
  with distinct pixel SVG marks, short role labels and an explicit pressed state.
- Grouped search and existing category, order and version filters in one workbench.
- Enlarged actual project artwork and separated project information from the
  compatibility and installation actions; removed the thick status stripe.
- Added static loading placeholders, responsive result columns and a quieter
  pagination layout. No new dependency, background timer or fabricated data.
- Localized result totals, author labels and release links in Spanish and English.
- Kept search state, compatibility checks, native install review, project links
  and mrpack import connected to the existing implementation.

## Verification

TypeScript compilation, Vite production compilation and all 84 frontend tests
passed. Native screenshots and mouse/keyboard inspection were not performed,
following the user's manual native review constraint. These source changes do not
alter the existing 0.28.0 installer or publish a release.

Thirty catalog text/theme combinations passed 4.5:1 contrast using the committed
End, Obsidian and Nether tokens; the lowest measured pair was 6.55:1. This checks
token colors, not a screenshot or rendered-layout audit.

Suggested commit: `style: rebuild Minecraft content discovery layout`
