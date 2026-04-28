; Tree-sitter highlight scopes for `.set` (Sail) components.
; Scope names follow the conventional `tree-sitter-highlight` palette
; (see https://tree-sitter.github.io/tree-sitter/syntax-highlighting).
; Editors map them onto their own theme tokens.

; ── framework keywords ────────────────────────────────────────────────
"prop"      @keyword
"state"     @keyword
"derived"   @keyword
"data"      @keyword
"effect"    @keyword
"event"     @keyword
"mount"     @keyword
"unmount"   @keyword
"function"  @keyword
"async"     @keyword
"component" @keyword
"once"      @keyword
"on"        @keyword
"of"        @keyword
"key"       @keyword

"formAssociated"  @keyword
"form_reset"      @keyword
"form_restore"    @keyword
"form_disabled"   @keyword

; Template control-flow openers/closers.
"{#if"     @keyword.control
"{:elif"   @keyword.control
"{:else if" @keyword.control
"{:else}"  @keyword.control
"{/if}"    @keyword.control
"{#for"    @keyword.control
"{:empty}" @keyword.control
"{/for}"   @keyword.control
"{#await"  @keyword.control
"{:then"   @keyword.control
"{:catch"  @keyword.control
"{/await}" @keyword.control
"{@html"   @keyword.control

; ── declaration names ─────────────────────────────────────────────────
(prop_decl     name: (identifier) @variable.member)
(state_decl    name: (identifier) @variable.member)
(derived_decl  name: (identifier) @variable.member)
(data_decl     name: (identifier) @variable.member)
(event_decl    name: (identifier) @variable.member)
(function_decl name: (identifier) @function)

; ── types ─────────────────────────────────────────────────────────────
(type_annotation) @type

; ── template tags / attributes ────────────────────────────────────────
(tag_name)  @tag
(attr_name) @attribute
(on_attr_name) @attribute
"<:host"   @tag
"</:host>" @tag
"bind:"    @attribute
"on:"      @attribute

; Element bracket punctuation.
"<"  @punctuation.bracket
">"  @punctuation.bracket
"</" @punctuation.bracket
"/>" @punctuation.bracket

; Interpolation / brace boundaries.
"{" @punctuation.special
"}" @punctuation.special

; ── literals / leaves ─────────────────────────────────────────────────
(string_literal) @string
(text)           @text

(line_comment)   @comment
(block_comment)  @comment
(html_comment)   @comment

; Block boundaries (opens/closes for the three top-level sections).
(script_open)    @tag.directive
(script_close)   @tag.directive
(template_open)  @tag.directive
(template_close) @tag.directive
(style_open)     @tag.directive
(style_close)    @tag.directive
