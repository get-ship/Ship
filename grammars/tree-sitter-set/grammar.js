// Tree-sitter grammar for `.set` single-file components.
//
// Strategy: a thin outer grammar identifies the three top-level blocks
// (`<script>`, `<template>`, `<style>`) and the framework-specific
// declarations inside `<script>` (prop/state/derived/data/effect/event/
// mount/unmount/function/component/on form_*). Anything that isn't a
// recognized framework keyword is captured as an opaque `js_chunk`
// (script) or `template_node` (template) — we'll layer language injection
// (tree-sitter-typescript / tree-sitter-html / tree-sitter-css) for the
// chunks in a follow-up so our grammar stays small and authoritative
// for the framework surface, not the host languages.
//
// Conformance target: every `.set` in `packages/ship-set-sail/components/`
// parses without ERROR nodes. Validation lives in `test/corpus/`.

module.exports = grammar({
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
    script_open: $ => /<script(\s+lang\s*=\s*"(?:js|ts)")?\s*>/,
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

    component_decl: $ => seq(
      'component',
      'formAssociated',
    ),

    prop_decl: $ => seq(
      'prop',
      field('name', $.identifier),
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
    dep_list: $ => seq($.identifier, repeat(seq(',', $.identifier))),

    event_decl: $ => seq(
      'event',
      field('name', $.identifier),
      '(',
      optional(seq(field('payload', $.identifier), ':', field('payload_type', $.type_annotation))),
      ')',
    ),

    mount_block: $ => seq('mount', field('body', $.brace_block)),
    unmount_block: $ => seq('unmount', field('body', $.brace_block)),

    form_lifecycle_block: $ => seq(
      'on',
      field('event', choice('form_reset', 'form_restore', 'form_disabled')),
      optional(seq(
        '(',
        field('payload', $.identifier),
        ')',
      )),
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
      $.host_element,
      $.if_block,
      $.for_block,
      $.await_block,
      $.element,
      $.interpolation,
      $.raw_html,
      $.text,
    ),

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
    attr_name: $ => /[a-zA-Z_:][a-zA-Z0-9_:.-]*/,
    string_literal: $ => choice(
      seq('"', /[^"]*/, '"'),
      seq("'", /[^']*/, "'"),
    ),

    interpolation: $ => seq('{', $._brace_body, '}'),
    raw_html: $ => seq('{@html', $._brace_body, '}'),
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
      '{:elif',
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
      field('iter', $._brace_body),
      optional(seq('(', field('key', $._brace_body), ')')),
      '}',
      field('body', repeat($._template_node)),
      optional(field('empty', $.for_empty)),
      '{/for}',
    ),
    for_pattern: $ => choice(
      $.identifier,
      seq('(', $.identifier, repeat(seq(',', $.identifier)), ')'),
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
    // The body is opaque CSS until language injection lands.
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
      $.style_body,
      $.style_close,
    ),
    style_open: $ => /<style(?:\s+global)?(?:\s+src\s*=\s*"[^"]*")?(?:\s+global)?/,
    style_close: $ => /<\/style>/,
    // v1: opaque CSS body, terminator is the next `<` character. Real CSS
    // rarely contains `<` outside of attribute selectors (which we can
    // round-trip via `\3c` escape). v2 will use an external C scanner
    // that consumes everything up to `</style>` directly.
    style_body: $ => /[^<]+/,

    // ── opaque expression / chunk leaves ──────────────────────────
    // Expressions terminate at a newline or `;` — good enough for the
    // single-line declaration forms (`prop`, `state`, `derived`, `data`).
    // Multi-line expressions belong inside `function`, `mount`, `effect`,
    // etc. which use `brace_block`.
    expression: $ => /[^\n;]+/,
    type_annotation: $ => /[^=\n!]+/,

    // Balanced-brace block. Tree-sitter doesn't do balanced delimiters
    // natively; we approximate with a regex that allows nested braces
    // up to one level (covers most real usage). Full nesting handled by
    // an external scanner in v2.
    brace_block: $ => seq(
      '{',
      repeat(choice(
        $._brace_inner,
        $.brace_block,
      )),
      '}',
    ),
    paren_block: $ => seq(
      '(',
      optional(/[^)]*/),
      ')',
    ),
    _brace_inner: $ => /[^{}]+/,
    _brace_body: $ => /[^{}]+/,

    // Lower lexer precedence than the framework keyword tokens so
    // `state count = 0` parses as `state_decl`, not as a single
    // js_chunk regex match. Keep the regex permissive — anything that
    // doesn't begin with a framework keyword falls through to here.
    js_chunk: $ => token(prec(-2, /[^\n<]+\n?/)),

    // ── leaves ────────────────────────────────────────────────────
    text: $ => /[^<{]+/,
    identifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/,
    line_comment: $ => /\/\/[^\n]*/,
    block_comment: $ => /\/\*[\s\S]*?\*\//,
  },
});
