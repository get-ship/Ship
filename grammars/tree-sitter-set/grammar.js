// Tree-sitter grammar for `.set` single-file components.
//
// Strategy: a thin outer grammar identifies the three top-level blocks
// (`<script>`, `<template>`, `<style>`) and the framework-specific
// declarations inside `<script>` (prop / state / derived / data /
// effect / event / mount / unmount / function / component / `on form_*`).
// Anything that isn't a recognized framework keyword is captured as an
// opaque `js_chunk` (script) or as opaque text/braced bodies in
// `<template>`. We layer language injection
// (tree-sitter-typescript / tree-sitter-html / tree-sitter-css) for
// those opaque ranges via `queries/injections.scm` so this grammar
// stays small and authoritative for the framework surface, not the
// host languages.
//
// Conformance target: every `.set` in `packages/ship-set-sail/components/`
// parses without ERROR nodes. Validation lives in `test/corpus/`.

export default grammar({
  name: 'set',

  extras: $ => [/\s/, $.line_comment, $.block_comment],

  conflicts: $ => [],

  externals: $ => [],

  word: $ => $.identifier,

  rules: {
    source_file: $ => seq(
      optional($.script_block),
      optional($.template_block),
      optional($.style_block),
    ),

    // ── <script> ──────────────────────────────────────────────────
    script_block: $ => seq(
      $.script_open,
      repeat($._script_item),
      $.script_close,
    ),
    script_open: $ => /<script(?:\s+lang\s*=\s*"(?:js|ts)")?\s*>/,
    script_close: $ => /<\/script>/,

    _script_item: $ => choice(
      $.component_decl,
      $.prop_decl,
      $.state_decl,
      $.derived_decl,
      $.data_decl,
      $.effect_decl,
      $.event_decl,
      $.mount_block,
      $.unmount_block,
      $.form_lifecycle_block,
      $.function_decl,
      $.js_chunk,
    ),

    // `component formAssociated` — currently the only flag.
    component_decl: $ => seq('component', 'formAssociated'),

    // `prop name [?] [: type] [!] [= default]`
    // The `?` mirrors TS optional syntax; the parser treats it the same
    // as the absence of a default (no required marker change).
    prop_decl: $ => seq(
      'prop',
      field('name', $.identifier),
      optional('?'),
      optional(seq(':', field('type', $.type_annotation))),
      optional($._required_marker),
      optional(seq('=', field('default', $.expression))),
    ),
    _required_marker: $ => '!',

    state_decl: $ => seq(
      'state',
      field('name', $.identifier),
      optional(seq(':', field('type', $.type_annotation))),
      '=',
      field('initial', $.expression),
    ),

    derived_decl: $ => seq(
      'derived',
      field('name', $.identifier),
      '=',
      field('expr', $.expression),
    ),

    data_decl: $ => seq(
      'data',
      field('name', $.identifier),
      optional(seq(':', field('type', $.type_annotation))),
      '=',
      field('expr', $.expression),
    ),

    effect_decl: $ => seq(
      'effect',
      optional(choice(
        seq('on', field('deps', $.dep_list)),
        field('once', 'once'),
      )),
      field('body', $.brace_block),
    ),
    // Deps may be dotted (e.g. `effect on props.sessionId, draft { … }`),
    // matching the script-side scanner that splits on commas only at
    // the top level.
    dep_list: $ => seq($.dotted_identifier, repeat(seq(',', $.dotted_identifier))),
    dotted_identifier: $ => seq($.identifier, repeat(seq('.', $.identifier))),

    // `event NAME` or `event NAME : payload_type` — colon form, matching
    // the script scanner. Payload type uses the same balanced-token
    // shape as `state`/`prop` types so inline object types like
    // `{ id: string, item: Item }` land in one annotation node.
    event_decl: $ => seq(
      'event',
      field('name', $.identifier),
      optional(seq(':', field('payload_type', $.type_annotation))),
    ),

    mount_block: $ => seq('mount', field('body', $.brace_block)),
    unmount_block: $ => seq('unmount', field('body', $.brace_block)),

    form_lifecycle_block: $ => seq(
      'on',
      field('event', choice('form_reset', 'form_restore', 'form_disabled')),
      optional(seq('(', field('payload', $.identifier), ')')),
      field('body', $.brace_block),
    ),

    function_decl: $ => seq(
      optional('async'),
      'function',
      field('name', $.identifier),
      field('params', $.paren_block),
      field('body', $.brace_block),
    ),

    // ── <template> ────────────────────────────────────────────────
    template_block: $ => seq(
      $.template_open,
      repeat($._template_node),
      $.template_close,
    ),
    template_open: $ => /<template\s*>/,
    template_close: $ => /<\/template>/,

    _template_node: $ => choice(
      $.html_comment,
      $.host_element,
      $.if_block,
      $.for_block,
      $.await_block,
      $.element,
      $.interpolation,
      $.raw_html,
      $.text,
    ),

    // `<!-- ... -->` — single token so its content is opaque to the
    // parser (no nested-element parsing inside comments).
    html_comment: $ => token(seq('<!--', /[\s\S]*?-->/)),

    host_element: $ => seq(
      '<:host',
      repeat($._attribute),
      choice('/>', seq('>', '</:host>')),
    ),

    element: $ => choice(
      $._self_closing_element,
      $._regular_element,
    ),
    _self_closing_element: $ => seq(
      '<',
      field('tag', $.tag_name),
      repeat($._attribute),
      '/>',
    ),
    _regular_element: $ => seq(
      '<',
      field('open_tag', $.tag_name),
      repeat($._attribute),
      '>',
      repeat($._template_node),
      '</',
      field('close_tag', $.tag_name),
      '>',
    ),
    tag_name: $ => /[a-zA-Z][a-zA-Z0-9_:-]*/,

    _attribute: $ => choice(
      $.bind_attr,
      $.event_attr,
      $.dynamic_attr,
      $.static_attr,
      $.boolean_attr,
    ),
    bind_attr: $ => prec(2, seq(
      'bind:',
      field('name', $.attr_name),
      '=',
      field('value', $.brace_expression),
    )),
    event_attr: $ => prec(2, seq(
      choice(
        seq('on:', field('event', $.attr_name)),
        field('event_dom', $.on_attr_name),
      ),
      repeat(seq('|', field('modifier', $.event_modifier))),
      '=',
      field('handler', $.brace_expression),
    )),
    on_attr_name: $ => /on[a-z][a-z0-9-]*/,
    event_modifier: $ => seq(
      $.identifier,
      optional(seq('.', $.identifier)),
    ),
    dynamic_attr: $ => seq(
      field('name', $.attr_name),
      '=',
      field('value', $.brace_expression),
    ),
    static_attr: $ => seq(
      field('name', $.attr_name),
      '=',
      field('value', $.string_literal),
    ),
    boolean_attr: $ => prec(-1, $.attr_name),
    // Attribute names exclude `:` so the directive prefixes `bind:`
    // and `on:` are matched as their own keyword tokens (otherwise
    // `bind:value` would be a single 10-char attr_name and the
    // bind_attr rule would never fire). Real corpus has no SVG-style
    // namespaced attributes (`xlink:href`, etc.) so the carve-out is
    // safe.
    attr_name: $ => /[a-zA-Z_][a-zA-Z0-9_.-]*/,
    string_literal: $ => choice(
      seq('"', /[^"]*/, '"'),
      seq("'", /[^']*/, "'"),
    ),

    interpolation: $ => seq('{', optional($._brace_body), '}'),
    raw_html: $ => seq('{@html', optional($._brace_body), '}'),
    brace_expression: $ => $.interpolation,

    if_block: $ => seq(
      '{#if',
      field('cond', $._brace_body),
      '}',
      field('then', repeat($._template_node)),
      repeat(field('elif', $.elif_clause)),
      optional(field('else', $.else_clause)),
      '{/if}',
    ),
    elif_clause: $ => seq(
      // Match either `{:elif` or `{:else if` — both spellings appear
      // in the corpus and the script parser accepts the latter.
      choice('{:elif', '{:else if'),
      field('cond', $._brace_body),
      '}',
      repeat($._template_node),
    ),
    else_clause: $ => seq(
      '{:else}',
      repeat($._template_node),
    ),

    for_block: $ => seq(
      '{#for',
      field('item', $.for_pattern),
      'of',
      field('iter', $._brace_body_no_key),
      // `key={EXPR}` — bare assign, not parenthesized. The braces are
      // the value expression.
      optional(seq('key', '=', field('key', $.brace_expression))),
      '}',
      field('body', repeat($._template_node)),
      optional(field('empty', $.for_empty)),
      '{/for}',
    ),
    for_pattern: $ => choice(
      // `(item, idx)` — parenthesised tuple
      seq('(', $.identifier, repeat(seq(',', $.identifier)), ')'),
      // `item, idx` — unparenthesised tuple, the spelling used in the
      // corpus (matches `parse_for_header` in `parse/template.rs`).
      seq($.identifier, repeat(seq(',', $.identifier))),
    ),
    for_empty: $ => seq(
      '{:empty}',
      repeat($._template_node),
    ),

    await_block: $ => seq(
      '{#await',
      field('promise', $._brace_body),
      '}',
      field('loading', repeat($._template_node)),
      optional(field('then', $.await_then)),
      optional(field('catch', $.await_catch)),
      '{/await}',
    ),
    await_then: $ => seq(
      '{:then',
      field('binding', $.identifier),
      '}',
      repeat($._template_node),
    ),
    await_catch: $ => seq(
      '{:catch',
      field('binding', $.identifier),
      '}',
      repeat($._template_node),
    ),

    // ── <style> ────────────────────────────────────────────────────
    style_block: $ => choice(
      $._style_external,
      $._style_inline,
    ),
    _style_external: $ => seq(
      $.style_open,
      '/>',
    ),
    _style_inline: $ => seq(
      $.style_open,
      '>',
      optional($.style_body),
      $.style_close,
    ),
    // `global` and `src="..."` may appear in any order; both optional.
    // `global global` and other duplicates aren't intentionally allowed
    // — they were a regex artefact and have been dropped.
    style_open: $ => /<style(?:\s+(?:global|src\s*=\s*"[^"]*"))*/,
    style_close: $ => /<\/style>/,
    // CSS body is opaque; we tokenise non-`<` runs and bare `<`
    // characters separately so selectors / rare HTML-in-CSS embeds
    // (`<svg slot="…">` referenced inside a comment, attribute-selector
    // escapes, etc.) don't break the parse. `style_close` (`</style>`)
    // wins by lexical precedence so the body always terminates cleanly.
    style_body: $ => repeat1(choice(
      $._style_text,
      $._style_lt,
    )),
    _style_text: $ => /[^<]+/,
    _style_lt: $ => '<',

    // ── opaque expression / chunk leaves ──────────────────────────
    // Multi-line aware expression. We model the right-hand side of
    // declarations as a balanced sequence: bare atoms, plus matched
    // `(…)`, `[…]`, `{…}`, and string/template literals. Because top-
    // level `\s` is in `extras`, atoms can span newlines as long as
    // each atom segment is whitespace-separated. The expression ends
    // when the next token is a higher-precedence framework keyword or
    // the end of the script block — matching the script scanner's
    // line-continuation rule for the corpus surface (multi-line
    // ternaries and object literals both round-trip through the
    // balanced delimiters).
    expression: $ => prec.right(repeat1($._expr_part)),

    // Type annotation on `prop`/`state`/`data`/`event`. Same balanced
    // shape as `expression`, but with `=` and `!` excluded from atoms
    // so `prop x: T = y` and `prop x: T!` end at the right place.
    type_annotation: $ => prec.right(repeat1($._type_part)),

    _expr_part: $ => choice(
      // Identifier-shaped tokens go through the lexer's keyword
      // extraction path so framework keywords (`state`, `derived`,
      // …) terminate the expression cleanly when they appear at a
      // statement boundary. Bare punctuation/operators are captured
      // by the punct chunk.
      $.identifier,
      // `.foo` member access — matched as a single token so that
      // `ship.dashboard.state()` doesn't have its `.state` segment
      // mis-tokenised as a `state` keyword (which would erroneously
      // terminate the surrounding expression and start a new
      // declaration).
      $._member_access,
      $._expr_number,
      $._expr_punct,
      $._balanced_paren,
      $._balanced_bracket,
      $._balanced_brace,
      $._balanced_string_d,
      $._balanced_string_s,
      $._balanced_template,
    ),
    _type_part: $ => choice(
      $.identifier,
      $._member_access,
      $._expr_number,
      $._type_punct,
      $._balanced_paren,
      $._balanced_bracket,
      $._balanced_brace,
      $._balanced_string_d,
      $._balanced_string_s,
      $._balanced_template,
    ),

    // Atoms: any run of non-whitespace, non-delimiter characters.
    // String/template starts (`'`/`"`/`` ` ``) are excluded so they
    // route to the dedicated string rules.
    // Numeric literals — covers integers, decimals, exponents, and
    // numeric separators (`60_000`). Looser than the JS spec but
    // tight enough to round-trip the corpus.
    _expr_number: $ => token(/[0-9][0-9_.]*(?:[eE][-+]?[0-9_]+)?/),
    // `.foo` segment of a property/method access. Captured as a
    // single token (with `prec(1)` so it wins over a bare `.`
    // punct + `identifier` split) which sidesteps the lexer's
    // keyword-extraction step on the identifier half.
    _member_access: $ => token(prec(1, /\.[a-zA-Z_$][a-zA-Z0-9_$]*/)),
    // Punctuation/operator runs inside expressions. Excludes
    // identifier characters (those go through `$.identifier` so
    // keyword extraction can terminate the expression), digits (those
    // form `_expr_number`), whitespace, and brackets/quotes/backticks.
    // `<` is included so JS comparisons (`page() <= 1`) parse — the
    // closing `</script>` token is longer (9 chars) and wins over the
    // punctuation chunk on length.
    _expr_punct: $ => token(/[^\sa-zA-Z_$0-9{}()\[\]"'`]+/),
    // Type punctuation additionally excludes `=` and `!`. `=>` (arrow
    // type separator) is admitted as a single multi-character atom
    // so `(id: string) => void` lands in one annotation node.
    _type_punct: $ => token(/(?:=>|[^\sa-zA-Z_$0-9={}()\[\]!"'`])+/),

    _balanced_paren: $ => seq('(', repeat($._balanced_part), ')'),
    _balanced_bracket: $ => seq('[', repeat($._balanced_part), ']'),
    _balanced_brace: $ => seq('{', repeat($._balanced_part), '}'),
    _balanced_part: $ => choice(
      $._balanced_atom,
      $._balanced_paren,
      $._balanced_bracket,
      $._balanced_brace,
      $._balanced_string_d,
      $._balanced_string_s,
      $._balanced_template,
    ),
    // `<` is allowed inside balanced contexts so JS comparisons
    // (`if (a < b)`) and generic-style angle brackets round-trip
    // through function/effect/mount bodies. The outer rules that
    // rely on `</script>` / `</template>` / `</style>` recognition
    // run at the top level, where `_balanced_atom` is not in scope.
    // Whitespace is excluded so the lexer hands control back to the
    // extras matcher between atoms — without this, an atom can
    // greedily span newlines and swallow `// comment` runs that
    // happen to be followed by an apostrophe (turning the line into
    // a phantom string literal).
    _balanced_atom: $ => token(prec(1, /[^\s{}()\[\]"'`]+/)),
    _balanced_string_d: $ => token(/"(?:[^"\\]|\\.)*"/),
    _balanced_string_s: $ => token(/'(?:[^'\\]|\\.)*'/),
    // Template literals: opaque body up to the matching backtick. We
    // accept `${` substitutions whose payload may contain backticks
    // by scanning until a non-escaped backtick at depth 0 — the
    // common-case forms (`${expr}` with no nested template) round
    // trip through a single token.
    _balanced_template: $ => token(/`(?:[^`\\]|\\.)*`/),

    // Inside template `{…}` interpolations and control-flow headers we
    // capture the body as the same balanced sequence so object literals
    // (`{ id: x }`) and template-literal substitutions
    // (`` `/p/${id}` ``) survive the tokenisation.
    _brace_body: $ => prec.right(repeat1($._balanced_part)),
    // `{#for ITEM of LIST key={…}}` — the iterator must NOT swallow the
    // trailing ` key=` segment. We use a stricter atom that excludes
    // the `=` sign at top level so the `key=` token appears separately.
    // (Object-literal `=` is wrapped in braces, which is captured by
    // `_balanced_brace` instead.)
    _brace_body_no_key: $ => prec.right(repeat1($._iter_part)),
    _iter_part: $ => choice(
      // Split into identifier / number / punct so `key` (the literal
      // that introduces `key={…}`) can win over the generic atom
      // through keyword extraction.
      $.identifier,
      $._member_access,
      $._expr_number,
      $._iter_punct,
      $._balanced_paren,
      $._balanced_bracket,
      $._balanced_brace,
      $._balanced_string_d,
      $._balanced_string_s,
      $._balanced_template,
    ),
    _iter_punct: $ => token(/[^=\sa-zA-Z_$0-9{}()\[\]"'`]+/),

    // Balanced-brace block for function/effect/mount bodies.
    brace_block: $ => seq(
      '{',
      repeat($._balanced_part),
      '}',
    ),
    paren_block: $ => seq(
      '(',
      repeat($._balanced_part),
      ')',
    ),

    // Pass-through for top-level script lines that aren't a recognised
    // declaration (imports, `const` helpers, type aliases, `// …` notes
    // already in `extras`, etc.). Lower lexer precedence than every
    // framework keyword token so `state count = 0` parses as a
    // `state_decl`, not as a single `js_chunk`.
    js_chunk: $ => token(prec(-3, /[^\n<]+\n?/)),

    // ── leaves ────────────────────────────────────────────────────
    text: $ => /[^<{]+/,
    identifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/,
    // Comments need a precedence higher than the balanced/expr atom
    // tokens so they win on equal-length lexer ties — otherwise a `//`
    // run that doesn't include any delimiter character can be eaten by
    // an atom and a stray backtick later on the line opens a phantom
    // template literal.
    line_comment: $ => token(prec(2, /\/\/[^\n]*/)),
    block_comment: $ => token(prec(2, /\/\*[\s\S]*?\*\//)),
  },
});
