import {test} from 'node:test'
import assert from 'node:assert/strict'
import {settingsMatch,propertyAdditionError} from '../src/settingsSearch.ts'

test('property addition refuses existing keys and unsafe native values without changing the draft',()=>{
  const values={'query.port':'25565',motd:'Existing name'}
  assert.equal(propertyAdditionError(' query.port ','12345',values),'La propiedad ya existe. Modifica su valor en la lista.')
  assert.equal(propertyAdditionError('rcon.port','25575',values),null)
  assert.equal(propertyAdditionError('mod_setting-1','',values),null)
  for(const key of ['','a'.repeat(81),'name=value','line\nkey','two words','á'])assert.ok(propertyAdditionError(key,'value',values))
  for(const value of ['a\nb','a\rb','a'.repeat(1025),'😀'.repeat(257)])assert.ok(propertyAdditionError('new',value,values))
  assert.equal(propertyAdditionError('a'.repeat(80),'😀'.repeat(256),values),null)
  assert.deepEqual(values,{'query.port':'25565',motd:'Existing name'})
})
test('settings search matches translated labels, native keys and accents while preserving values',()=>{
  assert.equal(settingsMatch('  generacion ','Generación y dimensiones','level-seed'),true)
  assert.equal(settingsMatch('QUERY.PORT','query.port'),true)
  assert.equal(settingsMatch('difficulty','Game','difficulty','Difficulty'),true)
  assert.equal(settingsMatch('missing','Juego','gamemode'),false)
  assert.equal(settingsMatch('','Juego'),true)
})
