import { test } from 'node:test'
import assert from 'node:assert/strict'
import { appearanceKey, defaultAppearance, parseAppearance, readAppearance, saveAppearance, applyAppearance } from '../src/appearance.ts'
import { shouldReduceMotion } from '../src/motionPreference.ts'

test('appearance survives a save and a fresh load for every theme and font', () => {
  const values = new Map()
  const storage = { getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value) }
  for (const theme of ['end', 'obsidian', 'nether']) for (const font of ['minecraft', 'reading']) for (const motion of ['system','reduced']) {
    assert.equal(saveAppearance(storage, { theme, font, motion }), true)
    assert.deepEqual(readAppearance(storage), { theme, font, motion })
    const root = { dataset: {} }
    applyAppearance(readAppearance(storage), root)
    assert.deepEqual(root.dataset, { theme, font, motion })
  }
  assert.ok(values.has(appearanceKey))
})
test('corrupt and obsolete preferences fall back to End without blocking startup', () => {
  for (const raw of [null, '', '{bad', 'null', '[]', '17', '"nether"']) assert.deepEqual(parseAppearance(raw), defaultAppearance)
  assert.deepEqual(parseAppearance('{"theme":"green","font":"reading"}'), { theme: 'end', font: 'reading', motion: 'system' })
  assert.deepEqual(parseAppearance('{"theme":"nether","font":"unknown"}'), { theme: 'nether', font: 'minecraft', motion: 'system' })
  assert.deepEqual(parseAppearance('{"theme":"obsidian","font":"reading","motion":"full"}'), {theme:'obsidian',font:'reading',motion:'system'})
})
test('blocked preference storage still allows the selected appearance to be applied', () => {
  const blocked = { getItem() { throw new Error('denied') }, setItem() { throw new Error('quota') } }
  assert.deepEqual(readAppearance(blocked), defaultAppearance)
  assert.equal(saveAppearance(blocked, { theme: 'nether', font: 'reading', motion:'reduced' }), false)
  assert.equal(saveAppearance(null, defaultAppearance), false)
  const root = { dataset: {} }
  applyAppearance({ theme: 'nether', font: 'reading', motion:'reduced' }, root)
  assert.deepEqual(root.dataset, { theme: 'nether', font: 'reading', motion:'reduced' })
})
test('launcher motion preferences cannot override reduced motion requested by the system',()=>{
  for(const preference of [undefined,'system','reduced','invalid'])assert.equal(shouldReduceMotion(preference,true),true)
  assert.equal(shouldReduceMotion('reduced',false),true)
  assert.equal(shouldReduceMotion('system',false),false)
  assert.equal(shouldReduceMotion(undefined,false),false)
})
