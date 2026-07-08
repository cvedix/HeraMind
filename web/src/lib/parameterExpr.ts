/**
 * Minimal expression evaluator for `ParameterDefinition.visible_when`.
 *
 * Supports the 90% case grammar:
 *   - Comparisons: == !=  >  <  >=  <=
 *   - Logic:       &&  ||
 *   - Literals:    number / "string" / 'string' / true / false / null
 *   - Identifiers: parameter names, resolved against the current values map
 *
 * Non-goals (intentionally unsupported):
 *   - Arithmetic (+, -, *, /)
 *   - Function calls
 *   - Nested parentheses beyond a single level
 *
 * The grammar is deliberately tiny — it exists only to make
 * `visible_when: "mode == 'advanced'"` work without pulling in a full
 * expression library. Anything more complex should be encoded as a
 * real field on the schema rather than an expression string.
 */

/** Token types produced by the lexer. */
type TokenType =
  | 'number'
  | 'string'
  | 'boolean'
  | 'null'
  | 'ident'
  | 'op' // == != > < >= <=
  | 'and' // &&
  | 'or' // ||
  | 'lparen'
  | 'rparen'

interface Token {
  type: TokenType
  value: string
}

const TWO_CHAR_OPS = new Set(['==', '!=', '>=', '<=', '&&', '||'])

/** Lex an expression string into tokens. Throws on unrecognized chars. */
function lex(input: string): Token[] {
  const tokens: Token[] = []
  let i = 0
  const n = input.length

  while (i < n) {
    const ch = input[i]

    // Whitespace
    if (/\s/.test(ch)) {
      i++
      continue
    }

    // Two-char operators
    const two = input.slice(i, i + 2)
    if (TWO_CHAR_OPS.has(two)) {
      tokens.push({ type: two === '&&' || two === '||' ? (two as 'and' | 'or') : 'op', value: two })
      i += 2
      continue
    }

    // Single-char operators
    if (ch === '>' || ch === '<') {
      tokens.push({ type: 'op', value: ch })
      i++
      continue
    }

    if (ch === '(') {
      tokens.push({ type: 'lparen', value: ch })
      i++
      continue
    }
    if (ch === ')') {
      tokens.push({ type: 'rparen', value: ch })
      i++
      continue
    }

    // Number literal (negative numbers handled via leading '-' only if not after operand)
    if (/[0-9]/.test(ch) || (ch === '-' && /[0-9]/.test(input[i + 1] ?? ''))) {
      let num = ''
      // leading sign only at start of number position
      if (ch === '-') {
        num += '-'
        i++
      }
      while (i < n && /[0-9.]/.test(input[i])) {
        num += input[i]
        i++
      }
      tokens.push({ type: 'number', value: num })
      continue
    }

    // String literal (double or single quoted)
    if (ch === '"' || ch === "'") {
      const quote = ch
      i++
      let str = ''
      while (i < n && input[i] !== quote) {
        str += input[i]
        i++
      }
      // skip closing quote
      if (i < n) i++
      tokens.push({ type: 'string', value: str })
      continue
    }

    // Identifier (letter / underscore / digit / dot / underscore)
    if (/[a-zA-Z_]/.test(ch)) {
      let ident = ''
      while (i < n && /[a-zA-Z0-9_.]/.test(input[i])) {
        ident += input[i]
        i++
      }
      const lower = ident.toLowerCase()
      if (lower === 'true' || lower === 'false') {
        tokens.push({ type: 'boolean', value: lower })
      } else if (lower === 'null') {
        tokens.push({ type: 'null', value: lower })
      } else {
        tokens.push({ type: 'ident', value: ident })
      }
      continue
    }

    throw new Error(`parameterExpr: unexpected character '${ch}' at position ${i}`)
  }

  return tokens
}

/** Recursive-descent parser producing a predicate function. */
class Parser {
  private pos = 0
  constructor(private readonly tokens: Token[]) {}

  parse(): (values: Record<string, unknown>) => boolean {
    return this.parseOr()
  }

  private peek(): Token | undefined {
    return this.tokens[this.pos]
  }

  private consume(): Token {
    const tok = this.tokens[this.pos]
    if (!tok) throw new Error('parameterExpr: unexpected end of expression')
    this.pos++
    return tok
  }

  // or := and ( '||' and )*
  private parseOr(): (values: Record<string, unknown>) => boolean {
    const left = this.parseAnd()
    if (this.peek()?.type !== 'or') return left
    const rights: Array<(v: Record<string, unknown>) => boolean> = [left]
    while (this.peek()?.type === 'or') {
      this.consume()
      rights.push(this.parseAnd())
    }
    return (v) => rights.some((fn) => fn(v))
  }

  // and := comparison ( '&&' comparison )*
  private parseAnd(): (values: Record<string, unknown>) => boolean {
    const left = this.parseComparison()
    if (this.peek()?.type !== 'and') return left
    const rights: Array<(v: Record<string, unknown>) => boolean> = [left]
    while (this.peek()?.type === 'and') {
      this.consume()
      rights.push(this.parseComparison())
    }
    return (v) => rights.every((fn) => fn(v))
  }

