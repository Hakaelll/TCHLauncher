import { test } from 'node:test'
import assert from 'node:assert/strict'
import { appearanceKey, defaultAppearance, parseAppearance, readAppearance, saveAppearance, applyAppearance } from '../src/appearance.ts'

test('appearance survives a save and a fresh load for every theme and font', () => {
  const values = new Map()
  const storage = { getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value) }
  for (const theme of ['end', 'obsidian', 'nether']) for (const font of ['minecraft', 'reading']) {
    assert.equal(saveAppearance(storage, { theme, font }), true)
    assert.deepEqual(readAppearance(storage), { theme, font })
    const root = { dataset: {} }
    applyAppearance(readAppearance(storage), root)
    assert.deepEqual(root.dataset, { theme, font })
  }
  assert.ok(values.has(appearanceKey))
})
test('corrupt and obsolete preferences fall back to End without blocking startup', () => {
  for (const raw of [null, '', '{bad', 'null', '[]', '17', '"nether"']) assert.deepEqual(parseAppearance(raw), defaultAppearance)
  assert.deepEqual(parseAppearance('{"theme":"green","font":"reading"}'), { theme: 'end', font: 'reading' })
  assert.deepEqual(parseAppearance('{"theme":"nether","font":"unknown"}'), { theme: 'nether', font: 'minecraft' })
})
test('blocked preference storage still allows the selected appearance to be applied', () => {
  const blocked = { getItem() { throw new Error('denied') }, setItem() { throw new Error('quota') } }
  assert.deepEqual(readAppearance(blocked), defaultAppearance)
  assert.equal(saveAppearance(blocked, { theme: 'nether', font: 'reading' }), false)
  assert.equal(saveAppearance(null, defaultAppearance), false)
  const root = { dataset: {} }
  applyAppearance({ theme: 'nether', font: 'reading' }, root)
  assert.deepEqual(root.dataset, { theme: 'nether', font: 'reading' })
})
