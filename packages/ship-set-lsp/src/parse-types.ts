export interface Range { start: number; end: number; }

export type AttrKind =
  | 'bind' | 'on' | 'class' | 'style' | 'attr'
  | 'let' | 'static' | 'dynamic' | 'boolean';

export interface Attribute {
  kind: AttrKind;
  name: string;
  nameRange: Range;
  modifiers: string[];
  modifierRanges: Range[];
  value?: { kind: 'string' | 'expr'; text: string; range: Range };
  outerValueRange?: Range;
  nameExpr?: { text: string; range: Range };
  range: Range;
}

export interface ElementOpen {
  kind: 'element-open';
  tag: string;
  tagRange: Range;
  attrs: Attribute[];
  selfClosing: boolean;
  range: Range;
}
export interface ElementClose { kind: 'element-close'; tag: string; range: Range; }
export interface Comment { kind: 'comment'; range: Range; }
export interface Text { kind: 'text'; range: Range; }
export interface Interpolation {
  kind: 'interpolation';
  text: string;
  range: Range;
  outerRange: Range;
}
export interface BlockOpen {
  kind: 'block-open';
  block: 'if' | 'for' | 'match';
  expr?: { text: string; range: Range };
  pattern?: { idents: string[]; range: Range };
  iter?: { text: string; range: Range };
  key?: { text: string; range: Range };
  outerRange: Range;
}
export interface BlockTag {
  kind: 'block-tag';
  tag: 'else' | 'elif' | 'case' | 'default';
  expr?: { text: string; range: Range };
  outerRange: Range;
}
export interface BlockClose {
  kind: 'block-close';
  block: 'if' | 'for' | 'match';
  outerRange: Range;
}

export type Node =
  | ElementOpen
  | ElementClose
  | Comment
  | Text
  | Interpolation
  | BlockOpen
  | BlockTag
  | BlockClose;

export interface ParseError { range: Range; message: string; }
export interface ParseResult { source: string; nodes: Node[]; errors: ParseError[]; }
