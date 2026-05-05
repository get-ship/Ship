// Minimal JSONC parser: strip line + block comments and trailing commas, then JSON.parse.
//
// Pulled inline so the router has zero external dependencies. The codebase
// hasn't picked a JSONC library yet; once it does, this file is a one-line
// re-export.

const enum State {
  Code,
  String,
  LineComment,
  BlockComment,
}

function strip(input: string): string {
  let out = ''
  let state: State = State.Code
  let i = 0
  while (i < input.length) {
    const c = input[i]!
    const next = input[i + 1]
    if (state === State.Code) {
      if (c === '"') {
        out += c
        state = State.String
        i++
        continue
      }
      if (c === '/' && next === '/') {
        state = State.LineComment
        i += 2
        continue
      }
      if (c === '/' && next === '*') {
        state = State.BlockComment
        i += 2
        continue
      }
      out += c
      i++
      continue
    }
    if (state === State.String) {
      out += c
      // Pass over escape sequences whole so an escaped quote inside a string
      // doesn't terminate it (the escaped character can be anything — `\"`,
      // `\\`, `\n`, `A`).
      if (c === '\\' && next !== undefined) {
        out += next
        i += 2
        continue
      }
      if (c === '"') {
        state = State.Code
      }
      i++
      continue
    }
    if (state === State.LineComment) {
      if (c === '\n') {
        out += c
        state = State.Code
      }
      i++
      continue
    }
    if (state === State.BlockComment) {
      if (c === '*' && next === '/') {
        state = State.Code
        i += 2
        continue
      }
      i++
    }
  }
  return out
}

function stripTrailingCommas(input: string): string {
  // Trailing commas in objects and arrays. Walk the string state-aware so
  // commas inside string literals stay put.
  let out = ''
  let inString = false
  let i = 0
  while (i < input.length) {
    const c = input[i]!
    if (inString) {
      out += c
      if (c === '\\' && i + 1 < input.length) {
        out += input[i + 1]
        i += 2
        continue
      }
      if (c === '"') inString = false
      i++
      continue
    }
    if (c === '"') {
      inString = true
      out += c
      i++
      continue
    }
    if (c === ',') {
      // Look ahead past whitespace for `}` or `]`.
      let j = i + 1
      while (j < input.length && /\s/.test(input[j]!)) j++
      const ahead = input[j]
      if (ahead === '}' || ahead === ']') {
        i++
        continue
      }
    }
    out += c
    i++
  }
  return out
}

export function parseJsonc(text: string): unknown {
  const stripped = stripTrailingCommas(strip(text))
  return JSON.parse(stripped)
}