  // comparison := primary ( op primary )?   — non-associative, at most one operator
  private parseComparison(): (values: Record<string, unknown>) => boolean {
    if (this.peek()?.type === 'lparen') {
      this.consume()
      const inner = this.parseOr()
      if (this.peek()?.type !== 'rparen') {
        throw new Error('parameterExpr: expected closing parenthesis')
      }
      this.consume()
      return inner
    }

    const left = this.parsePrimary()
    const op = this.peek()
    if (!op || op.type !== 'op') {
      // truthy check on the primary itself
      return (v) => isTruthy(left(v))
    }
    this.consume()
    const right = this.parsePrimary()
    return (v) => compare(left(v), op.value, right(v))
  }

  // primary := number | string | boolean | null | ident
  private parsePrimary(): (values: Record<string, unknown>) => unknown {
    const tok = this.consume()
    switch (tok.type) {
      case 'number':
        const num = parseFloat(tok.value)
        return () => num
      case 'string':
        const str = tok.value
        return () => str
      case 'boolean':
        const b = tok.value === 'true'
        return () => b
      case 'null':
        return () => null
      case 'ident':
        const name = tok.value
        return (v) => resolveIdent(v, name)
      default:
        throw new Error(`parameterExpr: unexpected token '${tok.value}'`)
    }
  }
}

/** Resolve an identifier from the current parameter values, chasing dot paths. */
function resolveIdent(values: Record<string, unknown>, name: string): unknown {
  if (Object.prototype.hasOwnProperty.call(values, name)) {
    return values[name]
  }
  // Allow dotted path like `params.mode`
  if (name.includes('.')) {
    const parts = name.split('.')
    let cur: unknown = values
    for (const part of parts) {
      if (cur && typeof cur === 'object' && part in (cur as Record<string, unknown>)) {
        cur = (cur as Record<string, unknown>)[part]
      } else {
        return undefined
      }
    }
    return cur
  }
  return undefined
}

/** JS-like truthiness: false / 0 / '' / null / undefined → false. */
function isTruthy(v: unknown): boolean {
  if (v === null || v === undefined) return false
  if (typeof v === 'boolean') return v
  if (typeof v === 'number') return v !== 0
  if (typeof v === 'string') return v.length > 0 && v !== 'false'
  return true
}

/** Apply a comparison operator, coercing types loosely. */
function compare(left: unknown, op: string, right: unknown): boolean {
  switch (op) {
    case '==':
      return looseEqual(left, right)
    case '!=':
      return !looseEqual(left, right)
    case '>':
      return num(left) > num(right)
    case '<':
      return num(left) < num(right)
    case '>=':
      return num(left) >= num(right)
    case '<=':
      return num(left) <= num(right)
    default:
      throw new Error(`parameterExpr: unknown operator '${op}'`)
  }
}

/** Loose equality — coerces string/number/boolean as JavaScript would. */
function looseEqual(left: unknown, right: unknown): boolean {
  if (left === right) return true
  // null == undefined → true
  if (left == null && right == null) return true
  if (left == null || right == null) return false
  // string vs number: "1" == 1
  if (typeof left === 'string' && typeof right === 'number') {
    const ln = parseFloat(left)
    return !isNaN(ln) && ln === right
  }
  if (typeof right === 'string' && typeof left === 'number') {
    const rn = parseFloat(right)
    return !isNaN(rn) && rn === left
  }
  // boolean coercion: true == "true" / 1
  if (typeof left === 'boolean') {
    return left === isTruthy(right)
  }
  if (typeof right === 'boolean') {
    return right === isTruthy(left)
  }
  return false
}

function num(v: unknown): number {
  if (typeof v === 'number') return v
  if (typeof v === 'string') {
    const n = parseFloat(v)
    return isNaN(n) ? 0 : n
  }
  if (typeof v === 'boolean') return v ? 1 : 0
  return 0
}

const cache = new Map<string, (values: Record<string, unknown>) => boolean>()

/**
 * Evaluate a `visible_when` expression against current parameter values.
 *
 * Returns `true` for empty / malformed expressions so we never hide a
 * parameter due to a typo in the template — failure-open is safer than
 * failure-closed here.
 */
export function evalVisibleWhen(
  expr: string | undefined,
  values: Record<string, unknown>,
): boolean {
  if (!expr || !expr.trim()) return true
  try {
    let fn = cache.get(expr)
    if (!fn) {
      const tokens = lex(expr)
      fn = new Parser(tokens).parse()
      cache.set(expr, fn)
    }
    return fn(values)
  } catch (err) {
    if (import.meta.env.DEV) {
      console.warn('[parameterExpr] failed to evaluate visible_when:', expr, err)
    }
    return true
  }
}
