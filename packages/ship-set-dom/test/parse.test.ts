import { describe, expect, it } from 'vitest'
import { parseSetSource } from '../src/parse'

describe('parseSetSource — directives', () => {
  it('extracts a <state> declaration and removes it from the body', () => {
    const parsed = parseSetSource(
      '<state name="count" value="0" /><div>x</div>',
    )
    expect(parsed.state).toHaveLength(1)
    expect(parsed.state[0]!.name).toBe('count')
    expect(parsed.state[0]!.value({})).toBe(0)

    // body retains only the <div>
    const tags = parsed.templateNodes
      .filter((n) => n.nodeType === 1)
      .map((n) => (n as Element).tagName.toLowerCase())
    expect(tags).toEqual(['div'])
  })

  it('extracts a <data> declaration with live flag', () => {
    const parsed = parseSetSource(
      '<data name="agents" source="ship.agents.summaries()" live /><div/>',
    )
    expect(parsed.data).toHaveLength(1)
    expect(parsed.data[0]!.name).toBe('agents')
    expect(parsed.data[0]!.live).toBe(true)
  })

  it('extracts a <component> declaration with <prop> children', () => {
    const parsed = parseSetSource(`
      <component name="ship-thing">
        <prop name="tone" values="default|primary" default="default" reflect />
        <prop name="loading" type="bool" default="false" />
      </component>
      <button>{label}</button>
    `)
    expect(parsed.component).not.toBeNull()
    expect(parsed.component!.tag).toBe('ship-thing')
    expect(parsed.component!.props).toHaveLength(2)
    expect(parsed.component!.props[0]!.type).toBe('union')
    expect(parsed.component!.props[1]!.type).toBe('bool')
  })

  it('throws on missing required attribute', () => {
    expect(() => parseSetSource('<state value="0" />')).toThrow(/name/)
    expect(() => parseSetSource('<data live />')).toThrow(/name/)
  })

  it('throws on duplicate <component>', () => {
    expect(() =>
      parseSetSource('<component name="a"></component><component name="b"></component>'),
    ).toThrow(/only one/)
  })
})
