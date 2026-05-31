// node_modules/kysely/dist/esm/util/object-utils.js
function isUndefined(obj) {
  return typeof obj === "undefined" || obj === undefined;
}
function isString(obj) {
  return typeof obj === "string";
}
function isNumber(obj) {
  return typeof obj === "number";
}
function isBoolean(obj) {
  return typeof obj === "boolean";
}
function isNull(obj) {
  return obj === null;
}
function isDate(obj) {
  return obj instanceof Date;
}
function isBigInt(obj) {
  return typeof obj === "bigint";
}
function isFunction(obj) {
  return typeof obj === "function";
}
function isObject(obj) {
  return typeof obj === "object" && obj !== null;
}
function getLast(arr) {
  return arr[arr.length - 1];
}
function freeze(obj) {
  return Object.freeze(obj);
}
function asArray(arg) {
  if (isReadonlyArray(arg)) {
    return arg;
  } else {
    return [arg];
  }
}
function isReadonlyArray(arg) {
  return Array.isArray(arg);
}
function noop(obj) {
  return obj;
}

// node_modules/kysely/dist/esm/operation-node/alter-table-node.js
var AlterTableNode = freeze({
  is(node) {
    return node.kind === "AlterTableNode";
  },
  create(table) {
    return freeze({
      kind: "AlterTableNode",
      table
    });
  },
  cloneWithTableProps(node, props) {
    return freeze({
      ...node,
      ...props
    });
  },
  cloneWithColumnAlteration(node, columnAlteration) {
    return freeze({
      ...node,
      columnAlterations: node.columnAlterations ? [...node.columnAlterations, columnAlteration] : [columnAlteration]
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/identifier-node.js
var IdentifierNode = freeze({
  is(node) {
    return node.kind === "IdentifierNode";
  },
  create(name) {
    return freeze({
      kind: "IdentifierNode",
      name
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/create-index-node.js
var CreateIndexNode = freeze({
  is(node) {
    return node.kind === "CreateIndexNode";
  },
  create(name) {
    return freeze({
      kind: "CreateIndexNode",
      name: IdentifierNode.create(name)
    });
  },
  cloneWith(node, props) {
    return freeze({
      ...node,
      ...props
    });
  },
  cloneWithColumns(node, columns) {
    return freeze({
      ...node,
      columns: [...node.columns || [], ...columns]
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/create-schema-node.js
var CreateSchemaNode = freeze({
  is(node) {
    return node.kind === "CreateSchemaNode";
  },
  create(schema, params) {
    return freeze({
      kind: "CreateSchemaNode",
      schema: IdentifierNode.create(schema),
      ...params
    });
  },
  cloneWith(createSchema, params) {
    return freeze({
      ...createSchema,
      ...params
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/create-table-node.js
var ON_COMMIT_ACTIONS = ["preserve rows", "delete rows", "drop"];
var CreateTableNode = freeze({
  is(node) {
    return node.kind === "CreateTableNode";
  },
  create(table) {
    return freeze({
      kind: "CreateTableNode",
      table,
      columns: freeze([])
    });
  },
  cloneWithColumn(createTable, column) {
    return freeze({
      ...createTable,
      columns: freeze([...createTable.columns, column])
    });
  },
  cloneWithConstraint(createTable, constraint) {
    return freeze({
      ...createTable,
      constraints: createTable.constraints ? freeze([...createTable.constraints, constraint]) : freeze([constraint])
    });
  },
  cloneWithFrontModifier(createTable, modifier) {
    return freeze({
      ...createTable,
      frontModifiers: createTable.frontModifiers ? freeze([...createTable.frontModifiers, modifier]) : freeze([modifier])
    });
  },
  cloneWithEndModifier(createTable, modifier) {
    return freeze({
      ...createTable,
      endModifiers: createTable.endModifiers ? freeze([...createTable.endModifiers, modifier]) : freeze([modifier])
    });
  },
  cloneWith(createTable, params) {
    return freeze({
      ...createTable,
      ...params
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/schemable-identifier-node.js
var SchemableIdentifierNode = freeze({
  is(node) {
    return node.kind === "SchemableIdentifierNode";
  },
  create(identifier) {
    return freeze({
      kind: "SchemableIdentifierNode",
      identifier: IdentifierNode.create(identifier)
    });
  },
  createWithSchema(schema, identifier) {
    return freeze({
      kind: "SchemableIdentifierNode",
      schema: IdentifierNode.create(schema),
      identifier: IdentifierNode.create(identifier)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/drop-index-node.js
var DropIndexNode = freeze({
  is(node) {
    return node.kind === "DropIndexNode";
  },
  create(name, params) {
    return freeze({
      kind: "DropIndexNode",
      name: SchemableIdentifierNode.create(name),
      ...params
    });
  },
  cloneWith(dropIndex, props) {
    return freeze({
      ...dropIndex,
      ...props
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/drop-schema-node.js
var DropSchemaNode = freeze({
  is(node) {
    return node.kind === "DropSchemaNode";
  },
  create(schema, params) {
    return freeze({
      kind: "DropSchemaNode",
      schema: IdentifierNode.create(schema),
      ...params
    });
  },
  cloneWith(dropSchema, params) {
    return freeze({
      ...dropSchema,
      ...params
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/drop-table-node.js
var DropTableNode = freeze({
  is(node) {
    return node.kind === "DropTableNode";
  },
  create(table, params) {
    return freeze({
      kind: "DropTableNode",
      table,
      ...params
    });
  },
  cloneWith(dropIndex, params) {
    return freeze({
      ...dropIndex,
      ...params
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/alias-node.js
var AliasNode = freeze({
  is(node) {
    return node.kind === "AliasNode";
  },
  create(node, alias) {
    return freeze({
      kind: "AliasNode",
      node,
      alias
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/table-node.js
var TableNode = freeze({
  is(node) {
    return node.kind === "TableNode";
  },
  create(table) {
    return freeze({
      kind: "TableNode",
      table: SchemableIdentifierNode.create(table)
    });
  },
  createWithSchema(schema, table) {
    return freeze({
      kind: "TableNode",
      table: SchemableIdentifierNode.createWithSchema(schema, table)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/operation-node-source.js
function isOperationNodeSource(obj) {
  return isObject(obj) && isFunction(obj.toOperationNode);
}

// node_modules/kysely/dist/esm/expression/expression.js
function isExpression(obj) {
  return isObject(obj) && "expressionType" in obj && isOperationNodeSource(obj);
}
function isAliasedExpression(obj) {
  return isObject(obj) && "expression" in obj && isString(obj.alias) && isOperationNodeSource(obj);
}

// node_modules/kysely/dist/esm/operation-node/select-modifier-node.js
var SelectModifierNode = freeze({
  is(node) {
    return node.kind === "SelectModifierNode";
  },
  create(modifier, of) {
    return freeze({
      kind: "SelectModifierNode",
      modifier,
      of
    });
  },
  createWithExpression(modifier) {
    return freeze({
      kind: "SelectModifierNode",
      rawModifier: modifier
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/and-node.js
var AndNode = freeze({
  is(node) {
    return node.kind === "AndNode";
  },
  create(left, right) {
    return freeze({
      kind: "AndNode",
      left,
      right
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/or-node.js
var OrNode = freeze({
  is(node) {
    return node.kind === "OrNode";
  },
  create(left, right) {
    return freeze({
      kind: "OrNode",
      left,
      right
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/on-node.js
var OnNode = freeze({
  is(node) {
    return node.kind === "OnNode";
  },
  create(filter) {
    return freeze({
      kind: "OnNode",
      on: filter
    });
  },
  cloneWithOperation(onNode, operator, operation) {
    return freeze({
      ...onNode,
      on: operator === "And" ? AndNode.create(onNode.on, operation) : OrNode.create(onNode.on, operation)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/join-node.js
var JoinNode = freeze({
  is(node) {
    return node.kind === "JoinNode";
  },
  create(joinType, table) {
    return freeze({
      kind: "JoinNode",
      joinType,
      table,
      on: undefined
    });
  },
  createWithOn(joinType, table, on) {
    return freeze({
      kind: "JoinNode",
      joinType,
      table,
      on: OnNode.create(on)
    });
  },
  cloneWithOn(joinNode, operation) {
    return freeze({
      ...joinNode,
      on: joinNode.on ? OnNode.cloneWithOperation(joinNode.on, "And", operation) : OnNode.create(operation)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/binary-operation-node.js
var BinaryOperationNode = freeze({
  is(node) {
    return node.kind === "BinaryOperationNode";
  },
  create(leftOperand, operator, rightOperand) {
    return freeze({
      kind: "BinaryOperationNode",
      leftOperand,
      operator,
      rightOperand
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/operator-node.js
var COMPARISON_OPERATORS = [
  "=",
  "==",
  "!=",
  "<>",
  ">",
  ">=",
  "<",
  "<=",
  "in",
  "not in",
  "is",
  "is not",
  "like",
  "not like",
  "match",
  "ilike",
  "not ilike",
  "@>",
  "<@",
  "^@",
  "&&",
  "?",
  "?&",
  "?|",
  "!<",
  "!>",
  "<=>",
  "!~",
  "~",
  "~*",
  "!~*",
  "@@",
  "@@@",
  "!!",
  "<->",
  "regexp",
  "is distinct from",
  "is not distinct from"
];
var ARITHMETIC_OPERATORS = [
  "+",
  "-",
  "*",
  "/",
  "%",
  "^",
  "&",
  "|",
  "#",
  "<<",
  ">>"
];
var JSON_OPERATORS = ["->", "->>"];
var BINARY_OPERATORS = [
  ...COMPARISON_OPERATORS,
  ...ARITHMETIC_OPERATORS,
  "&&",
  "||"
];
var UNARY_FILTER_OPERATORS = ["exists", "not exists"];
var UNARY_OPERATORS = ["not", "-", ...UNARY_FILTER_OPERATORS];
var OPERATORS = [
  ...BINARY_OPERATORS,
  ...JSON_OPERATORS,
  ...UNARY_OPERATORS,
  "between",
  "between symmetric"
];
var OperatorNode = freeze({
  is(node) {
    return node.kind === "OperatorNode";
  },
  create(operator) {
    return freeze({
      kind: "OperatorNode",
      operator
    });
  }
});
function isJSONOperator(op) {
  return isString(op) && JSON_OPERATORS.includes(op);
}

// node_modules/kysely/dist/esm/operation-node/column-node.js
var ColumnNode = freeze({
  is(node) {
    return node.kind === "ColumnNode";
  },
  create(column) {
    return freeze({
      kind: "ColumnNode",
      column: IdentifierNode.create(column)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/select-all-node.js
var SelectAllNode = freeze({
  is(node) {
    return node.kind === "SelectAllNode";
  },
  create() {
    return freeze({
      kind: "SelectAllNode"
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/reference-node.js
var ReferenceNode = freeze({
  is(node) {
    return node.kind === "ReferenceNode";
  },
  create(column, table) {
    return freeze({
      kind: "ReferenceNode",
      table,
      column
    });
  },
  createSelectAll(table) {
    return freeze({
      kind: "ReferenceNode",
      table,
      column: SelectAllNode.create()
    });
  }
});

// node_modules/kysely/dist/esm/dynamic/dynamic-reference-builder.js
class DynamicReferenceBuilder {
  #dynamicReference;
  get dynamicReference() {
    return this.#dynamicReference;
  }
  get refType() {
    return;
  }
  constructor(reference) {
    this.#dynamicReference = reference;
  }
  toOperationNode() {
    return parseSimpleReferenceExpression(this.#dynamicReference);
  }
}
function isDynamicReferenceBuilder(obj) {
  return isObject(obj) && isOperationNodeSource(obj) && isString(obj.dynamicReference);
}

// node_modules/kysely/dist/esm/operation-node/order-by-item-node.js
var OrderByItemNode = freeze({
  is(node) {
    return node.kind === "OrderByItemNode";
  },
  create(orderBy, direction) {
    return freeze({
      kind: "OrderByItemNode",
      orderBy,
      direction
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/raw-node.js
var RawNode = freeze({
  is(node) {
    return node.kind === "RawNode";
  },
  create(sqlFragments, parameters) {
    return freeze({
      kind: "RawNode",
      sqlFragments: freeze(sqlFragments),
      parameters: freeze(parameters)
    });
  },
  createWithSql(sql) {
    return RawNode.create([sql], []);
  },
  createWithChild(child) {
    return RawNode.create(["", ""], [child]);
  },
  createWithChildren(children) {
    return RawNode.create(new Array(children.length + 1).fill(""), children);
  }
});

// node_modules/kysely/dist/esm/parser/order-by-parser.js
function isOrderByDirection(thing) {
  return thing === "asc" || thing === "desc";
}
function parseOrderBy(args) {
  if (args.length === 2) {
    return [parseOrderByItem(args[0], args[1])];
  }
  if (args.length === 1) {
    const [orderBy] = args;
    if (Array.isArray(orderBy)) {
      return orderBy.map((item) => parseOrderByItem(item));
    }
    return [parseOrderByItem(orderBy)];
  }
  throw new Error(`Invalid number of arguments at order by! expected 1-2, received ${args.length}`);
}
function parseOrderByItem(ref, direction) {
  const parsedRef = parseOrderByExpression(ref);
  if (OrderByItemNode.is(parsedRef)) {
    if (direction) {
      throw new Error("Cannot specify direction twice!");
    }
    return parsedRef;
  }
  return OrderByItemNode.create(parsedRef, parseOrderByDirectionExpression(direction));
}
function parseOrderByExpression(expr) {
  if (isExpressionOrFactory(expr)) {
    return parseExpression(expr);
  }
  if (isDynamicReferenceBuilder(expr)) {
    return expr.toOperationNode();
  }
  const [ref, direction] = expr.split(" ");
  if (direction) {
    if (!isOrderByDirection(direction)) {
      throw new Error(`Invalid order by direction: ${direction}`);
    }
    return OrderByItemNode.create(parseStringReference(ref), parseOrderByDirectionExpression(direction));
  }
  return parseStringReference(expr);
}
function parseOrderByDirectionExpression(expr) {
  if (!expr) {
    return;
  }
  if (expr === "asc" || expr === "desc") {
    return RawNode.createWithSql(expr);
  }
  return expr.toOperationNode();
}

// node_modules/kysely/dist/esm/operation-node/json-reference-node.js
var JSONReferenceNode = freeze({
  is(node) {
    return node.kind === "JSONReferenceNode";
  },
  create(reference, traversal) {
    return freeze({
      kind: "JSONReferenceNode",
      reference,
      traversal
    });
  },
  cloneWithTraversal(node, traversal) {
    return freeze({
      ...node,
      traversal
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/json-operator-chain-node.js
var JSONOperatorChainNode = freeze({
  is(node) {
    return node.kind === "JSONOperatorChainNode";
  },
  create(operator) {
    return freeze({
      kind: "JSONOperatorChainNode",
      operator,
      values: freeze([])
    });
  },
  cloneWithValue(node, value) {
    return freeze({
      ...node,
      values: freeze([...node.values, value])
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/json-path-node.js
var JSONPathNode = freeze({
  is(node) {
    return node.kind === "JSONPathNode";
  },
  create(inOperator) {
    return freeze({
      kind: "JSONPathNode",
      inOperator,
      pathLegs: freeze([])
    });
  },
  cloneWithLeg(jsonPathNode, pathLeg) {
    return freeze({
      ...jsonPathNode,
      pathLegs: freeze([...jsonPathNode.pathLegs, pathLeg])
    });
  }
});

// node_modules/kysely/dist/esm/parser/reference-parser.js
function parseSimpleReferenceExpression(exp) {
  if (isString(exp)) {
    return parseStringReference(exp);
  }
  return exp.toOperationNode();
}
function parseReferenceExpressionOrList(arg) {
  if (isReadonlyArray(arg)) {
    return arg.map((it) => parseReferenceExpression(it));
  } else {
    return [parseReferenceExpression(arg)];
  }
}
function parseReferenceExpression(exp) {
  if (isExpressionOrFactory(exp)) {
    return parseExpression(exp);
  }
  return parseSimpleReferenceExpression(exp);
}
function parseJSONReference(ref, op) {
  const referenceNode = parseStringReference(ref);
  if (isJSONOperator(op)) {
    return JSONReferenceNode.create(referenceNode, JSONOperatorChainNode.create(OperatorNode.create(op)));
  }
  const opWithoutLastChar = op.slice(0, -1);
  if (isJSONOperator(opWithoutLastChar)) {
    return JSONReferenceNode.create(referenceNode, JSONPathNode.create(OperatorNode.create(opWithoutLastChar)));
  }
  throw new Error(`Invalid JSON operator: ${op}`);
}
function parseStringReference(ref) {
  const COLUMN_SEPARATOR = ".";
  if (!ref.includes(COLUMN_SEPARATOR)) {
    return ReferenceNode.create(ColumnNode.create(ref));
  }
  const parts = ref.split(COLUMN_SEPARATOR).map(trim);
  if (parts.length === 3) {
    return parseStringReferenceWithTableAndSchema(parts);
  }
  if (parts.length === 2) {
    return parseStringReferenceWithTable(parts);
  }
  throw new Error(`invalid column reference ${ref}`);
}
function parseAliasedStringReference(ref) {
  const ALIAS_SEPARATOR = " as ";
  if (ref.includes(ALIAS_SEPARATOR)) {
    const [columnRef, alias] = ref.split(ALIAS_SEPARATOR).map(trim);
    return AliasNode.create(parseStringReference(columnRef), IdentifierNode.create(alias));
  } else {
    return parseStringReference(ref);
  }
}
function parseColumnName(column) {
  return ColumnNode.create(column);
}
function parseOrderedColumnName(column) {
  const ORDER_SEPARATOR = " ";
  if (column.includes(ORDER_SEPARATOR)) {
    const [columnName, order] = column.split(ORDER_SEPARATOR).map(trim);
    if (!isOrderByDirection(order)) {
      throw new Error(`invalid order direction "${order}" next to "${columnName}"`);
    }
    return parseOrderBy([columnName, order])[0];
  } else {
    return parseColumnName(column);
  }
}
function parseStringReferenceWithTableAndSchema(parts) {
  const [schema, table, column] = parts;
  return ReferenceNode.create(ColumnNode.create(column), TableNode.createWithSchema(schema, table));
}
function parseStringReferenceWithTable(parts) {
  const [table, column] = parts;
  return ReferenceNode.create(ColumnNode.create(column), TableNode.create(table));
}
function trim(str) {
  return str.trim();
}

// node_modules/kysely/dist/esm/operation-node/primitive-value-list-node.js
var PrimitiveValueListNode = freeze({
  is(node) {
    return node.kind === "PrimitiveValueListNode";
  },
  create(values) {
    return freeze({
      kind: "PrimitiveValueListNode",
      values: freeze([...values])
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/value-list-node.js
var ValueListNode = freeze({
  is(node) {
    return node.kind === "ValueListNode";
  },
  create(values) {
    return freeze({
      kind: "ValueListNode",
      values: freeze(values)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/value-node.js
var ValueNode = freeze({
  is(node) {
    return node.kind === "ValueNode";
  },
  create(value) {
    return freeze({
      kind: "ValueNode",
      value
    });
  },
  createImmediate(value) {
    return freeze({
      kind: "ValueNode",
      value,
      immediate: true
    });
  }
});

// node_modules/kysely/dist/esm/parser/value-parser.js
function parseValueExpressionOrList(arg) {
  if (isReadonlyArray(arg)) {
    return parseValueExpressionList(arg);
  }
  return parseValueExpression(arg);
}
function parseValueExpression(exp) {
  if (isExpressionOrFactory(exp)) {
    return parseExpression(exp);
  }
  return ValueNode.create(exp);
}
function isSafeImmediateValue(value) {
  return isNumber(value) || isBoolean(value) || isNull(value);
}
function parseSafeImmediateValue(value) {
  if (!isSafeImmediateValue(value)) {
    throw new Error(`unsafe immediate value ${JSON.stringify(value)}`);
  }
  return ValueNode.createImmediate(value);
}
function parseValueExpressionList(arg) {
  if (arg.some(isExpressionOrFactory)) {
    return ValueListNode.create(arg.map((it) => parseValueExpression(it)));
  }
  return PrimitiveValueListNode.create(arg);
}

// node_modules/kysely/dist/esm/operation-node/parens-node.js
var ParensNode = freeze({
  is(node) {
    return node.kind === "ParensNode";
  },
  create(node) {
    return freeze({
      kind: "ParensNode",
      node
    });
  }
});

// node_modules/kysely/dist/esm/parser/binary-operation-parser.js
function parseValueBinaryOperationOrExpression(args) {
  if (args.length === 3) {
    return parseValueBinaryOperation(args[0], args[1], args[2]);
  } else if (args.length === 1) {
    return parseValueExpression(args[0]);
  }
  throw new Error(`invalid arguments: ${JSON.stringify(args)}`);
}
function parseValueBinaryOperation(left, operator, right) {
  if (isIsOperator(operator) && needsIsOperator(right)) {
    return BinaryOperationNode.create(parseReferenceExpression(left), parseOperator(operator), ValueNode.createImmediate(right));
  }
  return BinaryOperationNode.create(parseReferenceExpression(left), parseOperator(operator), parseValueExpressionOrList(right));
}
function parseReferentialBinaryOperation(left, operator, right) {
  return BinaryOperationNode.create(parseReferenceExpression(left), parseOperator(operator), parseReferenceExpression(right));
}
function parseFilterObject(obj, combinator) {
  return parseFilterList(Object.entries(obj).filter(([, v]) => !isUndefined(v)).map(([k, v]) => parseValueBinaryOperation(k, needsIsOperator(v) ? "is" : "=", v)), combinator);
}
function parseFilterList(list, combinator, withParens = true) {
  const combine = combinator === "and" ? AndNode.create : OrNode.create;
  if (list.length === 0) {
    return BinaryOperationNode.create(ValueNode.createImmediate(1), OperatorNode.create("="), ValueNode.createImmediate(combinator === "and" ? 1 : 0));
  }
  let node = toOperationNode(list[0]);
  for (let i = 1;i < list.length; ++i) {
    node = combine(node, toOperationNode(list[i]));
  }
  if (list.length > 1 && withParens) {
    return ParensNode.create(node);
  }
  return node;
}
function isIsOperator(operator) {
  return operator === "is" || operator === "is not";
}
function needsIsOperator(value) {
  return isNull(value) || isBoolean(value);
}
function parseOperator(operator) {
  if (isString(operator) && OPERATORS.includes(operator)) {
    return OperatorNode.create(operator);
  }
  if (isOperationNodeSource(operator)) {
    return operator.toOperationNode();
  }
  throw new Error(`invalid operator ${JSON.stringify(operator)}`);
}
function toOperationNode(nodeOrSource) {
  return isOperationNodeSource(nodeOrSource) ? nodeOrSource.toOperationNode() : nodeOrSource;
}

// node_modules/kysely/dist/esm/operation-node/order-by-node.js
var OrderByNode = freeze({
  is(node) {
    return node.kind === "OrderByNode";
  },
  create(items) {
    return freeze({
      kind: "OrderByNode",
      items: freeze([...items])
    });
  },
  cloneWithItems(orderBy, items) {
    return freeze({
      ...orderBy,
      items: freeze([...orderBy.items, ...items])
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/partition-by-node.js
var PartitionByNode = freeze({
  is(node) {
    return node.kind === "PartitionByNode";
  },
  create(items) {
    return freeze({
      kind: "PartitionByNode",
      items: freeze(items)
    });
  },
  cloneWithItems(partitionBy, items) {
    return freeze({
      ...partitionBy,
      items: freeze([...partitionBy.items, ...items])
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/over-node.js
var OverNode = freeze({
  is(node) {
    return node.kind === "OverNode";
  },
  create() {
    return freeze({
      kind: "OverNode"
    });
  },
  cloneWithOrderByItems(overNode, items) {
    return freeze({
      ...overNode,
      orderBy: overNode.orderBy ? OrderByNode.cloneWithItems(overNode.orderBy, items) : OrderByNode.create(items)
    });
  },
  cloneWithPartitionByItems(overNode, items) {
    return freeze({
      ...overNode,
      partitionBy: overNode.partitionBy ? PartitionByNode.cloneWithItems(overNode.partitionBy, items) : PartitionByNode.create(items)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/from-node.js
var FromNode = freeze({
  is(node) {
    return node.kind === "FromNode";
  },
  create(froms) {
    return freeze({
      kind: "FromNode",
      froms: freeze(froms)
    });
  },
  cloneWithFroms(from, froms) {
    return freeze({
      ...from,
      froms: freeze([...from.froms, ...froms])
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/group-by-node.js
var GroupByNode = freeze({
  is(node) {
    return node.kind === "GroupByNode";
  },
  create(items) {
    return freeze({
      kind: "GroupByNode",
      items: freeze(items)
    });
  },
  cloneWithItems(groupBy, items) {
    return freeze({
      ...groupBy,
      items: freeze([...groupBy.items, ...items])
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/having-node.js
var HavingNode = freeze({
  is(node) {
    return node.kind === "HavingNode";
  },
  create(filter) {
    return freeze({
      kind: "HavingNode",
      having: filter
    });
  },
  cloneWithOperation(havingNode, operator, operation) {
    return freeze({
      ...havingNode,
      having: operator === "And" ? AndNode.create(havingNode.having, operation) : OrNode.create(havingNode.having, operation)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/select-query-node.js
var SelectQueryNode = freeze({
  is(node) {
    return node.kind === "SelectQueryNode";
  },
  create(withNode) {
    return freeze({
      kind: "SelectQueryNode",
      ...withNode && { with: withNode }
    });
  },
  createFrom(fromItems, withNode) {
    return freeze({
      kind: "SelectQueryNode",
      from: FromNode.create(fromItems),
      ...withNode && { with: withNode }
    });
  },
  cloneWithSelections(select, selections) {
    return freeze({
      ...select,
      selections: select.selections ? freeze([...select.selections, ...selections]) : freeze(selections)
    });
  },
  cloneWithDistinctOn(select, expressions) {
    return freeze({
      ...select,
      distinctOn: select.distinctOn ? freeze([...select.distinctOn, ...expressions]) : freeze(expressions)
    });
  },
  cloneWithFrontModifier(select, modifier) {
    return freeze({
      ...select,
      frontModifiers: select.frontModifiers ? freeze([...select.frontModifiers, modifier]) : freeze([modifier])
    });
  },
  cloneWithEndModifier(select, modifier) {
    return freeze({
      ...select,
      endModifiers: select.endModifiers ? freeze([...select.endModifiers, modifier]) : freeze([modifier])
    });
  },
  cloneWithOrderByItems(selectNode, items) {
    return freeze({
      ...selectNode,
      orderBy: selectNode.orderBy ? OrderByNode.cloneWithItems(selectNode.orderBy, items) : OrderByNode.create(items)
    });
  },
  cloneWithGroupByItems(selectNode, items) {
    return freeze({
      ...selectNode,
      groupBy: selectNode.groupBy ? GroupByNode.cloneWithItems(selectNode.groupBy, items) : GroupByNode.create(items)
    });
  },
  cloneWithLimit(selectNode, limit) {
    return freeze({
      ...selectNode,
      limit
    });
  },
  cloneWithOffset(selectNode, offset) {
    return freeze({
      ...selectNode,
      offset
    });
  },
  cloneWithFetch(selectNode, fetch) {
    return freeze({
      ...selectNode,
      fetch
    });
  },
  cloneWithHaving(selectNode, operation) {
    return freeze({
      ...selectNode,
      having: selectNode.having ? HavingNode.cloneWithOperation(selectNode.having, "And", operation) : HavingNode.create(operation)
    });
  },
  cloneWithSetOperations(selectNode, setOperations) {
    return freeze({
      ...selectNode,
      setOperations: selectNode.setOperations ? freeze([...selectNode.setOperations, ...setOperations]) : freeze([...setOperations])
    });
  },
  cloneWithoutSelections(select) {
    return freeze({
      ...select,
      selections: []
    });
  },
  cloneWithoutLimit(select) {
    return freeze({
      ...select,
      limit: undefined
    });
  },
  cloneWithoutOffset(select) {
    return freeze({
      ...select,
      offset: undefined
    });
  },
  cloneWithoutOrderBy(select) {
    return freeze({
      ...select,
      orderBy: undefined
    });
  },
  cloneWithoutGroupBy(select) {
    return freeze({
      ...select,
      groupBy: undefined
    });
  }
});

// node_modules/kysely/dist/esm/util/prevent-await.js
function preventAwait(clazz, message) {
  Object.defineProperties(clazz.prototype, {
    then: {
      enumerable: false,
      value: () => {
        throw new Error(message);
      }
    }
  });
}

// node_modules/kysely/dist/esm/query-builder/join-builder.js
class JoinBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  on(...args) {
    return new JoinBuilder({
      ...this.#props,
      joinNode: JoinNode.cloneWithOn(this.#props.joinNode, parseValueBinaryOperationOrExpression(args))
    });
  }
  onRef(lhs, op, rhs) {
    return new JoinBuilder({
      ...this.#props,
      joinNode: JoinNode.cloneWithOn(this.#props.joinNode, parseReferentialBinaryOperation(lhs, op, rhs))
    });
  }
  onTrue() {
    return new JoinBuilder({
      ...this.#props,
      joinNode: JoinNode.cloneWithOn(this.#props.joinNode, RawNode.createWithSql("true"))
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.joinNode;
  }
}
preventAwait(JoinBuilder, "don't await JoinBuilder instances. They are never executed directly and are always just a part of a query.");

// node_modules/kysely/dist/esm/operation-node/partition-by-item-node.js
var PartitionByItemNode = freeze({
  is(node) {
    return node.kind === "PartitionByItemNode";
  },
  create(partitionBy) {
    return freeze({
      kind: "PartitionByItemNode",
      partitionBy
    });
  }
});

// node_modules/kysely/dist/esm/parser/partition-by-parser.js
function parsePartitionBy(partitionBy) {
  return parseReferenceExpressionOrList(partitionBy).map(PartitionByItemNode.create);
}

// node_modules/kysely/dist/esm/query-builder/over-builder.js
class OverBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  orderBy(orderBy, direction) {
    return new OverBuilder({
      overNode: OverNode.cloneWithOrderByItems(this.#props.overNode, parseOrderBy([orderBy, direction]))
    });
  }
  partitionBy(partitionBy) {
    return new OverBuilder({
      overNode: OverNode.cloneWithPartitionByItems(this.#props.overNode, parsePartitionBy(partitionBy))
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.overNode;
  }
}
preventAwait(OverBuilder, "don't await OverBuilder instances. They are never executed directly and are always just a part of a query.");

// node_modules/kysely/dist/esm/operation-node/selection-node.js
var SelectionNode = freeze({
  is(node) {
    return node.kind === "SelectionNode";
  },
  create(selection) {
    return freeze({
      kind: "SelectionNode",
      selection
    });
  },
  createSelectAll() {
    return freeze({
      kind: "SelectionNode",
      selection: SelectAllNode.create()
    });
  },
  createSelectAllFromTable(table) {
    return freeze({
      kind: "SelectionNode",
      selection: ReferenceNode.createSelectAll(table)
    });
  }
});

// node_modules/kysely/dist/esm/parser/select-parser.js
function parseSelectArg(selection) {
  if (isFunction(selection)) {
    return parseSelectArg(selection(expressionBuilder()));
  } else if (isReadonlyArray(selection)) {
    return selection.map((it) => parseSelectExpression(it));
  } else {
    return [parseSelectExpression(selection)];
  }
}
function parseSelectExpression(selection) {
  if (isString(selection)) {
    return SelectionNode.create(parseAliasedStringReference(selection));
  } else if (isDynamicReferenceBuilder(selection)) {
    return SelectionNode.create(selection.toOperationNode());
  } else {
    return SelectionNode.create(parseAliasedExpression(selection));
  }
}
function parseSelectAll(table) {
  if (!table) {
    return [SelectionNode.createSelectAll()];
  } else if (Array.isArray(table)) {
    return table.map(parseSelectAllArg);
  } else {
    return [parseSelectAllArg(table)];
  }
}
function parseSelectAllArg(table) {
  if (isString(table)) {
    return SelectionNode.createSelectAllFromTable(parseTable(table));
  }
  throw new Error(`invalid value selectAll expression: ${JSON.stringify(table)}`);
}

// node_modules/kysely/dist/esm/operation-node/values-node.js
var ValuesNode = freeze({
  is(node) {
    return node.kind === "ValuesNode";
  },
  create(values) {
    return freeze({
      kind: "ValuesNode",
      values: freeze(values)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/default-insert-value-node.js
var DefaultInsertValueNode = freeze({
  is(node) {
    return node.kind === "DefaultInsertValueNode";
  },
  create() {
    return freeze({
      kind: "DefaultInsertValueNode"
    });
  }
});

// node_modules/kysely/dist/esm/parser/insert-values-parser.js
function parseInsertExpression(arg) {
  const objectOrList = isFunction(arg) ? arg(expressionBuilder()) : arg;
  const list = isReadonlyArray(objectOrList) ? objectOrList : freeze([objectOrList]);
  return parseInsertColumnsAndValues(list);
}
function parseInsertColumnsAndValues(rows) {
  const columns = parseColumnNamesAndIndexes(rows);
  return [
    freeze([...columns.keys()].map(ColumnNode.create)),
    ValuesNode.create(rows.map((row) => parseRowValues(row, columns)))
  ];
}
function parseColumnNamesAndIndexes(rows) {
  const columns = new Map;
  for (const row of rows) {
    const cols = Object.keys(row);
    for (const col of cols) {
      if (!columns.has(col) && row[col] !== undefined) {
        columns.set(col, columns.size);
      }
    }
  }
  return columns;
}
function parseRowValues(row, columns) {
  const rowColumns = Object.keys(row);
  const rowValues = Array.from({
    length: columns.size
  });
  let hasUndefinedOrComplexColumns = false;
  for (const col of rowColumns) {
    const columnIdx = columns.get(col);
    if (isUndefined(columnIdx)) {
      continue;
    }
    const value = row[col];
    if (isUndefined(value) || isExpressionOrFactory(value)) {
      hasUndefinedOrComplexColumns = true;
    }
    rowValues[columnIdx] = value;
  }
  const hasMissingColumns = rowColumns.length < columns.size;
  if (hasMissingColumns || hasUndefinedOrComplexColumns) {
    const defaultValue = DefaultInsertValueNode.create();
    return ValueListNode.create(rowValues.map((it) => isUndefined(it) ? defaultValue : parseValueExpression(it)));
  }
  return PrimitiveValueListNode.create(rowValues);
}

// node_modules/kysely/dist/esm/operation-node/insert-query-node.js
var InsertQueryNode = freeze({
  is(node) {
    return node.kind === "InsertQueryNode";
  },
  create(into, withNode, replace) {
    return freeze({
      kind: "InsertQueryNode",
      into,
      ...withNode && { with: withNode },
      replace
    });
  },
  createWithoutInto() {
    return freeze({
      kind: "InsertQueryNode"
    });
  },
  cloneWith(insertQuery, props) {
    return freeze({
      ...insertQuery,
      ...props
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/update-query-node.js
var UpdateQueryNode = freeze({
  is(node) {
    return node.kind === "UpdateQueryNode";
  },
  create(table, withNode) {
    return freeze({
      kind: "UpdateQueryNode",
      table,
      ...withNode && { with: withNode }
    });
  },
  createWithoutTable() {
    return freeze({
      kind: "UpdateQueryNode"
    });
  },
  cloneWithFromItems(updateQuery, fromItems) {
    return freeze({
      ...updateQuery,
      from: updateQuery.from ? FromNode.cloneWithFroms(updateQuery.from, fromItems) : FromNode.create(fromItems)
    });
  },
  cloneWithUpdates(updateQuery, updates) {
    return freeze({
      ...updateQuery,
      updates: updateQuery.updates ? freeze([...updateQuery.updates, ...updates]) : updates
    });
  },
  cloneWithLimit(updateQuery, limit) {
    return freeze({
      ...updateQuery,
      limit
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/using-node.js
var UsingNode = freeze({
  is(node) {
    return node.kind === "UsingNode";
  },
  create(tables) {
    return freeze({
      kind: "UsingNode",
      tables: freeze(tables)
    });
  },
  cloneWithTables(using, tables) {
    return freeze({
      ...using,
      tables: freeze([...using.tables, ...tables])
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/delete-query-node.js
var DeleteQueryNode = freeze({
  is(node) {
    return node.kind === "DeleteQueryNode";
  },
  create(fromItems, withNode) {
    return freeze({
      kind: "DeleteQueryNode",
      from: FromNode.create(fromItems),
      ...withNode && { with: withNode }
    });
  },
  cloneWithOrderByItems(deleteNode, items) {
    return freeze({
      ...deleteNode,
      orderBy: deleteNode.orderBy ? OrderByNode.cloneWithItems(deleteNode.orderBy, items) : OrderByNode.create(items)
    });
  },
  cloneWithoutOrderBy(deleteNode) {
    return freeze({
      ...deleteNode,
      orderBy: undefined
    });
  },
  cloneWithLimit(deleteNode, limit) {
    return freeze({
      ...deleteNode,
      limit
    });
  },
  cloneWithoutLimit(deleteNode) {
    return freeze({
      ...deleteNode,
      limit: undefined
    });
  },
  cloneWithUsing(deleteNode, tables) {
    return freeze({
      ...deleteNode,
      using: deleteNode.using !== undefined ? UsingNode.cloneWithTables(deleteNode.using, tables) : UsingNode.create(tables)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/where-node.js
var WhereNode = freeze({
  is(node) {
    return node.kind === "WhereNode";
  },
  create(filter) {
    return freeze({
      kind: "WhereNode",
      where: filter
    });
  },
  cloneWithOperation(whereNode, operator, operation) {
    return freeze({
      ...whereNode,
      where: operator === "And" ? AndNode.create(whereNode.where, operation) : OrNode.create(whereNode.where, operation)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/returning-node.js
var ReturningNode = freeze({
  is(node) {
    return node.kind === "ReturningNode";
  },
  create(selections) {
    return freeze({
      kind: "ReturningNode",
      selections: freeze(selections)
    });
  },
  cloneWithSelections(returning, selections) {
    return freeze({
      ...returning,
      selections: returning.selections ? freeze([...returning.selections, ...selections]) : freeze(selections)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/explain-node.js
var ExplainNode = freeze({
  is(node) {
    return node.kind === "ExplainNode";
  },
  create(format, options) {
    return freeze({
      kind: "ExplainNode",
      format,
      options
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/when-node.js
var WhenNode = freeze({
  is(node) {
    return node.kind === "WhenNode";
  },
  create(condition) {
    return freeze({
      kind: "WhenNode",
      condition
    });
  },
  cloneWithResult(whenNode, result) {
    return freeze({
      ...whenNode,
      result
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/merge-query-node.js
var MergeQueryNode = freeze({
  is(node) {
    return node.kind === "MergeQueryNode";
  },
  create(into, withNode) {
    return freeze({
      kind: "MergeQueryNode",
      into,
      ...withNode && { with: withNode }
    });
  },
  cloneWithUsing(mergeNode, using) {
    return freeze({
      ...mergeNode,
      using
    });
  },
  cloneWithWhen(mergeNode, when) {
    return freeze({
      ...mergeNode,
      whens: mergeNode.whens ? freeze([...mergeNode.whens, when]) : freeze([when])
    });
  },
  cloneWithThen(mergeNode, then) {
    return freeze({
      ...mergeNode,
      whens: mergeNode.whens ? freeze([
        ...mergeNode.whens.slice(0, -1),
        WhenNode.cloneWithResult(mergeNode.whens[mergeNode.whens.length - 1], then)
      ]) : undefined
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/output-node.js
var OutputNode = freeze({
  is(node) {
    return node.kind === "OutputNode";
  },
  create(selections) {
    return freeze({
      kind: "OutputNode",
      selections: freeze(selections)
    });
  },
  cloneWithSelections(output, selections) {
    return freeze({
      ...output,
      selections: output.selections ? freeze([...output.selections, ...selections]) : freeze(selections)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/query-node.js
var QueryNode = freeze({
  is(node) {
    return SelectQueryNode.is(node) || InsertQueryNode.is(node) || UpdateQueryNode.is(node) || DeleteQueryNode.is(node) || MergeQueryNode.is(node);
  },
  cloneWithWhere(node, operation) {
    return freeze({
      ...node,
      where: node.where ? WhereNode.cloneWithOperation(node.where, "And", operation) : WhereNode.create(operation)
    });
  },
  cloneWithJoin(node, join) {
    return freeze({
      ...node,
      joins: node.joins ? freeze([...node.joins, join]) : freeze([join])
    });
  },
  cloneWithReturning(node, selections) {
    return freeze({
      ...node,
      returning: node.returning ? ReturningNode.cloneWithSelections(node.returning, selections) : ReturningNode.create(selections)
    });
  },
  cloneWithoutReturning(node) {
    return freeze({
      ...node,
      returning: undefined
    });
  },
  cloneWithoutWhere(node) {
    return freeze({
      ...node,
      where: undefined
    });
  },
  cloneWithExplain(node, format, options) {
    return freeze({
      ...node,
      explain: ExplainNode.create(format, options?.toOperationNode())
    });
  },
  cloneWithTop(node, top) {
    return freeze({
      ...node,
      top
    });
  },
  cloneWithOutput(node, selections) {
    return freeze({
      ...node,
      output: node.output ? OutputNode.cloneWithSelections(node.output, selections) : OutputNode.create(selections)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/column-update-node.js
var ColumnUpdateNode = freeze({
  is(node) {
    return node.kind === "ColumnUpdateNode";
  },
  create(column, value) {
    return freeze({
      kind: "ColumnUpdateNode",
      column,
      value
    });
  }
});

// node_modules/kysely/dist/esm/parser/update-set-parser.js
function parseUpdate(...args) {
  if (args.length === 2) {
    return [
      ColumnUpdateNode.create(parseReferenceExpression(args[0]), parseValueExpression(args[1]))
    ];
  }
  return parseUpdateObjectExpression(args[0]);
}
function parseUpdateObjectExpression(update) {
  const updateObj = isFunction(update) ? update(expressionBuilder()) : update;
  return Object.entries(updateObj).filter(([_, value]) => value !== undefined).map(([key, value]) => {
    return ColumnUpdateNode.create(ColumnNode.create(key), parseValueExpression(value));
  });
}

// node_modules/kysely/dist/esm/operation-node/on-duplicate-key-node.js
var OnDuplicateKeyNode = freeze({
  is(node) {
    return node.kind === "OnDuplicateKeyNode";
  },
  create(updates) {
    return freeze({
      kind: "OnDuplicateKeyNode",
      updates
    });
  }
});

// node_modules/kysely/dist/esm/query-builder/insert-result.js
class InsertResult {
  insertId;
  numInsertedOrUpdatedRows;
  constructor(insertId, numInsertedOrUpdatedRows) {
    this.insertId = insertId;
    this.numInsertedOrUpdatedRows = numInsertedOrUpdatedRows;
  }
}

// node_modules/kysely/dist/esm/query-builder/no-result-error.js
class NoResultError extends Error {
  node;
  constructor(node) {
    super("no result");
    this.node = node;
  }
}
function isNoResultErrorConstructor(fn) {
  return Object.prototype.hasOwnProperty.call(fn, "prototype");
}

// node_modules/kysely/dist/esm/operation-node/on-conflict-node.js
var OnConflictNode = freeze({
  is(node) {
    return node.kind === "OnConflictNode";
  },
  create() {
    return freeze({
      kind: "OnConflictNode"
    });
  },
  cloneWith(node, props) {
    return freeze({
      ...node,
      ...props
    });
  },
  cloneWithIndexWhere(node, operation) {
    return freeze({
      ...node,
      indexWhere: node.indexWhere ? WhereNode.cloneWithOperation(node.indexWhere, "And", operation) : WhereNode.create(operation)
    });
  },
  cloneWithIndexOrWhere(node, operation) {
    return freeze({
      ...node,
      indexWhere: node.indexWhere ? WhereNode.cloneWithOperation(node.indexWhere, "Or", operation) : WhereNode.create(operation)
    });
  },
  cloneWithUpdateWhere(node, operation) {
    return freeze({
      ...node,
      updateWhere: node.updateWhere ? WhereNode.cloneWithOperation(node.updateWhere, "And", operation) : WhereNode.create(operation)
    });
  },
  cloneWithUpdateOrWhere(node, operation) {
    return freeze({
      ...node,
      updateWhere: node.updateWhere ? WhereNode.cloneWithOperation(node.updateWhere, "Or", operation) : WhereNode.create(operation)
    });
  },
  cloneWithoutIndexWhere(node) {
    return freeze({
      ...node,
      indexWhere: undefined
    });
  },
  cloneWithoutUpdateWhere(node) {
    return freeze({
      ...node,
      updateWhere: undefined
    });
  }
});

// node_modules/kysely/dist/esm/query-builder/on-conflict-builder.js
class OnConflictBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  column(column) {
    const columnNode = ColumnNode.create(column);
    return new OnConflictBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWith(this.#props.onConflictNode, {
        columns: this.#props.onConflictNode.columns ? freeze([...this.#props.onConflictNode.columns, columnNode]) : freeze([columnNode])
      })
    });
  }
  columns(columns) {
    const columnNodes = columns.map(ColumnNode.create);
    return new OnConflictBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWith(this.#props.onConflictNode, {
        columns: this.#props.onConflictNode.columns ? freeze([...this.#props.onConflictNode.columns, ...columnNodes]) : freeze(columnNodes)
      })
    });
  }
  constraint(constraintName) {
    return new OnConflictBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWith(this.#props.onConflictNode, {
        constraint: IdentifierNode.create(constraintName)
      })
    });
  }
  expression(expression) {
    return new OnConflictBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWith(this.#props.onConflictNode, {
        indexExpression: expression.toOperationNode()
      })
    });
  }
  where(...args) {
    return new OnConflictBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWithIndexWhere(this.#props.onConflictNode, parseValueBinaryOperationOrExpression(args))
    });
  }
  whereRef(lhs, op, rhs) {
    return new OnConflictBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWithIndexWhere(this.#props.onConflictNode, parseReferentialBinaryOperation(lhs, op, rhs))
    });
  }
  clearWhere() {
    return new OnConflictBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWithoutIndexWhere(this.#props.onConflictNode)
    });
  }
  doNothing() {
    return new OnConflictDoNothingBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWith(this.#props.onConflictNode, {
        doNothing: true
      })
    });
  }
  doUpdateSet(update) {
    return new OnConflictUpdateBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWith(this.#props.onConflictNode, {
        updates: parseUpdateObjectExpression(update)
      })
    });
  }
  $call(func) {
    return func(this);
  }
}
preventAwait(OnConflictBuilder, "don't await OnConflictBuilder instances.");

class OnConflictDoNothingBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  toOperationNode() {
    return this.#props.onConflictNode;
  }
}
preventAwait(OnConflictDoNothingBuilder, "don't await OnConflictDoNothingBuilder instances.");

class OnConflictUpdateBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  where(...args) {
    return new OnConflictUpdateBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWithUpdateWhere(this.#props.onConflictNode, parseValueBinaryOperationOrExpression(args))
    });
  }
  whereRef(lhs, op, rhs) {
    return new OnConflictUpdateBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWithUpdateWhere(this.#props.onConflictNode, parseReferentialBinaryOperation(lhs, op, rhs))
    });
  }
  clearWhere() {
    return new OnConflictUpdateBuilder({
      ...this.#props,
      onConflictNode: OnConflictNode.cloneWithoutUpdateWhere(this.#props.onConflictNode)
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.onConflictNode;
  }
}
preventAwait(OnConflictUpdateBuilder, "don't await OnConflictUpdateBuilder instances.");

// node_modules/kysely/dist/esm/operation-node/top-node.js
var TopNode = freeze({
  is(node) {
    return node.kind === "TopNode";
  },
  create(expression, modifiers) {
    return freeze({
      kind: "TopNode",
      expression,
      modifiers
    });
  }
});

// node_modules/kysely/dist/esm/parser/top-parser.js
function parseTop(expression, modifiers) {
  if (!isNumber(expression) && !isBigInt(expression)) {
    throw new Error(`Invalid top expression: ${expression}`);
  }
  if (!isUndefined(modifiers) && !isTopModifiers(modifiers)) {
    throw new Error(`Invalid top modifiers: ${modifiers}`);
  }
  return TopNode.create(expression, modifiers);
}
function isTopModifiers(modifiers) {
  return modifiers === "percent" || modifiers === "with ties" || modifiers === "percent with ties";
}

// node_modules/kysely/dist/esm/query-builder/insert-query-builder.js
class InsertQueryBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  values(insert) {
    const [columns, values] = parseInsertExpression(insert);
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: InsertQueryNode.cloneWith(this.#props.queryNode, {
        columns,
        values
      })
    });
  }
  columns(columns) {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: InsertQueryNode.cloneWith(this.#props.queryNode, {
        columns: freeze(columns.map(ColumnNode.create))
      })
    });
  }
  expression(expression) {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: InsertQueryNode.cloneWith(this.#props.queryNode, {
        values: parseExpression(expression)
      })
    });
  }
  defaultValues() {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: InsertQueryNode.cloneWith(this.#props.queryNode, {
        defaultValues: true
      })
    });
  }
  ignore() {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: InsertQueryNode.cloneWith(this.#props.queryNode, {
        ignore: true
      })
    });
  }
  top(expression, modifiers) {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithTop(this.#props.queryNode, parseTop(expression, modifiers))
    });
  }
  onConflict(callback) {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: InsertQueryNode.cloneWith(this.#props.queryNode, {
        onConflict: callback(new OnConflictBuilder({
          onConflictNode: OnConflictNode.create()
        })).toOperationNode()
      })
    });
  }
  onDuplicateKeyUpdate(update) {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: InsertQueryNode.cloneWith(this.#props.queryNode, {
        onDuplicateKey: OnDuplicateKeyNode.create(parseUpdateObjectExpression(update))
      })
    });
  }
  returning(selection) {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithReturning(this.#props.queryNode, parseSelectArg(selection))
    });
  }
  returningAll() {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithReturning(this.#props.queryNode, parseSelectAll())
    });
  }
  output(args) {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithOutput(this.#props.queryNode, parseSelectArg(args))
    });
  }
  outputAll(table) {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithOutput(this.#props.queryNode, parseSelectAll(table))
    });
  }
  clearReturning() {
    return new InsertQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithoutReturning(this.#props.queryNode)
    });
  }
  $call(func) {
    return func(this);
  }
  $if(condition, func) {
    if (condition) {
      return func(this);
    }
    return new InsertQueryBuilder({
      ...this.#props
    });
  }
  $castTo() {
    return new InsertQueryBuilder(this.#props);
  }
  $narrowType() {
    return new InsertQueryBuilder(this.#props);
  }
  $assertType() {
    return new InsertQueryBuilder(this.#props);
  }
  withPlugin(plugin) {
    return new InsertQueryBuilder({
      ...this.#props,
      executor: this.#props.executor.withPlugin(plugin)
    });
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.queryNode, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    const compiledQuery = this.compile();
    const result = await this.#props.executor.executeQuery(compiledQuery, this.#props.queryId);
    const { adapter } = this.#props.executor;
    const query = compiledQuery.query;
    if (query.returning && adapter.supportsReturning || query.output && adapter.supportsOutput) {
      return result.rows;
    }
    return [
      new InsertResult(result.insertId, result.numAffectedRows ?? result.numUpdatedOrDeletedRows)
    ];
  }
  async executeTakeFirst() {
    const [result] = await this.execute();
    return result;
  }
  async executeTakeFirstOrThrow(errorConstructor = NoResultError) {
    const result = await this.executeTakeFirst();
    if (result === undefined) {
      const error = isNoResultErrorConstructor(errorConstructor) ? new errorConstructor(this.toOperationNode()) : errorConstructor(this.toOperationNode());
      throw error;
    }
    return result;
  }
  async* stream(chunkSize = 100) {
    const compiledQuery = this.compile();
    const stream = this.#props.executor.stream(compiledQuery, chunkSize, this.#props.queryId);
    for await (const item of stream) {
      yield* item.rows;
    }
  }
  async explain(format, options) {
    const builder = new InsertQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithExplain(this.#props.queryNode, format, options)
    });
    return await builder.execute();
  }
}
preventAwait(InsertQueryBuilder, "don't await InsertQueryBuilder instances directly. To execute the query you need to call `execute` or `executeTakeFirst`.");

// node_modules/kysely/dist/esm/query-builder/delete-result.js
class DeleteResult {
  numDeletedRows;
  constructor(numDeletedRows) {
    this.numDeletedRows = numDeletedRows;
  }
}

// node_modules/kysely/dist/esm/operation-node/limit-node.js
var LimitNode = freeze({
  is(node) {
    return node.kind === "LimitNode";
  },
  create(limit) {
    return freeze({
      kind: "LimitNode",
      limit
    });
  }
});

// node_modules/kysely/dist/esm/query-builder/delete-query-builder.js
class DeleteQueryBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  where(...args) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithWhere(this.#props.queryNode, parseValueBinaryOperationOrExpression(args))
    });
  }
  whereRef(lhs, op, rhs) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithWhere(this.#props.queryNode, parseReferentialBinaryOperation(lhs, op, rhs))
    });
  }
  clearWhere() {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithoutWhere(this.#props.queryNode)
    });
  }
  top(expression, modifiers) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithTop(this.#props.queryNode, parseTop(expression, modifiers))
    });
  }
  using(tables) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: DeleteQueryNode.cloneWithUsing(this.#props.queryNode, parseTableExpressionOrList(tables))
    });
  }
  innerJoin(...args) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("InnerJoin", args))
    });
  }
  leftJoin(...args) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("LeftJoin", args))
    });
  }
  rightJoin(...args) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("RightJoin", args))
    });
  }
  fullJoin(...args) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("FullJoin", args))
    });
  }
  returning(selection) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithReturning(this.#props.queryNode, parseSelectArg(selection))
    });
  }
  returningAll(table) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithReturning(this.#props.queryNode, parseSelectAll(table))
    });
  }
  output(args) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithOutput(this.#props.queryNode, parseSelectArg(args))
    });
  }
  outputAll(table) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithOutput(this.#props.queryNode, parseSelectAll(table))
    });
  }
  clearReturning() {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithoutReturning(this.#props.queryNode)
    });
  }
  clearLimit() {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: DeleteQueryNode.cloneWithoutLimit(this.#props.queryNode)
    });
  }
  clearOrderBy() {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: DeleteQueryNode.cloneWithoutOrderBy(this.#props.queryNode)
    });
  }
  orderBy(orderBy, direction) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: DeleteQueryNode.cloneWithOrderByItems(this.#props.queryNode, parseOrderBy([orderBy, direction]))
    });
  }
  limit(limit) {
    return new DeleteQueryBuilder({
      ...this.#props,
      queryNode: DeleteQueryNode.cloneWithLimit(this.#props.queryNode, LimitNode.create(parseValueExpression(limit)))
    });
  }
  $call(func) {
    return func(this);
  }
  $if(condition, func) {
    if (condition) {
      return func(this);
    }
    return new DeleteQueryBuilder({
      ...this.#props
    });
  }
  $castTo() {
    return new DeleteQueryBuilder(this.#props);
  }
  $narrowType() {
    return new DeleteQueryBuilder(this.#props);
  }
  $assertType() {
    return new DeleteQueryBuilder(this.#props);
  }
  withPlugin(plugin) {
    return new DeleteQueryBuilder({
      ...this.#props,
      executor: this.#props.executor.withPlugin(plugin)
    });
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.queryNode, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    const compiledQuery = this.compile();
    const result = await this.#props.executor.executeQuery(compiledQuery, this.#props.queryId);
    const { adapter } = this.#props.executor;
    const query = compiledQuery.query;
    if (query.returning && adapter.supportsReturning || query.output && adapter.supportsOutput) {
      return result.rows;
    }
    return [
      new DeleteResult(result.numAffectedRows ?? result.numUpdatedOrDeletedRows ?? BigInt(0))
    ];
  }
  async executeTakeFirst() {
    const [result] = await this.execute();
    return result;
  }
  async executeTakeFirstOrThrow(errorConstructor = NoResultError) {
    const result = await this.executeTakeFirst();
    if (result === undefined) {
      const error = isNoResultErrorConstructor(errorConstructor) ? new errorConstructor(this.toOperationNode()) : errorConstructor(this.toOperationNode());
      throw error;
    }
    return result;
  }
  async* stream(chunkSize = 100) {
    const compiledQuery = this.compile();
    const stream = this.#props.executor.stream(compiledQuery, chunkSize, this.#props.queryId);
    for await (const item of stream) {
      yield* item.rows;
    }
  }
  async explain(format, options) {
    const builder = new DeleteQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithExplain(this.#props.queryNode, format, options)
    });
    return await builder.execute();
  }
}
preventAwait(DeleteQueryBuilder, "don't await DeleteQueryBuilder instances directly. To execute the query you need to call `execute` or `executeTakeFirst`.");

// node_modules/kysely/dist/esm/query-builder/update-result.js
class UpdateResult {
  numUpdatedRows;
  numChangedRows;
  constructor(numUpdatedRows, numChangedRows) {
    this.numUpdatedRows = numUpdatedRows;
    this.numChangedRows = numChangedRows;
  }
}

// node_modules/kysely/dist/esm/query-builder/update-query-builder.js
class UpdateQueryBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  where(...args) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithWhere(this.#props.queryNode, parseValueBinaryOperationOrExpression(args))
    });
  }
  whereRef(lhs, op, rhs) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithWhere(this.#props.queryNode, parseReferentialBinaryOperation(lhs, op, rhs))
    });
  }
  clearWhere() {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithoutWhere(this.#props.queryNode)
    });
  }
  top(expression, modifiers) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithTop(this.#props.queryNode, parseTop(expression, modifiers))
    });
  }
  from(from) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: UpdateQueryNode.cloneWithFromItems(this.#props.queryNode, parseTableExpressionOrList(from))
    });
  }
  innerJoin(...args) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("InnerJoin", args))
    });
  }
  leftJoin(...args) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("LeftJoin", args))
    });
  }
  rightJoin(...args) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("RightJoin", args))
    });
  }
  fullJoin(...args) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("FullJoin", args))
    });
  }
  limit(limit) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: UpdateQueryNode.cloneWithLimit(this.#props.queryNode, LimitNode.create(parseValueExpression(limit)))
    });
  }
  set(...args) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: UpdateQueryNode.cloneWithUpdates(this.#props.queryNode, parseUpdate(...args))
    });
  }
  returning(selection) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithReturning(this.#props.queryNode, parseSelectArg(selection))
    });
  }
  returningAll(table) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithReturning(this.#props.queryNode, parseSelectAll(table))
    });
  }
  output(args) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithOutput(this.#props.queryNode, parseSelectArg(args))
    });
  }
  outputAll(table) {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithOutput(this.#props.queryNode, parseSelectAll(table))
    });
  }
  clearReturning() {
    return new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithoutReturning(this.#props.queryNode)
    });
  }
  $call(func) {
    return func(this);
  }
  $if(condition, func) {
    if (condition) {
      return func(this);
    }
    return new UpdateQueryBuilder({
      ...this.#props
    });
  }
  $castTo() {
    return new UpdateQueryBuilder(this.#props);
  }
  $narrowType() {
    return new UpdateQueryBuilder(this.#props);
  }
  $assertType() {
    return new UpdateQueryBuilder(this.#props);
  }
  withPlugin(plugin) {
    return new UpdateQueryBuilder({
      ...this.#props,
      executor: this.#props.executor.withPlugin(plugin)
    });
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.queryNode, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    const compiledQuery = this.compile();
    const result = await this.#props.executor.executeQuery(compiledQuery, this.#props.queryId);
    const { adapter } = this.#props.executor;
    const query = compiledQuery.query;
    if (query.returning && adapter.supportsReturning || query.output && adapter.supportsOutput) {
      return result.rows;
    }
    return [
      new UpdateResult(result.numAffectedRows ?? result.numUpdatedOrDeletedRows ?? BigInt(0), result.numChangedRows)
    ];
  }
  async executeTakeFirst() {
    const [result] = await this.execute();
    return result;
  }
  async executeTakeFirstOrThrow(errorConstructor = NoResultError) {
    const result = await this.executeTakeFirst();
    if (result === undefined) {
      const error = isNoResultErrorConstructor(errorConstructor) ? new errorConstructor(this.toOperationNode()) : errorConstructor(this.toOperationNode());
      throw error;
    }
    return result;
  }
  async* stream(chunkSize = 100) {
    const compiledQuery = this.compile();
    const stream = this.#props.executor.stream(compiledQuery, chunkSize, this.#props.queryId);
    for await (const item of stream) {
      yield* item.rows;
    }
  }
  async explain(format, options) {
    const builder = new UpdateQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithExplain(this.#props.queryNode, format, options)
    });
    return await builder.execute();
  }
}
preventAwait(UpdateQueryBuilder, "don't await UpdateQueryBuilder instances directly. To execute the query you need to call `execute` or `executeTakeFirst`.");

// node_modules/kysely/dist/esm/operation-node/common-table-expression-name-node.js
var CommonTableExpressionNameNode = freeze({
  is(node) {
    return node.kind === "CommonTableExpressionNameNode";
  },
  create(tableName, columnNames) {
    return freeze({
      kind: "CommonTableExpressionNameNode",
      table: TableNode.create(tableName),
      columns: columnNames ? freeze(columnNames.map(ColumnNode.create)) : undefined
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/common-table-expression-node.js
var CommonTableExpressionNode = freeze({
  is(node) {
    return node.kind === "CommonTableExpressionNode";
  },
  create(name, expression) {
    return freeze({
      kind: "CommonTableExpressionNode",
      name,
      expression
    });
  },
  cloneWith(node, props) {
    return freeze({
      ...node,
      ...props
    });
  }
});

// node_modules/kysely/dist/esm/query-builder/cte-builder.js
class CTEBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  materialized() {
    return new CTEBuilder({
      ...this.#props,
      node: CommonTableExpressionNode.cloneWith(this.#props.node, {
        materialized: true
      })
    });
  }
  notMaterialized() {
    return new CTEBuilder({
      ...this.#props,
      node: CommonTableExpressionNode.cloneWith(this.#props.node, {
        materialized: false
      })
    });
  }
  toOperationNode() {
    return this.#props.node;
  }
}
preventAwait(CTEBuilder, "don't await CTEBuilder instances. They are never executed directly and are always just a part of a query.");

// node_modules/kysely/dist/esm/parser/with-parser.js
function parseCommonTableExpression(nameOrBuilderCallback, expression) {
  const expressionNode = expression(createQueryCreator()).toOperationNode();
  if (isFunction(nameOrBuilderCallback)) {
    return nameOrBuilderCallback(cteBuilderFactory(expressionNode)).toOperationNode();
  }
  return CommonTableExpressionNode.create(parseCommonTableExpressionName(nameOrBuilderCallback), expressionNode);
}
function cteBuilderFactory(expressionNode) {
  return (name) => {
    return new CTEBuilder({
      node: CommonTableExpressionNode.create(parseCommonTableExpressionName(name), expressionNode)
    });
  };
}
function parseCommonTableExpressionName(name) {
  if (name.includes("(")) {
    const parts = name.split(/[\(\)]/);
    const table = parts[0];
    const columns = parts[1].split(",").map((it) => it.trim());
    return CommonTableExpressionNameNode.create(table, columns);
  } else {
    return CommonTableExpressionNameNode.create(name);
  }
}

// node_modules/kysely/dist/esm/operation-node/with-node.js
var WithNode = freeze({
  is(node) {
    return node.kind === "WithNode";
  },
  create(expression, params) {
    return freeze({
      kind: "WithNode",
      expressions: freeze([expression]),
      ...params
    });
  },
  cloneWithExpression(withNode, expression) {
    return freeze({
      ...withNode,
      expressions: freeze([...withNode.expressions, expression])
    });
  }
});

// node_modules/kysely/dist/esm/util/random-string.js
var CHARS = [
  "A",
  "B",
  "C",
  "D",
  "E",
  "F",
  "G",
  "H",
  "I",
  "J",
  "K",
  "L",
  "M",
  "N",
  "O",
  "P",
  "Q",
  "R",
  "S",
  "T",
  "U",
  "V",
  "W",
  "X",
  "Y",
  "Z",
  "a",
  "b",
  "c",
  "d",
  "e",
  "f",
  "g",
  "h",
  "i",
  "j",
  "k",
  "l",
  "m",
  "n",
  "o",
  "p",
  "q",
  "r",
  "s",
  "t",
  "u",
  "v",
  "w",
  "x",
  "y",
  "z",
  "0",
  "1",
  "2",
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9"
];
function randomString(length) {
  let chars = "";
  for (let i = 0;i < length; ++i) {
    chars += randomChar();
  }
  return chars;
}
function randomChar() {
  return CHARS[~~(Math.random() * CHARS.length)];
}

// node_modules/kysely/dist/esm/util/query-id.js
function createQueryId() {
  return new LazyQueryId;
}

class LazyQueryId {
  #queryId;
  get queryId() {
    if (this.#queryId === undefined) {
      this.#queryId = randomString(8);
    }
    return this.#queryId;
  }
}

// node_modules/kysely/dist/esm/util/require-all-props.js
function requireAllProps(obj) {
  return obj;
}

// node_modules/kysely/dist/esm/operation-node/operation-node-transformer.js
class OperationNodeTransformer {
  nodeStack = [];
  #transformers = freeze({
    AliasNode: this.transformAlias.bind(this),
    ColumnNode: this.transformColumn.bind(this),
    IdentifierNode: this.transformIdentifier.bind(this),
    SchemableIdentifierNode: this.transformSchemableIdentifier.bind(this),
    RawNode: this.transformRaw.bind(this),
    ReferenceNode: this.transformReference.bind(this),
    SelectQueryNode: this.transformSelectQuery.bind(this),
    SelectionNode: this.transformSelection.bind(this),
    TableNode: this.transformTable.bind(this),
    FromNode: this.transformFrom.bind(this),
    SelectAllNode: this.transformSelectAll.bind(this),
    AndNode: this.transformAnd.bind(this),
    OrNode: this.transformOr.bind(this),
    ValueNode: this.transformValue.bind(this),
    ValueListNode: this.transformValueList.bind(this),
    PrimitiveValueListNode: this.transformPrimitiveValueList.bind(this),
    ParensNode: this.transformParens.bind(this),
    JoinNode: this.transformJoin.bind(this),
    OperatorNode: this.transformOperator.bind(this),
    WhereNode: this.transformWhere.bind(this),
    InsertQueryNode: this.transformInsertQuery.bind(this),
    DeleteQueryNode: this.transformDeleteQuery.bind(this),
    ReturningNode: this.transformReturning.bind(this),
    CreateTableNode: this.transformCreateTable.bind(this),
    AddColumnNode: this.transformAddColumn.bind(this),
    ColumnDefinitionNode: this.transformColumnDefinition.bind(this),
    DropTableNode: this.transformDropTable.bind(this),
    DataTypeNode: this.transformDataType.bind(this),
    OrderByNode: this.transformOrderBy.bind(this),
    OrderByItemNode: this.transformOrderByItem.bind(this),
    GroupByNode: this.transformGroupBy.bind(this),
    GroupByItemNode: this.transformGroupByItem.bind(this),
    UpdateQueryNode: this.transformUpdateQuery.bind(this),
    ColumnUpdateNode: this.transformColumnUpdate.bind(this),
    LimitNode: this.transformLimit.bind(this),
    OffsetNode: this.transformOffset.bind(this),
    OnConflictNode: this.transformOnConflict.bind(this),
    OnDuplicateKeyNode: this.transformOnDuplicateKey.bind(this),
    CreateIndexNode: this.transformCreateIndex.bind(this),
    DropIndexNode: this.transformDropIndex.bind(this),
    ListNode: this.transformList.bind(this),
    PrimaryKeyConstraintNode: this.transformPrimaryKeyConstraint.bind(this),
    UniqueConstraintNode: this.transformUniqueConstraint.bind(this),
    ReferencesNode: this.transformReferences.bind(this),
    CheckConstraintNode: this.transformCheckConstraint.bind(this),
    WithNode: this.transformWith.bind(this),
    CommonTableExpressionNode: this.transformCommonTableExpression.bind(this),
    CommonTableExpressionNameNode: this.transformCommonTableExpressionName.bind(this),
    HavingNode: this.transformHaving.bind(this),
    CreateSchemaNode: this.transformCreateSchema.bind(this),
    DropSchemaNode: this.transformDropSchema.bind(this),
    AlterTableNode: this.transformAlterTable.bind(this),
    DropColumnNode: this.transformDropColumn.bind(this),
    RenameColumnNode: this.transformRenameColumn.bind(this),
    AlterColumnNode: this.transformAlterColumn.bind(this),
    ModifyColumnNode: this.transformModifyColumn.bind(this),
    AddConstraintNode: this.transformAddConstraint.bind(this),
    DropConstraintNode: this.transformDropConstraint.bind(this),
    ForeignKeyConstraintNode: this.transformForeignKeyConstraint.bind(this),
    CreateViewNode: this.transformCreateView.bind(this),
    DropViewNode: this.transformDropView.bind(this),
    GeneratedNode: this.transformGenerated.bind(this),
    DefaultValueNode: this.transformDefaultValue.bind(this),
    OnNode: this.transformOn.bind(this),
    ValuesNode: this.transformValues.bind(this),
    SelectModifierNode: this.transformSelectModifier.bind(this),
    CreateTypeNode: this.transformCreateType.bind(this),
    DropTypeNode: this.transformDropType.bind(this),
    ExplainNode: this.transformExplain.bind(this),
    DefaultInsertValueNode: this.transformDefaultInsertValue.bind(this),
    AggregateFunctionNode: this.transformAggregateFunction.bind(this),
    OverNode: this.transformOver.bind(this),
    PartitionByNode: this.transformPartitionBy.bind(this),
    PartitionByItemNode: this.transformPartitionByItem.bind(this),
    SetOperationNode: this.transformSetOperation.bind(this),
    BinaryOperationNode: this.transformBinaryOperation.bind(this),
    UnaryOperationNode: this.transformUnaryOperation.bind(this),
    UsingNode: this.transformUsing.bind(this),
    FunctionNode: this.transformFunction.bind(this),
    CaseNode: this.transformCase.bind(this),
    WhenNode: this.transformWhen.bind(this),
    JSONReferenceNode: this.transformJSONReference.bind(this),
    JSONPathNode: this.transformJSONPath.bind(this),
    JSONPathLegNode: this.transformJSONPathLeg.bind(this),
    JSONOperatorChainNode: this.transformJSONOperatorChain.bind(this),
    TupleNode: this.transformTuple.bind(this),
    MergeQueryNode: this.transformMergeQuery.bind(this),
    MatchedNode: this.transformMatched.bind(this),
    AddIndexNode: this.transformAddIndex.bind(this),
    CastNode: this.transformCast.bind(this),
    FetchNode: this.transformFetch.bind(this),
    TopNode: this.transformTop.bind(this),
    OutputNode: this.transformOutput.bind(this)
  });
  transformNode(node) {
    if (!node) {
      return node;
    }
    this.nodeStack.push(node);
    const out = this.transformNodeImpl(node);
    this.nodeStack.pop();
    return freeze(out);
  }
  transformNodeImpl(node) {
    return this.#transformers[node.kind](node);
  }
  transformNodeList(list) {
    if (!list) {
      return list;
    }
    return freeze(list.map((node) => this.transformNode(node)));
  }
  transformSelectQuery(node) {
    return requireAllProps({
      kind: "SelectQueryNode",
      from: this.transformNode(node.from),
      selections: this.transformNodeList(node.selections),
      distinctOn: this.transformNodeList(node.distinctOn),
      joins: this.transformNodeList(node.joins),
      groupBy: this.transformNode(node.groupBy),
      orderBy: this.transformNode(node.orderBy),
      where: this.transformNode(node.where),
      frontModifiers: this.transformNodeList(node.frontModifiers),
      endModifiers: this.transformNodeList(node.endModifiers),
      limit: this.transformNode(node.limit),
      offset: this.transformNode(node.offset),
      with: this.transformNode(node.with),
      having: this.transformNode(node.having),
      explain: this.transformNode(node.explain),
      setOperations: this.transformNodeList(node.setOperations),
      fetch: this.transformNode(node.fetch),
      top: this.transformNode(node.top)
    });
  }
  transformSelection(node) {
    return requireAllProps({
      kind: "SelectionNode",
      selection: this.transformNode(node.selection)
    });
  }
  transformColumn(node) {
    return requireAllProps({
      kind: "ColumnNode",
      column: this.transformNode(node.column)
    });
  }
  transformAlias(node) {
    return requireAllProps({
      kind: "AliasNode",
      node: this.transformNode(node.node),
      alias: this.transformNode(node.alias)
    });
  }
  transformTable(node) {
    return requireAllProps({
      kind: "TableNode",
      table: this.transformNode(node.table)
    });
  }
  transformFrom(node) {
    return requireAllProps({
      kind: "FromNode",
      froms: this.transformNodeList(node.froms)
    });
  }
  transformReference(node) {
    return requireAllProps({
      kind: "ReferenceNode",
      column: this.transformNode(node.column),
      table: this.transformNode(node.table)
    });
  }
  transformAnd(node) {
    return requireAllProps({
      kind: "AndNode",
      left: this.transformNode(node.left),
      right: this.transformNode(node.right)
    });
  }
  transformOr(node) {
    return requireAllProps({
      kind: "OrNode",
      left: this.transformNode(node.left),
      right: this.transformNode(node.right)
    });
  }
  transformValueList(node) {
    return requireAllProps({
      kind: "ValueListNode",
      values: this.transformNodeList(node.values)
    });
  }
  transformParens(node) {
    return requireAllProps({
      kind: "ParensNode",
      node: this.transformNode(node.node)
    });
  }
  transformJoin(node) {
    return requireAllProps({
      kind: "JoinNode",
      joinType: node.joinType,
      table: this.transformNode(node.table),
      on: this.transformNode(node.on)
    });
  }
  transformRaw(node) {
    return requireAllProps({
      kind: "RawNode",
      sqlFragments: freeze([...node.sqlFragments]),
      parameters: this.transformNodeList(node.parameters)
    });
  }
  transformWhere(node) {
    return requireAllProps({
      kind: "WhereNode",
      where: this.transformNode(node.where)
    });
  }
  transformInsertQuery(node) {
    return requireAllProps({
      kind: "InsertQueryNode",
      into: this.transformNode(node.into),
      columns: this.transformNodeList(node.columns),
      values: this.transformNode(node.values),
      returning: this.transformNode(node.returning),
      onConflict: this.transformNode(node.onConflict),
      onDuplicateKey: this.transformNode(node.onDuplicateKey),
      with: this.transformNode(node.with),
      ignore: node.ignore,
      replace: node.replace,
      explain: this.transformNode(node.explain),
      defaultValues: node.defaultValues,
      top: this.transformNode(node.top),
      output: this.transformNode(node.output)
    });
  }
  transformValues(node) {
    return requireAllProps({
      kind: "ValuesNode",
      values: this.transformNodeList(node.values)
    });
  }
  transformDeleteQuery(node) {
    return requireAllProps({
      kind: "DeleteQueryNode",
      from: this.transformNode(node.from),
      using: this.transformNode(node.using),
      joins: this.transformNodeList(node.joins),
      where: this.transformNode(node.where),
      returning: this.transformNode(node.returning),
      with: this.transformNode(node.with),
      orderBy: this.transformNode(node.orderBy),
      limit: this.transformNode(node.limit),
      explain: this.transformNode(node.explain),
      top: this.transformNode(node.top),
      output: this.transformNode(node.output)
    });
  }
  transformReturning(node) {
    return requireAllProps({
      kind: "ReturningNode",
      selections: this.transformNodeList(node.selections)
    });
  }
  transformCreateTable(node) {
    return requireAllProps({
      kind: "CreateTableNode",
      table: this.transformNode(node.table),
      columns: this.transformNodeList(node.columns),
      constraints: this.transformNodeList(node.constraints),
      temporary: node.temporary,
      ifNotExists: node.ifNotExists,
      onCommit: node.onCommit,
      frontModifiers: this.transformNodeList(node.frontModifiers),
      endModifiers: this.transformNodeList(node.endModifiers),
      selectQuery: this.transformNode(node.selectQuery)
    });
  }
  transformColumnDefinition(node) {
    return requireAllProps({
      kind: "ColumnDefinitionNode",
      column: this.transformNode(node.column),
      dataType: this.transformNode(node.dataType),
      references: this.transformNode(node.references),
      primaryKey: node.primaryKey,
      autoIncrement: node.autoIncrement,
      unique: node.unique,
      notNull: node.notNull,
      unsigned: node.unsigned,
      defaultTo: this.transformNode(node.defaultTo),
      check: this.transformNode(node.check),
      generated: this.transformNode(node.generated),
      frontModifiers: this.transformNodeList(node.frontModifiers),
      endModifiers: this.transformNodeList(node.endModifiers),
      nullsNotDistinct: node.nullsNotDistinct,
      identity: node.identity,
      ifNotExists: node.ifNotExists
    });
  }
  transformAddColumn(node) {
    return requireAllProps({
      kind: "AddColumnNode",
      column: this.transformNode(node.column)
    });
  }
  transformDropTable(node) {
    return requireAllProps({
      kind: "DropTableNode",
      table: this.transformNode(node.table),
      ifExists: node.ifExists,
      cascade: node.cascade
    });
  }
  transformOrderBy(node) {
    return requireAllProps({
      kind: "OrderByNode",
      items: this.transformNodeList(node.items)
    });
  }
  transformOrderByItem(node) {
    return requireAllProps({
      kind: "OrderByItemNode",
      orderBy: this.transformNode(node.orderBy),
      direction: this.transformNode(node.direction)
    });
  }
  transformGroupBy(node) {
    return requireAllProps({
      kind: "GroupByNode",
      items: this.transformNodeList(node.items)
    });
  }
  transformGroupByItem(node) {
    return requireAllProps({
      kind: "GroupByItemNode",
      groupBy: this.transformNode(node.groupBy)
    });
  }
  transformUpdateQuery(node) {
    return requireAllProps({
      kind: "UpdateQueryNode",
      table: this.transformNode(node.table),
      from: this.transformNode(node.from),
      joins: this.transformNodeList(node.joins),
      where: this.transformNode(node.where),
      updates: this.transformNodeList(node.updates),
      returning: this.transformNode(node.returning),
      with: this.transformNode(node.with),
      explain: this.transformNode(node.explain),
      limit: this.transformNode(node.limit),
      top: this.transformNode(node.top),
      output: this.transformNode(node.output)
    });
  }
  transformColumnUpdate(node) {
    return requireAllProps({
      kind: "ColumnUpdateNode",
      column: this.transformNode(node.column),
      value: this.transformNode(node.value)
    });
  }
  transformLimit(node) {
    return requireAllProps({
      kind: "LimitNode",
      limit: this.transformNode(node.limit)
    });
  }
  transformOffset(node) {
    return requireAllProps({
      kind: "OffsetNode",
      offset: this.transformNode(node.offset)
    });
  }
  transformOnConflict(node) {
    return requireAllProps({
      kind: "OnConflictNode",
      columns: this.transformNodeList(node.columns),
      constraint: this.transformNode(node.constraint),
      indexExpression: this.transformNode(node.indexExpression),
      indexWhere: this.transformNode(node.indexWhere),
      updates: this.transformNodeList(node.updates),
      updateWhere: this.transformNode(node.updateWhere),
      doNothing: node.doNothing
    });
  }
  transformOnDuplicateKey(node) {
    return requireAllProps({
      kind: "OnDuplicateKeyNode",
      updates: this.transformNodeList(node.updates)
    });
  }
  transformCreateIndex(node) {
    return requireAllProps({
      kind: "CreateIndexNode",
      name: this.transformNode(node.name),
      table: this.transformNode(node.table),
      columns: this.transformNodeList(node.columns),
      unique: node.unique,
      using: this.transformNode(node.using),
      ifNotExists: node.ifNotExists,
      where: this.transformNode(node.where),
      nullsNotDistinct: node.nullsNotDistinct
    });
  }
  transformList(node) {
    return requireAllProps({
      kind: "ListNode",
      items: this.transformNodeList(node.items)
    });
  }
  transformDropIndex(node) {
    return requireAllProps({
      kind: "DropIndexNode",
      name: this.transformNode(node.name),
      table: this.transformNode(node.table),
      ifExists: node.ifExists,
      cascade: node.cascade
    });
  }
  transformPrimaryKeyConstraint(node) {
    return requireAllProps({
      kind: "PrimaryKeyConstraintNode",
      columns: this.transformNodeList(node.columns),
      name: this.transformNode(node.name)
    });
  }
  transformUniqueConstraint(node) {
    return requireAllProps({
      kind: "UniqueConstraintNode",
      columns: this.transformNodeList(node.columns),
      name: this.transformNode(node.name),
      nullsNotDistinct: node.nullsNotDistinct
    });
  }
  transformForeignKeyConstraint(node) {
    return requireAllProps({
      kind: "ForeignKeyConstraintNode",
      columns: this.transformNodeList(node.columns),
      references: this.transformNode(node.references),
      name: this.transformNode(node.name),
      onDelete: node.onDelete,
      onUpdate: node.onUpdate
    });
  }
  transformSetOperation(node) {
    return requireAllProps({
      kind: "SetOperationNode",
      operator: node.operator,
      expression: this.transformNode(node.expression),
      all: node.all
    });
  }
  transformReferences(node) {
    return requireAllProps({
      kind: "ReferencesNode",
      table: this.transformNode(node.table),
      columns: this.transformNodeList(node.columns),
      onDelete: node.onDelete,
      onUpdate: node.onUpdate
    });
  }
  transformCheckConstraint(node) {
    return requireAllProps({
      kind: "CheckConstraintNode",
      expression: this.transformNode(node.expression),
      name: this.transformNode(node.name)
    });
  }
  transformWith(node) {
    return requireAllProps({
      kind: "WithNode",
      expressions: this.transformNodeList(node.expressions),
      recursive: node.recursive
    });
  }
  transformCommonTableExpression(node) {
    return requireAllProps({
      kind: "CommonTableExpressionNode",
      name: this.transformNode(node.name),
      materialized: node.materialized,
      expression: this.transformNode(node.expression)
    });
  }
  transformCommonTableExpressionName(node) {
    return requireAllProps({
      kind: "CommonTableExpressionNameNode",
      table: this.transformNode(node.table),
      columns: this.transformNodeList(node.columns)
    });
  }
  transformHaving(node) {
    return requireAllProps({
      kind: "HavingNode",
      having: this.transformNode(node.having)
    });
  }
  transformCreateSchema(node) {
    return requireAllProps({
      kind: "CreateSchemaNode",
      schema: this.transformNode(node.schema),
      ifNotExists: node.ifNotExists
    });
  }
  transformDropSchema(node) {
    return requireAllProps({
      kind: "DropSchemaNode",
      schema: this.transformNode(node.schema),
      ifExists: node.ifExists,
      cascade: node.cascade
    });
  }
  transformAlterTable(node) {
    return requireAllProps({
      kind: "AlterTableNode",
      table: this.transformNode(node.table),
      renameTo: this.transformNode(node.renameTo),
      setSchema: this.transformNode(node.setSchema),
      columnAlterations: this.transformNodeList(node.columnAlterations),
      addConstraint: this.transformNode(node.addConstraint),
      dropConstraint: this.transformNode(node.dropConstraint),
      addIndex: this.transformNode(node.addIndex),
      dropIndex: this.transformNode(node.dropIndex)
    });
  }
  transformDropColumn(node) {
    return requireAllProps({
      kind: "DropColumnNode",
      column: this.transformNode(node.column)
    });
  }
  transformRenameColumn(node) {
    return requireAllProps({
      kind: "RenameColumnNode",
      column: this.transformNode(node.column),
      renameTo: this.transformNode(node.renameTo)
    });
  }
  transformAlterColumn(node) {
    return requireAllProps({
      kind: "AlterColumnNode",
      column: this.transformNode(node.column),
      dataType: this.transformNode(node.dataType),
      dataTypeExpression: this.transformNode(node.dataTypeExpression),
      setDefault: this.transformNode(node.setDefault),
      dropDefault: node.dropDefault,
      setNotNull: node.setNotNull,
      dropNotNull: node.dropNotNull
    });
  }
  transformModifyColumn(node) {
    return requireAllProps({
      kind: "ModifyColumnNode",
      column: this.transformNode(node.column)
    });
  }
  transformAddConstraint(node) {
    return requireAllProps({
      kind: "AddConstraintNode",
      constraint: this.transformNode(node.constraint)
    });
  }
  transformDropConstraint(node) {
    return requireAllProps({
      kind: "DropConstraintNode",
      constraintName: this.transformNode(node.constraintName),
      ifExists: node.ifExists,
      modifier: node.modifier
    });
  }
  transformCreateView(node) {
    return requireAllProps({
      kind: "CreateViewNode",
      name: this.transformNode(node.name),
      temporary: node.temporary,
      orReplace: node.orReplace,
      ifNotExists: node.ifNotExists,
      materialized: node.materialized,
      columns: this.transformNodeList(node.columns),
      as: this.transformNode(node.as)
    });
  }
  transformDropView(node) {
    return requireAllProps({
      kind: "DropViewNode",
      name: this.transformNode(node.name),
      ifExists: node.ifExists,
      materialized: node.materialized,
      cascade: node.cascade
    });
  }
  transformGenerated(node) {
    return requireAllProps({
      kind: "GeneratedNode",
      byDefault: node.byDefault,
      always: node.always,
      identity: node.identity,
      stored: node.stored,
      expression: this.transformNode(node.expression)
    });
  }
  transformDefaultValue(node) {
    return requireAllProps({
      kind: "DefaultValueNode",
      defaultValue: this.transformNode(node.defaultValue)
    });
  }
  transformOn(node) {
    return requireAllProps({
      kind: "OnNode",
      on: this.transformNode(node.on)
    });
  }
  transformSelectModifier(node) {
    return requireAllProps({
      kind: "SelectModifierNode",
      modifier: node.modifier,
      rawModifier: this.transformNode(node.rawModifier),
      of: this.transformNodeList(node.of)
    });
  }
  transformCreateType(node) {
    return requireAllProps({
      kind: "CreateTypeNode",
      name: this.transformNode(node.name),
      enum: this.transformNode(node.enum)
    });
  }
  transformDropType(node) {
    return requireAllProps({
      kind: "DropTypeNode",
      name: this.transformNode(node.name),
      ifExists: node.ifExists
    });
  }
  transformExplain(node) {
    return requireAllProps({
      kind: "ExplainNode",
      format: node.format,
      options: this.transformNode(node.options)
    });
  }
  transformSchemableIdentifier(node) {
    return requireAllProps({
      kind: "SchemableIdentifierNode",
      schema: this.transformNode(node.schema),
      identifier: this.transformNode(node.identifier)
    });
  }
  transformAggregateFunction(node) {
    return requireAllProps({
      kind: "AggregateFunctionNode",
      aggregated: this.transformNodeList(node.aggregated),
      distinct: node.distinct,
      filter: this.transformNode(node.filter),
      func: node.func,
      over: this.transformNode(node.over)
    });
  }
  transformOver(node) {
    return requireAllProps({
      kind: "OverNode",
      orderBy: this.transformNode(node.orderBy),
      partitionBy: this.transformNode(node.partitionBy)
    });
  }
  transformPartitionBy(node) {
    return requireAllProps({
      kind: "PartitionByNode",
      items: this.transformNodeList(node.items)
    });
  }
  transformPartitionByItem(node) {
    return requireAllProps({
      kind: "PartitionByItemNode",
      partitionBy: this.transformNode(node.partitionBy)
    });
  }
  transformBinaryOperation(node) {
    return requireAllProps({
      kind: "BinaryOperationNode",
      leftOperand: this.transformNode(node.leftOperand),
      operator: this.transformNode(node.operator),
      rightOperand: this.transformNode(node.rightOperand)
    });
  }
  transformUnaryOperation(node) {
    return requireAllProps({
      kind: "UnaryOperationNode",
      operator: this.transformNode(node.operator),
      operand: this.transformNode(node.operand)
    });
  }
  transformUsing(node) {
    return requireAllProps({
      kind: "UsingNode",
      tables: this.transformNodeList(node.tables)
    });
  }
  transformFunction(node) {
    return requireAllProps({
      kind: "FunctionNode",
      func: node.func,
      arguments: this.transformNodeList(node.arguments)
    });
  }
  transformCase(node) {
    return requireAllProps({
      kind: "CaseNode",
      value: this.transformNode(node.value),
      when: this.transformNodeList(node.when),
      else: this.transformNode(node.else),
      isStatement: node.isStatement
    });
  }
  transformWhen(node) {
    return requireAllProps({
      kind: "WhenNode",
      condition: this.transformNode(node.condition),
      result: this.transformNode(node.result)
    });
  }
  transformJSONReference(node) {
    return requireAllProps({
      kind: "JSONReferenceNode",
      reference: this.transformNode(node.reference),
      traversal: this.transformNode(node.traversal)
    });
  }
  transformJSONPath(node) {
    return requireAllProps({
      kind: "JSONPathNode",
      inOperator: this.transformNode(node.inOperator),
      pathLegs: this.transformNodeList(node.pathLegs)
    });
  }
  transformJSONPathLeg(node) {
    return requireAllProps({
      kind: "JSONPathLegNode",
      type: node.type,
      value: node.value
    });
  }
  transformJSONOperatorChain(node) {
    return requireAllProps({
      kind: "JSONOperatorChainNode",
      operator: this.transformNode(node.operator),
      values: this.transformNodeList(node.values)
    });
  }
  transformTuple(node) {
    return requireAllProps({
      kind: "TupleNode",
      values: this.transformNodeList(node.values)
    });
  }
  transformMergeQuery(node) {
    return requireAllProps({
      kind: "MergeQueryNode",
      into: this.transformNode(node.into),
      using: this.transformNode(node.using),
      whens: this.transformNodeList(node.whens),
      with: this.transformNode(node.with),
      top: this.transformNode(node.top),
      output: this.transformNode(node.output)
    });
  }
  transformMatched(node) {
    return requireAllProps({
      kind: "MatchedNode",
      not: node.not,
      bySource: node.bySource
    });
  }
  transformAddIndex(node) {
    return requireAllProps({
      kind: "AddIndexNode",
      name: this.transformNode(node.name),
      columns: this.transformNodeList(node.columns),
      unique: node.unique,
      using: this.transformNode(node.using),
      ifNotExists: node.ifNotExists
    });
  }
  transformCast(node) {
    return requireAllProps({
      kind: "CastNode",
      expression: this.transformNode(node.expression),
      dataType: this.transformNode(node.dataType)
    });
  }
  transformFetch(node) {
    return requireAllProps({
      kind: "FetchNode",
      rowCount: this.transformNode(node.rowCount),
      modifier: node.modifier
    });
  }
  transformTop(node) {
    return requireAllProps({
      kind: "TopNode",
      expression: node.expression,
      modifiers: node.modifiers
    });
  }
  transformOutput(node) {
    return requireAllProps({
      kind: "OutputNode",
      selections: this.transformNodeList(node.selections)
    });
  }
  transformDataType(node) {
    return node;
  }
  transformSelectAll(node) {
    return node;
  }
  transformIdentifier(node) {
    return node;
  }
  transformValue(node) {
    return node;
  }
  transformPrimitiveValueList(node) {
    return node;
  }
  transformOperator(node) {
    return node;
  }
  transformDefaultInsertValue(node) {
    return node;
  }
}

// node_modules/kysely/dist/esm/plugin/with-schema/with-schema-transformer.js
var ROOT_OPERATION_NODES = freeze({
  AlterTableNode: true,
  CreateIndexNode: true,
  CreateSchemaNode: true,
  CreateTableNode: true,
  CreateTypeNode: true,
  CreateViewNode: true,
  DeleteQueryNode: true,
  DropIndexNode: true,
  DropSchemaNode: true,
  DropTableNode: true,
  DropTypeNode: true,
  DropViewNode: true,
  InsertQueryNode: true,
  RawNode: true,
  SelectQueryNode: true,
  UpdateQueryNode: true,
  MergeQueryNode: true
});

class WithSchemaTransformer extends OperationNodeTransformer {
  #schema;
  #schemableIds = new Set;
  #ctes = new Set;
  constructor(schema) {
    super();
    this.#schema = schema;
  }
  transformNodeImpl(node) {
    if (!this.#isRootOperationNode(node)) {
      return super.transformNodeImpl(node);
    }
    const ctes = this.#collectCTEs(node);
    for (const cte of ctes) {
      this.#ctes.add(cte);
    }
    const tables = this.#collectSchemableIds(node);
    for (const table of tables) {
      this.#schemableIds.add(table);
    }
    const transformed = super.transformNodeImpl(node);
    for (const table of tables) {
      this.#schemableIds.delete(table);
    }
    for (const cte of ctes) {
      this.#ctes.delete(cte);
    }
    return transformed;
  }
  transformSchemableIdentifier(node) {
    const transformed = super.transformSchemableIdentifier(node);
    if (transformed.schema || !this.#schemableIds.has(node.identifier.name)) {
      return transformed;
    }
    return {
      ...transformed,
      schema: IdentifierNode.create(this.#schema)
    };
  }
  transformReferences(node) {
    const transformed = super.transformReferences(node);
    if (transformed.table.table.schema) {
      return transformed;
    }
    return {
      ...transformed,
      table: TableNode.createWithSchema(this.#schema, transformed.table.table.identifier.name)
    };
  }
  #isRootOperationNode(node) {
    return node.kind in ROOT_OPERATION_NODES;
  }
  #collectSchemableIds(node) {
    const schemableIds = new Set;
    if ("name" in node && node.name && SchemableIdentifierNode.is(node.name)) {
      this.#collectSchemableId(node.name, schemableIds);
    }
    if ("from" in node && node.from) {
      for (const from of node.from.froms) {
        this.#collectSchemableIdsFromTableExpr(from, schemableIds);
      }
    }
    if ("into" in node && node.into) {
      this.#collectSchemableIdsFromTableExpr(node.into, schemableIds);
    }
    if ("table" in node && node.table) {
      this.#collectSchemableIdsFromTableExpr(node.table, schemableIds);
    }
    if ("joins" in node && node.joins) {
      for (const join of node.joins) {
        this.#collectSchemableIdsFromTableExpr(join.table, schemableIds);
      }
    }
    if ("using" in node && node.using) {
      this.#collectSchemableIdsFromTableExpr(node.using, schemableIds);
    }
    return schemableIds;
  }
  #collectCTEs(node) {
    const ctes = new Set;
    if ("with" in node && node.with) {
      this.#collectCTEIds(node.with, ctes);
    }
    return ctes;
  }
  #collectSchemableIdsFromTableExpr(node, schemableIds) {
    const table = TableNode.is(node) ? node : AliasNode.is(node) && TableNode.is(node.node) ? node.node : null;
    if (table) {
      this.#collectSchemableId(table.table, schemableIds);
    }
  }
  #collectSchemableId(node, schemableIds) {
    const id = node.identifier.name;
    if (!this.#schemableIds.has(id) && !this.#ctes.has(id)) {
      schemableIds.add(id);
    }
  }
  #collectCTEIds(node, ctes) {
    for (const expr of node.expressions) {
      const cteId = expr.name.table.table.identifier.name;
      if (!this.#ctes.has(cteId)) {
        ctes.add(cteId);
      }
    }
  }
}

// node_modules/kysely/dist/esm/plugin/with-schema/with-schema-plugin.js
class WithSchemaPlugin {
  #transformer;
  constructor(schema) {
    this.#transformer = new WithSchemaTransformer(schema);
  }
  transformQuery(args) {
    return this.#transformer.transformNode(args.node);
  }
  async transformResult(args) {
    return args.result;
  }
}

// node_modules/kysely/dist/esm/operation-node/matched-node.js
var MatchedNode = freeze({
  is(node) {
    return node.kind === "MatchedNode";
  },
  create(not, bySource = false) {
    return freeze({
      kind: "MatchedNode",
      not,
      bySource
    });
  }
});

// node_modules/kysely/dist/esm/parser/merge-parser.js
function parseMergeWhen(type, args, refRight) {
  return WhenNode.create(parseFilterList([
    MatchedNode.create(!type.isMatched, type.bySource),
    ...args && args.length > 0 ? [
      args.length === 3 && refRight ? parseReferentialBinaryOperation(args[0], args[1], args[2]) : parseValueBinaryOperationOrExpression(args)
    ] : []
  ], "and", false));
}
function parseMergeThen(result) {
  if (isString(result)) {
    return RawNode.create([result], []);
  }
  if (isOperationNodeSource(result)) {
    return result.toOperationNode();
  }
  return result;
}

// node_modules/kysely/dist/esm/util/deferred.js
class Deferred {
  #promise;
  #resolve;
  #reject;
  constructor() {
    this.#promise = new Promise((resolve, reject) => {
      this.#reject = reject;
      this.#resolve = resolve;
    });
  }
  get promise() {
    return this.#promise;
  }
  resolve = (value) => {
    if (this.#resolve) {
      this.#resolve(value);
    }
  };
  reject = (reason) => {
    if (this.#reject) {
      this.#reject(reason);
    }
  };
}

// node_modules/kysely/dist/esm/util/log-once.js
var LOGGED_MESSAGES = new Set;
function logOnce(message) {
  if (LOGGED_MESSAGES.has(message)) {
    return;
  }
  LOGGED_MESSAGES.add(message);
  console.log(message);
}

// node_modules/kysely/dist/esm/query-executor/query-executor-base.js
var NO_PLUGINS = freeze([]);

class QueryExecutorBase {
  #plugins;
  constructor(plugins = NO_PLUGINS) {
    this.#plugins = plugins;
  }
  get plugins() {
    return this.#plugins;
  }
  transformQuery(node, queryId) {
    for (const plugin of this.#plugins) {
      const transformedNode = plugin.transformQuery({ node, queryId });
      if (transformedNode.kind === node.kind) {
        node = transformedNode;
      } else {
        throw new Error([
          `KyselyPlugin.transformQuery must return a node`,
          `of the same kind that was given to it.`,
          `The plugin was given a ${node.kind}`,
          `but it returned a ${transformedNode.kind}`
        ].join(" "));
      }
    }
    return node;
  }
  async executeQuery(compiledQuery, queryId) {
    return await this.provideConnection(async (connection) => {
      const result = await connection.executeQuery(compiledQuery);
      const transformedResult = await this.#transformResult(result, queryId);
      warnOfOutdatedDriverOrPlugins(result, transformedResult);
      return transformedResult;
    });
  }
  async* stream(compiledQuery, chunkSize, queryId) {
    const connectionDefer = new Deferred;
    const connectionReleaseDefer = new Deferred;
    this.provideConnection(async (connection2) => {
      connectionDefer.resolve(connection2);
      return await connectionReleaseDefer.promise;
    }).catch((ex) => connectionDefer.reject(ex));
    const connection = await connectionDefer.promise;
    try {
      for await (const result of connection.streamQuery(compiledQuery, chunkSize)) {
        yield await this.#transformResult(result, queryId);
      }
    } finally {
      connectionReleaseDefer.resolve();
    }
  }
  async#transformResult(result, queryId) {
    for (const plugin of this.#plugins) {
      result = await plugin.transformResult({ result, queryId });
    }
    return result;
  }
}
function warnOfOutdatedDriverOrPlugins(result, transformedResult) {
  const { numAffectedRows } = result;
  if (numAffectedRows === undefined && result.numUpdatedOrDeletedRows === undefined || numAffectedRows !== undefined && transformedResult.numAffectedRows !== undefined) {
    return;
  }
  logOnce("kysely:warning: outdated driver/plugin detected! QueryResult.numUpdatedOrDeletedRows is deprecated and will be removed in a future release.");
}

// node_modules/kysely/dist/esm/query-executor/noop-query-executor.js
class NoopQueryExecutor extends QueryExecutorBase {
  get adapter() {
    throw new Error("this query cannot be compiled to SQL");
  }
  compileQuery() {
    throw new Error("this query cannot be compiled to SQL");
  }
  provideConnection() {
    throw new Error("this query cannot be executed");
  }
  withConnectionProvider() {
    throw new Error("this query cannot have a connection provider");
  }
  withPlugin(plugin) {
    return new NoopQueryExecutor([...this.plugins, plugin]);
  }
  withPlugins(plugins) {
    return new NoopQueryExecutor([...this.plugins, ...plugins]);
  }
  withPluginAtFront(plugin) {
    return new NoopQueryExecutor([plugin, ...this.plugins]);
  }
  withoutPlugins() {
    return new NoopQueryExecutor([]);
  }
}
var NOOP_QUERY_EXECUTOR = new NoopQueryExecutor;

// node_modules/kysely/dist/esm/query-builder/merge-result.js
class MergeResult {
  numChangedRows;
  constructor(numChangedRows) {
    this.numChangedRows = numChangedRows;
  }
}

// node_modules/kysely/dist/esm/query-builder/merge-query-builder.js
class MergeQueryBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  top(expression, modifiers) {
    return new MergeQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithTop(this.#props.queryNode, parseTop(expression, modifiers))
    });
  }
  using(...args) {
    return new WheneableMergeQueryBuilder({
      ...this.#props,
      queryNode: MergeQueryNode.cloneWithUsing(this.#props.queryNode, parseJoin("Using", args))
    });
  }
  output(args) {
    return new MergeQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithOutput(this.#props.queryNode, parseSelectArg(args))
    });
  }
  outputAll(table) {
    return new MergeQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithOutput(this.#props.queryNode, parseSelectAll(table))
    });
  }
}
preventAwait(MergeQueryBuilder, "don't await MergeQueryBuilder instances directly. To execute the query you need to call `execute` when available.");

class WheneableMergeQueryBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  top(expression, modifiers) {
    return new WheneableMergeQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithTop(this.#props.queryNode, parseTop(expression, modifiers))
    });
  }
  whenMatched() {
    return this.#whenMatched([]);
  }
  whenMatchedAnd(...args) {
    return this.#whenMatched(args);
  }
  whenMatchedAndRef(lhs, op, rhs) {
    return this.#whenMatched([lhs, op, rhs], true);
  }
  #whenMatched(args, refRight) {
    return new MatchedThenableMergeQueryBuilder({
      ...this.#props,
      queryNode: MergeQueryNode.cloneWithWhen(this.#props.queryNode, parseMergeWhen({ isMatched: true }, args, refRight))
    });
  }
  whenNotMatched() {
    return this.#whenNotMatched([]);
  }
  whenNotMatchedAnd(...args) {
    return this.#whenNotMatched(args);
  }
  whenNotMatchedAndRef(lhs, op, rhs) {
    return this.#whenNotMatched([lhs, op, rhs], true);
  }
  whenNotMatchedBySource() {
    return this.#whenNotMatched([], false, true);
  }
  whenNotMatchedBySourceAnd(...args) {
    return this.#whenNotMatched(args, false, true);
  }
  whenNotMatchedBySourceAndRef(lhs, op, rhs) {
    return this.#whenNotMatched([lhs, op, rhs], true, true);
  }
  output(args) {
    return new WheneableMergeQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithOutput(this.#props.queryNode, parseSelectArg(args))
    });
  }
  outputAll(table) {
    return new WheneableMergeQueryBuilder({
      ...this.#props,
      queryNode: QueryNode.cloneWithOutput(this.#props.queryNode, parseSelectAll(table))
    });
  }
  #whenNotMatched(args, refRight = false, bySource = false) {
    const props = {
      ...this.#props,
      queryNode: MergeQueryNode.cloneWithWhen(this.#props.queryNode, parseMergeWhen({ isMatched: false, bySource }, args, refRight))
    };
    const Builder = bySource ? MatchedThenableMergeQueryBuilder : NotMatchedThenableMergeQueryBuilder;
    return new Builder(props);
  }
  $call(func) {
    return func(this);
  }
  $if(condition, func) {
    if (condition) {
      return func(this);
    }
    return new WheneableMergeQueryBuilder({
      ...this.#props
    });
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.queryNode, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    const compiledQuery = this.compile();
    const result = await this.#props.executor.executeQuery(compiledQuery, this.#props.queryId);
    if (compiledQuery.query.output && this.#props.executor.adapter.supportsOutput) {
      return result.rows;
    }
    return [new MergeResult(result.numAffectedRows)];
  }
  async executeTakeFirst() {
    const [result] = await this.execute();
    return result;
  }
  async executeTakeFirstOrThrow(errorConstructor = NoResultError) {
    const result = await this.executeTakeFirst();
    if (result === undefined) {
      const error = isNoResultErrorConstructor(errorConstructor) ? new errorConstructor(this.toOperationNode()) : errorConstructor(this.toOperationNode());
      throw error;
    }
    return result;
  }
}
preventAwait(WheneableMergeQueryBuilder, "don't await WheneableMergeQueryBuilder instances directly. To execute the query you need to call `execute`.");

class MatchedThenableMergeQueryBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  thenDelete() {
    return new WheneableMergeQueryBuilder({
      ...this.#props,
      queryNode: MergeQueryNode.cloneWithThen(this.#props.queryNode, parseMergeThen("delete"))
    });
  }
  thenDoNothing() {
    return new WheneableMergeQueryBuilder({
      ...this.#props,
      queryNode: MergeQueryNode.cloneWithThen(this.#props.queryNode, parseMergeThen("do nothing"))
    });
  }
  thenUpdate(set) {
    return new WheneableMergeQueryBuilder({
      ...this.#props,
      queryNode: MergeQueryNode.cloneWithThen(this.#props.queryNode, parseMergeThen(set(new UpdateQueryBuilder({
        queryId: this.#props.queryId,
        executor: NOOP_QUERY_EXECUTOR,
        queryNode: UpdateQueryNode.createWithoutTable()
      }))))
    });
  }
  thenUpdateSet(...args) {
    return this.thenUpdate((ub) => ub.set(...args));
  }
}
preventAwait(MatchedThenableMergeQueryBuilder, "don't await MatchedThenableMergeQueryBuilder instances directly. To execute the query you need to call `execute` when available.");

class NotMatchedThenableMergeQueryBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  thenDoNothing() {
    return new WheneableMergeQueryBuilder({
      ...this.#props,
      queryNode: MergeQueryNode.cloneWithThen(this.#props.queryNode, parseMergeThen("do nothing"))
    });
  }
  thenInsertValues(insert) {
    const [columns, values] = parseInsertExpression(insert);
    return new WheneableMergeQueryBuilder({
      ...this.#props,
      queryNode: MergeQueryNode.cloneWithThen(this.#props.queryNode, parseMergeThen(InsertQueryNode.cloneWith(InsertQueryNode.createWithoutInto(), {
        columns,
        values
      })))
    });
  }
}
preventAwait(NotMatchedThenableMergeQueryBuilder, "don't await NotMatchedThenableMergeQueryBuilder instances directly. To execute the query you need to call `execute` when available.");

// node_modules/kysely/dist/esm/query-creator.js
class QueryCreator {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  selectFrom(from) {
    return createSelectQueryBuilder({
      queryId: createQueryId(),
      executor: this.#props.executor,
      queryNode: SelectQueryNode.createFrom(parseTableExpressionOrList(from), this.#props.withNode)
    });
  }
  selectNoFrom(selection) {
    return createSelectQueryBuilder({
      queryId: createQueryId(),
      executor: this.#props.executor,
      queryNode: SelectQueryNode.cloneWithSelections(SelectQueryNode.create(this.#props.withNode), parseSelectArg(selection))
    });
  }
  insertInto(table) {
    return new InsertQueryBuilder({
      queryId: createQueryId(),
      executor: this.#props.executor,
      queryNode: InsertQueryNode.create(parseTable(table), this.#props.withNode)
    });
  }
  replaceInto(table) {
    return new InsertQueryBuilder({
      queryId: createQueryId(),
      executor: this.#props.executor,
      queryNode: InsertQueryNode.create(parseTable(table), this.#props.withNode, true)
    });
  }
  deleteFrom(tables) {
    return new DeleteQueryBuilder({
      queryId: createQueryId(),
      executor: this.#props.executor,
      queryNode: DeleteQueryNode.create(parseTableExpressionOrList(tables), this.#props.withNode)
    });
  }
  updateTable(table) {
    return new UpdateQueryBuilder({
      queryId: createQueryId(),
      executor: this.#props.executor,
      queryNode: UpdateQueryNode.create(parseTableExpression(table), this.#props.withNode)
    });
  }
  mergeInto(targetTable) {
    return new MergeQueryBuilder({
      queryId: createQueryId(),
      executor: this.#props.executor,
      queryNode: MergeQueryNode.create(parseAliasedTable(targetTable), this.#props.withNode)
    });
  }
  with(nameOrBuilder, expression) {
    const cte = parseCommonTableExpression(nameOrBuilder, expression);
    return new QueryCreator({
      ...this.#props,
      withNode: this.#props.withNode ? WithNode.cloneWithExpression(this.#props.withNode, cte) : WithNode.create(cte)
    });
  }
  withRecursive(nameOrBuilder, expression) {
    const cte = parseCommonTableExpression(nameOrBuilder, expression);
    return new QueryCreator({
      ...this.#props,
      withNode: this.#props.withNode ? WithNode.cloneWithExpression(this.#props.withNode, cte) : WithNode.create(cte, { recursive: true })
    });
  }
  withPlugin(plugin) {
    return new QueryCreator({
      ...this.#props,
      executor: this.#props.executor.withPlugin(plugin)
    });
  }
  withoutPlugins() {
    return new QueryCreator({
      ...this.#props,
      executor: this.#props.executor.withoutPlugins()
    });
  }
  withSchema(schema) {
    return new QueryCreator({
      ...this.#props,
      executor: this.#props.executor.withPluginAtFront(new WithSchemaPlugin(schema))
    });
  }
}

// node_modules/kysely/dist/esm/parser/parse-utils.js
function createQueryCreator() {
  return new QueryCreator({
    executor: NOOP_QUERY_EXECUTOR
  });
}
function createJoinBuilder(joinType, table) {
  return new JoinBuilder({
    joinNode: JoinNode.create(joinType, parseTableExpression(table))
  });
}
function createOverBuilder() {
  return new OverBuilder({
    overNode: OverNode.create()
  });
}

// node_modules/kysely/dist/esm/parser/join-parser.js
function parseJoin(joinType, args) {
  if (args.length === 3) {
    return parseSingleOnJoin(joinType, args[0], args[1], args[2]);
  } else if (args.length === 2) {
    return parseCallbackJoin(joinType, args[0], args[1]);
  } else {
    throw new Error("not implemented");
  }
}
function parseCallbackJoin(joinType, from, callback) {
  return callback(createJoinBuilder(joinType, from)).toOperationNode();
}
function parseSingleOnJoin(joinType, from, lhsColumn, rhsColumn) {
  return JoinNode.createWithOn(joinType, parseTableExpression(from), parseReferentialBinaryOperation(lhsColumn, "=", rhsColumn));
}

// node_modules/kysely/dist/esm/operation-node/offset-node.js
var OffsetNode = freeze({
  is(node) {
    return node.kind === "OffsetNode";
  },
  create(offset) {
    return freeze({
      kind: "OffsetNode",
      offset
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/group-by-item-node.js
var GroupByItemNode = freeze({
  is(node) {
    return node.kind === "GroupByItemNode";
  },
  create(groupBy) {
    return freeze({
      kind: "GroupByItemNode",
      groupBy
    });
  }
});

// node_modules/kysely/dist/esm/parser/group-by-parser.js
function parseGroupBy(groupBy) {
  groupBy = isFunction(groupBy) ? groupBy(expressionBuilder()) : groupBy;
  return parseReferenceExpressionOrList(groupBy).map(GroupByItemNode.create);
}

// node_modules/kysely/dist/esm/operation-node/set-operation-node.js
var SetOperationNode = freeze({
  is(node) {
    return node.kind === "SetOperationNode";
  },
  create(operator, expression, all) {
    return freeze({
      kind: "SetOperationNode",
      operator,
      expression,
      all
    });
  }
});

// node_modules/kysely/dist/esm/parser/set-operation-parser.js
function parseSetOperations(operator, expression, all) {
  if (isFunction(expression)) {
    expression = expression(createExpressionBuilder());
  }
  if (!isReadonlyArray(expression)) {
    expression = [expression];
  }
  return expression.map((expr) => SetOperationNode.create(operator, parseExpression(expr), all));
}

// node_modules/kysely/dist/esm/expression/expression-wrapper.js
class ExpressionWrapper {
  #node;
  constructor(node) {
    this.#node = node;
  }
  get expressionType() {
    return;
  }
  as(alias) {
    return new AliasedExpressionWrapper(this, alias);
  }
  or(...args) {
    return new OrWrapper(OrNode.create(this.#node, parseValueBinaryOperationOrExpression(args)));
  }
  and(...args) {
    return new AndWrapper(AndNode.create(this.#node, parseValueBinaryOperationOrExpression(args)));
  }
  $castTo() {
    return new ExpressionWrapper(this.#node);
  }
  $notNull() {
    return new ExpressionWrapper(this.#node);
  }
  toOperationNode() {
    return this.#node;
  }
}

class AliasedExpressionWrapper {
  #expr;
  #alias;
  constructor(expr, alias) {
    this.#expr = expr;
    this.#alias = alias;
  }
  get expression() {
    return this.#expr;
  }
  get alias() {
    return this.#alias;
  }
  toOperationNode() {
    return AliasNode.create(this.#expr.toOperationNode(), isOperationNodeSource(this.#alias) ? this.#alias.toOperationNode() : IdentifierNode.create(this.#alias));
  }
}

class OrWrapper {
  #node;
  constructor(node) {
    this.#node = node;
  }
  get expressionType() {
    return;
  }
  as(alias) {
    return new AliasedExpressionWrapper(this, alias);
  }
  or(...args) {
    return new OrWrapper(OrNode.create(this.#node, parseValueBinaryOperationOrExpression(args)));
  }
  $castTo() {
    return new OrWrapper(this.#node);
  }
  toOperationNode() {
    return ParensNode.create(this.#node);
  }
}

class AndWrapper {
  #node;
  constructor(node) {
    this.#node = node;
  }
  get expressionType() {
    return;
  }
  as(alias) {
    return new AliasedExpressionWrapper(this, alias);
  }
  and(...args) {
    return new AndWrapper(AndNode.create(this.#node, parseValueBinaryOperationOrExpression(args)));
  }
  $castTo() {
    return new AndWrapper(this.#node);
  }
  toOperationNode() {
    return ParensNode.create(this.#node);
  }
}

// node_modules/kysely/dist/esm/operation-node/fetch-node.js
var FetchNode = {
  is(node) {
    return node.kind === "FetchNode";
  },
  create(rowCount, modifier) {
    return {
      kind: "FetchNode",
      rowCount: ValueNode.create(rowCount),
      modifier
    };
  }
};

// node_modules/kysely/dist/esm/parser/fetch-parser.js
function parseFetch(rowCount, modifier) {
  if (!isNumber(rowCount) && !isBigInt(rowCount)) {
    throw new Error(`Invalid fetch row count: ${rowCount}`);
  }
  if (!isFetchModifier(modifier)) {
    throw new Error(`Invalid fetch modifier: ${modifier}`);
  }
  return FetchNode.create(rowCount, modifier);
}
function isFetchModifier(value) {
  return value === "only" || value === "with ties";
}

// node_modules/kysely/dist/esm/query-builder/select-query-builder.js
class SelectQueryBuilderImpl {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  get expressionType() {
    return;
  }
  get isSelectQueryBuilder() {
    return true;
  }
  where(...args) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithWhere(this.#props.queryNode, parseValueBinaryOperationOrExpression(args))
    });
  }
  whereRef(lhs, op, rhs) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithWhere(this.#props.queryNode, parseReferentialBinaryOperation(lhs, op, rhs))
    });
  }
  having(...args) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithHaving(this.#props.queryNode, parseValueBinaryOperationOrExpression(args))
    });
  }
  havingRef(lhs, op, rhs) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithHaving(this.#props.queryNode, parseReferentialBinaryOperation(lhs, op, rhs))
    });
  }
  select(selection) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithSelections(this.#props.queryNode, parseSelectArg(selection))
    });
  }
  distinctOn(selection) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithDistinctOn(this.#props.queryNode, parseReferenceExpressionOrList(selection))
    });
  }
  modifyFront(modifier) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithFrontModifier(this.#props.queryNode, SelectModifierNode.createWithExpression(modifier.toOperationNode()))
    });
  }
  modifyEnd(modifier) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithEndModifier(this.#props.queryNode, SelectModifierNode.createWithExpression(modifier.toOperationNode()))
    });
  }
  distinct() {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithFrontModifier(this.#props.queryNode, SelectModifierNode.create("Distinct"))
    });
  }
  forUpdate(of) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithEndModifier(this.#props.queryNode, SelectModifierNode.create("ForUpdate", of ? asArray(of).map(parseTable) : undefined))
    });
  }
  forShare(of) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithEndModifier(this.#props.queryNode, SelectModifierNode.create("ForShare", of ? asArray(of).map(parseTable) : undefined))
    });
  }
  forKeyShare(of) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithEndModifier(this.#props.queryNode, SelectModifierNode.create("ForKeyShare", of ? asArray(of).map(parseTable) : undefined))
    });
  }
  forNoKeyUpdate(of) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithEndModifier(this.#props.queryNode, SelectModifierNode.create("ForNoKeyUpdate", of ? asArray(of).map(parseTable) : undefined))
    });
  }
  skipLocked() {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithEndModifier(this.#props.queryNode, SelectModifierNode.create("SkipLocked"))
    });
  }
  noWait() {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithEndModifier(this.#props.queryNode, SelectModifierNode.create("NoWait"))
    });
  }
  selectAll(table) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithSelections(this.#props.queryNode, parseSelectAll(table))
    });
  }
  innerJoin(...args) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("InnerJoin", args))
    });
  }
  leftJoin(...args) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("LeftJoin", args))
    });
  }
  rightJoin(...args) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("RightJoin", args))
    });
  }
  fullJoin(...args) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("FullJoin", args))
    });
  }
  innerJoinLateral(...args) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("LateralInnerJoin", args))
    });
  }
  leftJoinLateral(...args) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithJoin(this.#props.queryNode, parseJoin("LateralLeftJoin", args))
    });
  }
  orderBy(...args) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithOrderByItems(this.#props.queryNode, parseOrderBy(args))
    });
  }
  groupBy(groupBy) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithGroupByItems(this.#props.queryNode, parseGroupBy(groupBy))
    });
  }
  limit(limit) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithLimit(this.#props.queryNode, LimitNode.create(parseValueExpression(limit)))
    });
  }
  offset(offset) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithOffset(this.#props.queryNode, OffsetNode.create(parseValueExpression(offset)))
    });
  }
  fetch(rowCount, modifier = "only") {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithFetch(this.#props.queryNode, parseFetch(rowCount, modifier))
    });
  }
  top(expression, modifiers) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithTop(this.#props.queryNode, parseTop(expression, modifiers))
    });
  }
  union(expression) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithSetOperations(this.#props.queryNode, parseSetOperations("union", expression, false))
    });
  }
  unionAll(expression) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithSetOperations(this.#props.queryNode, parseSetOperations("union", expression, true))
    });
  }
  intersect(expression) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithSetOperations(this.#props.queryNode, parseSetOperations("intersect", expression, false))
    });
  }
  intersectAll(expression) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithSetOperations(this.#props.queryNode, parseSetOperations("intersect", expression, true))
    });
  }
  except(expression) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithSetOperations(this.#props.queryNode, parseSetOperations("except", expression, false))
    });
  }
  exceptAll(expression) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithSetOperations(this.#props.queryNode, parseSetOperations("except", expression, true))
    });
  }
  as(alias) {
    return new AliasedSelectQueryBuilderImpl(this, alias);
  }
  clearSelect() {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithoutSelections(this.#props.queryNode)
    });
  }
  clearWhere() {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithoutWhere(this.#props.queryNode)
    });
  }
  clearLimit() {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithoutLimit(this.#props.queryNode)
    });
  }
  clearOffset() {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithoutOffset(this.#props.queryNode)
    });
  }
  clearOrderBy() {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithoutOrderBy(this.#props.queryNode)
    });
  }
  clearGroupBy() {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: SelectQueryNode.cloneWithoutGroupBy(this.#props.queryNode)
    });
  }
  $call(func) {
    return func(this);
  }
  $if(condition, func) {
    if (condition) {
      return func(this);
    }
    return new SelectQueryBuilderImpl({
      ...this.#props
    });
  }
  $castTo() {
    return new SelectQueryBuilderImpl(this.#props);
  }
  $narrowType() {
    return new SelectQueryBuilderImpl(this.#props);
  }
  $assertType() {
    return new SelectQueryBuilderImpl(this.#props);
  }
  $asTuple() {
    return new ExpressionWrapper(this.toOperationNode());
  }
  withPlugin(plugin) {
    return new SelectQueryBuilderImpl({
      ...this.#props,
      executor: this.#props.executor.withPlugin(plugin)
    });
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.queryNode, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    const compiledQuery = this.compile();
    const result = await this.#props.executor.executeQuery(compiledQuery, this.#props.queryId);
    return result.rows;
  }
  async executeTakeFirst() {
    const [result] = await this.execute();
    return result;
  }
  async executeTakeFirstOrThrow(errorConstructor = NoResultError) {
    const result = await this.executeTakeFirst();
    if (result === undefined) {
      const error = isNoResultErrorConstructor(errorConstructor) ? new errorConstructor(this.toOperationNode()) : errorConstructor(this.toOperationNode());
      throw error;
    }
    return result;
  }
  async* stream(chunkSize = 100) {
    const compiledQuery = this.compile();
    const stream = this.#props.executor.stream(compiledQuery, chunkSize, this.#props.queryId);
    for await (const item of stream) {
      yield* item.rows;
    }
  }
  async explain(format, options) {
    const builder = new SelectQueryBuilderImpl({
      ...this.#props,
      queryNode: QueryNode.cloneWithExplain(this.#props.queryNode, format, options)
    });
    return await builder.execute();
  }
}
preventAwait(SelectQueryBuilderImpl, "don't await SelectQueryBuilder instances directly. To execute the query you need to call `execute` or `executeTakeFirst`.");
function createSelectQueryBuilder(props) {
  return new SelectQueryBuilderImpl(props);
}

class AliasedSelectQueryBuilderImpl {
  #queryBuilder;
  #alias;
  constructor(queryBuilder, alias) {
    this.#queryBuilder = queryBuilder;
    this.#alias = alias;
  }
  get expression() {
    return this.#queryBuilder;
  }
  get alias() {
    return this.#alias;
  }
  get isAliasedSelectQueryBuilder() {
    return true;
  }
  toOperationNode() {
    return AliasNode.create(this.#queryBuilder.toOperationNode(), IdentifierNode.create(this.#alias));
  }
}
preventAwait(AliasedSelectQueryBuilderImpl, "don't await AliasedSelectQueryBuilder instances directly. AliasedSelectQueryBuilder should never be executed directly since it's always a part of another query.");

// node_modules/kysely/dist/esm/operation-node/aggregate-function-node.js
var AggregateFunctionNode = freeze({
  is(node) {
    return node.kind === "AggregateFunctionNode";
  },
  create(aggregateFunction, aggregated = []) {
    return freeze({
      kind: "AggregateFunctionNode",
      func: aggregateFunction,
      aggregated
    });
  },
  cloneWithDistinct(aggregateFunctionNode) {
    return freeze({
      ...aggregateFunctionNode,
      distinct: true
    });
  },
  cloneWithFilter(aggregateFunctionNode, filter) {
    return freeze({
      ...aggregateFunctionNode,
      filter: aggregateFunctionNode.filter ? WhereNode.cloneWithOperation(aggregateFunctionNode.filter, "And", filter) : WhereNode.create(filter)
    });
  },
  cloneWithOrFilter(aggregateFunctionNode, filter) {
    return freeze({
      ...aggregateFunctionNode,
      filter: aggregateFunctionNode.filter ? WhereNode.cloneWithOperation(aggregateFunctionNode.filter, "Or", filter) : WhereNode.create(filter)
    });
  },
  cloneWithOver(aggregateFunctionNode, over) {
    return freeze({
      ...aggregateFunctionNode,
      over
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/function-node.js
var FunctionNode = freeze({
  is(node) {
    return node.kind === "FunctionNode";
  },
  create(func, args) {
    return freeze({
      kind: "FunctionNode",
      func,
      arguments: args
    });
  }
});

// node_modules/kysely/dist/esm/query-builder/aggregate-function-builder.js
class AggregateFunctionBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  get expressionType() {
    return;
  }
  as(alias) {
    return new AliasedAggregateFunctionBuilder(this, alias);
  }
  distinct() {
    return new AggregateFunctionBuilder({
      ...this.#props,
      aggregateFunctionNode: AggregateFunctionNode.cloneWithDistinct(this.#props.aggregateFunctionNode)
    });
  }
  filterWhere(...args) {
    return new AggregateFunctionBuilder({
      ...this.#props,
      aggregateFunctionNode: AggregateFunctionNode.cloneWithFilter(this.#props.aggregateFunctionNode, parseValueBinaryOperationOrExpression(args))
    });
  }
  filterWhereRef(lhs, op, rhs) {
    return new AggregateFunctionBuilder({
      ...this.#props,
      aggregateFunctionNode: AggregateFunctionNode.cloneWithFilter(this.#props.aggregateFunctionNode, parseReferentialBinaryOperation(lhs, op, rhs))
    });
  }
  over(over) {
    const builder = createOverBuilder();
    return new AggregateFunctionBuilder({
      ...this.#props,
      aggregateFunctionNode: AggregateFunctionNode.cloneWithOver(this.#props.aggregateFunctionNode, (over ? over(builder) : builder).toOperationNode())
    });
  }
  $call(func) {
    return func(this);
  }
  $castTo() {
    return new AggregateFunctionBuilder(this.#props);
  }
  $notNull() {
    return new AggregateFunctionBuilder(this.#props);
  }
  toOperationNode() {
    return this.#props.aggregateFunctionNode;
  }
}
preventAwait(AggregateFunctionBuilder, "don't await AggregateFunctionBuilder instances. They are never executed directly and are always just a part of a query.");

class AliasedAggregateFunctionBuilder {
  #aggregateFunctionBuilder;
  #alias;
  constructor(aggregateFunctionBuilder, alias) {
    this.#aggregateFunctionBuilder = aggregateFunctionBuilder;
    this.#alias = alias;
  }
  get expression() {
    return this.#aggregateFunctionBuilder;
  }
  get alias() {
    return this.#alias;
  }
  toOperationNode() {
    return AliasNode.create(this.#aggregateFunctionBuilder.toOperationNode(), IdentifierNode.create(this.#alias));
  }
}

// node_modules/kysely/dist/esm/query-builder/function-module.js
function createFunctionModule() {
  const fn = (name, args) => {
    return new ExpressionWrapper(FunctionNode.create(name, parseReferenceExpressionOrList(args ?? [])));
  };
  const agg = (name, args) => {
    return new AggregateFunctionBuilder({
      aggregateFunctionNode: AggregateFunctionNode.create(name, args ? parseReferenceExpressionOrList(args) : undefined)
    });
  };
  return Object.assign(fn, {
    agg,
    avg(column) {
      return agg("avg", [column]);
    },
    coalesce(...values) {
      return fn("coalesce", values);
    },
    count(column) {
      return agg("count", [column]);
    },
    countAll(table) {
      return new AggregateFunctionBuilder({
        aggregateFunctionNode: AggregateFunctionNode.create("count", parseSelectAll(table))
      });
    },
    max(column) {
      return agg("max", [column]);
    },
    min(column) {
      return agg("min", [column]);
    },
    sum(column) {
      return agg("sum", [column]);
    },
    any(column) {
      return fn("any", [column]);
    },
    jsonAgg(table) {
      return new AggregateFunctionBuilder({
        aggregateFunctionNode: AggregateFunctionNode.create("json_agg", [
          isString(table) ? parseTable(table) : table.toOperationNode()
        ])
      });
    },
    toJson(table) {
      return new ExpressionWrapper(FunctionNode.create("to_json", [
        isString(table) ? parseTable(table) : table.toOperationNode()
      ]));
    }
  });
}

// node_modules/kysely/dist/esm/operation-node/unary-operation-node.js
var UnaryOperationNode = freeze({
  is(node) {
    return node.kind === "UnaryOperationNode";
  },
  create(operator, operand) {
    return freeze({
      kind: "UnaryOperationNode",
      operator,
      operand
    });
  }
});

// node_modules/kysely/dist/esm/parser/unary-operation-parser.js
function parseUnaryOperation(operator, operand) {
  return UnaryOperationNode.create(OperatorNode.create(operator), parseReferenceExpression(operand));
}

// node_modules/kysely/dist/esm/operation-node/case-node.js
var CaseNode = freeze({
  is(node) {
    return node.kind === "CaseNode";
  },
  create(value) {
    return freeze({
      kind: "CaseNode",
      value
    });
  },
  cloneWithWhen(caseNode, when) {
    return freeze({
      ...caseNode,
      when: freeze(caseNode.when ? [...caseNode.when, when] : [when])
    });
  },
  cloneWithThen(caseNode, then) {
    return freeze({
      ...caseNode,
      when: caseNode.when ? freeze([
        ...caseNode.when.slice(0, -1),
        WhenNode.cloneWithResult(caseNode.when[caseNode.when.length - 1], then)
      ]) : undefined
    });
  },
  cloneWith(caseNode, props) {
    return freeze({
      ...caseNode,
      ...props
    });
  }
});

// node_modules/kysely/dist/esm/query-builder/case-builder.js
class CaseBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  when(...args) {
    return new CaseThenBuilder({
      ...this.#props,
      node: CaseNode.cloneWithWhen(this.#props.node, WhenNode.create(parseValueBinaryOperationOrExpression(args)))
    });
  }
}

class CaseThenBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  then(valueExpression) {
    return new CaseWhenBuilder({
      ...this.#props,
      node: CaseNode.cloneWithThen(this.#props.node, isSafeImmediateValue(valueExpression) ? parseSafeImmediateValue(valueExpression) : parseValueExpression(valueExpression))
    });
  }
}

class CaseWhenBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  when(...args) {
    return new CaseThenBuilder({
      ...this.#props,
      node: CaseNode.cloneWithWhen(this.#props.node, WhenNode.create(parseValueBinaryOperationOrExpression(args)))
    });
  }
  else(valueExpression) {
    return new CaseEndBuilder({
      ...this.#props,
      node: CaseNode.cloneWith(this.#props.node, {
        else: isSafeImmediateValue(valueExpression) ? parseSafeImmediateValue(valueExpression) : parseValueExpression(valueExpression)
      })
    });
  }
  end() {
    return new ExpressionWrapper(CaseNode.cloneWith(this.#props.node, { isStatement: false }));
  }
  endCase() {
    return new ExpressionWrapper(CaseNode.cloneWith(this.#props.node, { isStatement: true }));
  }
}

class CaseEndBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  end() {
    return new ExpressionWrapper(CaseNode.cloneWith(this.#props.node, { isStatement: false }));
  }
  endCase() {
    return new ExpressionWrapper(CaseNode.cloneWith(this.#props.node, { isStatement: true }));
  }
}

// node_modules/kysely/dist/esm/operation-node/json-path-leg-node.js
var JSONPathLegNode = freeze({
  is(node) {
    return node.kind === "JSONPathLegNode";
  },
  create(type, value) {
    return freeze({
      kind: "JSONPathLegNode",
      type,
      value
    });
  }
});

// node_modules/kysely/dist/esm/query-builder/json-path-builder.js
class JSONPathBuilder {
  #node;
  constructor(node) {
    this.#node = node;
  }
  at(index) {
    return this.#createBuilderWithPathLeg("ArrayLocation", index);
  }
  key(key) {
    return this.#createBuilderWithPathLeg("Member", key);
  }
  #createBuilderWithPathLeg(legType, value) {
    if (JSONReferenceNode.is(this.#node)) {
      return new TraversedJSONPathBuilder(JSONReferenceNode.cloneWithTraversal(this.#node, JSONPathNode.is(this.#node.traversal) ? JSONPathNode.cloneWithLeg(this.#node.traversal, JSONPathLegNode.create(legType, value)) : JSONOperatorChainNode.cloneWithValue(this.#node.traversal, ValueNode.createImmediate(value))));
    }
    return new TraversedJSONPathBuilder(JSONPathNode.cloneWithLeg(this.#node, JSONPathLegNode.create(legType, value)));
  }
}

class TraversedJSONPathBuilder extends JSONPathBuilder {
  #node;
  constructor(node) {
    super(node);
    this.#node = node;
  }
  get expressionType() {
    return;
  }
  as(alias) {
    return new AliasedJSONPathBuilder(this, alias);
  }
  $castTo() {
    return new JSONPathBuilder(this.#node);
  }
  $notNull() {
    return new JSONPathBuilder(this.#node);
  }
  toOperationNode() {
    return this.#node;
  }
}

class AliasedJSONPathBuilder {
  #jsonPath;
  #alias;
  constructor(jsonPath, alias) {
    this.#jsonPath = jsonPath;
    this.#alias = alias;
  }
  get expression() {
    return this.#jsonPath;
  }
  get alias() {
    return this.#alias;
  }
  toOperationNode() {
    return AliasNode.create(this.#jsonPath.toOperationNode(), isOperationNodeSource(this.#alias) ? this.#alias.toOperationNode() : IdentifierNode.create(this.#alias));
  }
}

// node_modules/kysely/dist/esm/operation-node/tuple-node.js
var TupleNode = freeze({
  is(node) {
    return node.kind === "TupleNode";
  },
  create(values) {
    return freeze({
      kind: "TupleNode",
      values: freeze(values)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/data-type-node.js
var SIMPLE_COLUMN_DATA_TYPES = [
  "varchar",
  "char",
  "text",
  "integer",
  "int2",
  "int4",
  "int8",
  "smallint",
  "bigint",
  "boolean",
  "real",
  "double precision",
  "float4",
  "float8",
  "decimal",
  "numeric",
  "binary",
  "bytea",
  "date",
  "datetime",
  "time",
  "timetz",
  "timestamp",
  "timestamptz",
  "serial",
  "bigserial",
  "uuid",
  "json",
  "jsonb",
  "blob",
  "varbinary"
];
var COLUMN_DATA_TYPE_REGEX = [
  /^varchar\(\d+\)$/,
  /^char\(\d+\)$/,
  /^decimal\(\d+, \d+\)$/,
  /^numeric\(\d+, \d+\)$/,
  /^binary\(\d+\)$/,
  /^datetime\(\d+\)$/,
  /^time\(\d+\)$/,
  /^timetz\(\d+\)$/,
  /^timestamp\(\d+\)$/,
  /^timestamptz\(\d+\)$/,
  /^varbinary\(\d+\)$/
];
var DataTypeNode = freeze({
  is(node) {
    return node.kind === "DataTypeNode";
  },
  create(dataType) {
    return freeze({
      kind: "DataTypeNode",
      dataType
    });
  }
});
function isColumnDataType(dataType) {
  if (SIMPLE_COLUMN_DATA_TYPES.includes(dataType)) {
    return true;
  }
  if (COLUMN_DATA_TYPE_REGEX.some((r) => r.test(dataType))) {
    return true;
  }
  return false;
}

// node_modules/kysely/dist/esm/parser/data-type-parser.js
function parseDataTypeExpression(dataType) {
  if (isOperationNodeSource(dataType)) {
    return dataType.toOperationNode();
  }
  if (isColumnDataType(dataType)) {
    return DataTypeNode.create(dataType);
  }
  throw new Error(`invalid column data type ${JSON.stringify(dataType)}`);
}

// node_modules/kysely/dist/esm/operation-node/cast-node.js
var CastNode = freeze({
  is(node) {
    return node.kind === "CastNode";
  },
  create(expression, dataType) {
    return freeze({
      kind: "CastNode",
      expression,
      dataType
    });
  }
});

// node_modules/kysely/dist/esm/expression/expression-builder.js
function createExpressionBuilder(executor = NOOP_QUERY_EXECUTOR) {
  function binary(lhs, op, rhs) {
    return new ExpressionWrapper(parseValueBinaryOperation(lhs, op, rhs));
  }
  function unary(op, expr) {
    return new ExpressionWrapper(parseUnaryOperation(op, expr));
  }
  const eb = Object.assign(binary, {
    fn: undefined,
    eb: undefined,
    selectFrom(table) {
      return createSelectQueryBuilder({
        queryId: createQueryId(),
        executor,
        queryNode: SelectQueryNode.createFrom(parseTableExpressionOrList(table))
      });
    },
    case(reference) {
      return new CaseBuilder({
        node: CaseNode.create(isUndefined(reference) ? undefined : parseReferenceExpression(reference))
      });
    },
    ref(reference, op) {
      if (isUndefined(op)) {
        return new ExpressionWrapper(parseStringReference(reference));
      }
      return new JSONPathBuilder(parseJSONReference(reference, op));
    },
    jsonPath() {
      return new JSONPathBuilder(JSONPathNode.create());
    },
    table(table) {
      return new ExpressionWrapper(parseTable(table));
    },
    val(value) {
      return new ExpressionWrapper(parseValueExpression(value));
    },
    refTuple(...values) {
      return new ExpressionWrapper(TupleNode.create(values.map(parseReferenceExpression)));
    },
    tuple(...values) {
      return new ExpressionWrapper(TupleNode.create(values.map(parseValueExpression)));
    },
    lit(value) {
      return new ExpressionWrapper(parseSafeImmediateValue(value));
    },
    unary,
    not(expr) {
      return unary("not", expr);
    },
    exists(expr) {
      return unary("exists", expr);
    },
    neg(expr) {
      return unary("-", expr);
    },
    between(expr, start, end) {
      return new ExpressionWrapper(BinaryOperationNode.create(parseReferenceExpression(expr), OperatorNode.create("between"), AndNode.create(parseValueExpression(start), parseValueExpression(end))));
    },
    betweenSymmetric(expr, start, end) {
      return new ExpressionWrapper(BinaryOperationNode.create(parseReferenceExpression(expr), OperatorNode.create("between symmetric"), AndNode.create(parseValueExpression(start), parseValueExpression(end))));
    },
    and(exprs) {
      if (isReadonlyArray(exprs)) {
        return new ExpressionWrapper(parseFilterList(exprs, "and"));
      }
      return new ExpressionWrapper(parseFilterObject(exprs, "and"));
    },
    or(exprs) {
      if (isReadonlyArray(exprs)) {
        return new ExpressionWrapper(parseFilterList(exprs, "or"));
      }
      return new ExpressionWrapper(parseFilterObject(exprs, "or"));
    },
    parens(...args) {
      const node = parseValueBinaryOperationOrExpression(args);
      if (ParensNode.is(node)) {
        return new ExpressionWrapper(node);
      } else {
        return new ExpressionWrapper(ParensNode.create(node));
      }
    },
    cast(expr, dataType) {
      return new ExpressionWrapper(CastNode.create(parseReferenceExpression(expr), parseDataTypeExpression(dataType)));
    },
    withSchema(schema) {
      return createExpressionBuilder(executor.withPluginAtFront(new WithSchemaPlugin(schema)));
    }
  });
  eb.fn = createFunctionModule();
  eb.eb = eb;
  return eb;
}
function expressionBuilder(_) {
  return createExpressionBuilder();
}

// node_modules/kysely/dist/esm/parser/expression-parser.js
function parseExpression(exp) {
  if (isOperationNodeSource(exp)) {
    return exp.toOperationNode();
  } else if (isFunction(exp)) {
    return exp(expressionBuilder()).toOperationNode();
  }
  throw new Error(`invalid expression: ${JSON.stringify(exp)}`);
}
function parseAliasedExpression(exp) {
  if (isOperationNodeSource(exp)) {
    return exp.toOperationNode();
  } else if (isFunction(exp)) {
    return exp(expressionBuilder()).toOperationNode();
  }
  throw new Error(`invalid aliased expression: ${JSON.stringify(exp)}`);
}
function isExpressionOrFactory(obj) {
  return isExpression(obj) || isAliasedExpression(obj) || isFunction(obj);
}

// node_modules/kysely/dist/esm/parser/table-parser.js
function parseTableExpressionOrList(table) {
  if (isReadonlyArray(table)) {
    return table.map((it) => parseTableExpression(it));
  } else {
    return [parseTableExpression(table)];
  }
}
function parseTableExpression(table) {
  if (isString(table)) {
    return parseAliasedTable(table);
  } else {
    return parseAliasedExpression(table);
  }
}
function parseAliasedTable(from) {
  const ALIAS_SEPARATOR = " as ";
  if (from.includes(ALIAS_SEPARATOR)) {
    const [table, alias] = from.split(ALIAS_SEPARATOR).map(trim2);
    return AliasNode.create(parseTable(table), IdentifierNode.create(alias));
  } else {
    return parseTable(from);
  }
}
function parseTable(from) {
  const SCHEMA_SEPARATOR = ".";
  if (from.includes(SCHEMA_SEPARATOR)) {
    const [schema, table] = from.split(SCHEMA_SEPARATOR).map(trim2);
    return TableNode.createWithSchema(schema, table);
  } else {
    return TableNode.create(from);
  }
}
function trim2(str) {
  return str.trim();
}

// node_modules/kysely/dist/esm/operation-node/add-column-node.js
var AddColumnNode = freeze({
  is(node) {
    return node.kind === "AddColumnNode";
  },
  create(column) {
    return freeze({
      kind: "AddColumnNode",
      column
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/column-definition-node.js
var ColumnDefinitionNode = freeze({
  is(node) {
    return node.kind === "ColumnDefinitionNode";
  },
  create(column, dataType) {
    return freeze({
      kind: "ColumnDefinitionNode",
      column: ColumnNode.create(column),
      dataType
    });
  },
  cloneWithFrontModifier(node, modifier) {
    return freeze({
      ...node,
      frontModifiers: node.frontModifiers ? freeze([...node.frontModifiers, modifier]) : [modifier]
    });
  },
  cloneWithEndModifier(node, modifier) {
    return freeze({
      ...node,
      endModifiers: node.endModifiers ? freeze([...node.endModifiers, modifier]) : [modifier]
    });
  },
  cloneWith(node, props) {
    return freeze({
      ...node,
      ...props
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/drop-column-node.js
var DropColumnNode = freeze({
  is(node) {
    return node.kind === "DropColumnNode";
  },
  create(column) {
    return freeze({
      kind: "DropColumnNode",
      column: ColumnNode.create(column)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/rename-column-node.js
var RenameColumnNode = freeze({
  is(node) {
    return node.kind === "RenameColumnNode";
  },
  create(column, newColumn) {
    return freeze({
      kind: "RenameColumnNode",
      column: ColumnNode.create(column),
      renameTo: ColumnNode.create(newColumn)
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/check-constraint-node.js
var CheckConstraintNode = freeze({
  is(node) {
    return node.kind === "CheckConstraintNode";
  },
  create(expression, constraintName) {
    return freeze({
      kind: "CheckConstraintNode",
      expression,
      name: constraintName ? IdentifierNode.create(constraintName) : undefined
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/references-node.js
var ON_MODIFY_FOREIGN_ACTIONS = [
  "no action",
  "restrict",
  "cascade",
  "set null",
  "set default"
];
var ReferencesNode = freeze({
  is(node) {
    return node.kind === "ReferencesNode";
  },
  create(table, columns) {
    return freeze({
      kind: "ReferencesNode",
      table,
      columns: freeze([...columns])
    });
  },
  cloneWithOnDelete(references, onDelete) {
    return freeze({
      ...references,
      onDelete
    });
  },
  cloneWithOnUpdate(references, onUpdate) {
    return freeze({
      ...references,
      onUpdate
    });
  }
});

// node_modules/kysely/dist/esm/parser/default-value-parser.js
function parseDefaultValueExpression(value) {
  return isOperationNodeSource(value) ? value.toOperationNode() : ValueNode.createImmediate(value);
}

// node_modules/kysely/dist/esm/operation-node/generated-node.js
var GeneratedNode = freeze({
  is(node) {
    return node.kind === "GeneratedNode";
  },
  create(params) {
    return freeze({
      kind: "GeneratedNode",
      ...params
    });
  },
  createWithExpression(expression) {
    return freeze({
      kind: "GeneratedNode",
      always: true,
      expression
    });
  },
  cloneWith(node, params) {
    return freeze({
      ...node,
      ...params
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/default-value-node.js
var DefaultValueNode = freeze({
  is(node) {
    return node.kind === "DefaultValueNode";
  },
  create(defaultValue) {
    return freeze({
      kind: "DefaultValueNode",
      defaultValue
    });
  }
});

// node_modules/kysely/dist/esm/parser/on-modify-action-parser.js
function parseOnModifyForeignAction(action) {
  if (ON_MODIFY_FOREIGN_ACTIONS.includes(action)) {
    return action;
  }
  throw new Error(`invalid OnModifyForeignAction ${action}`);
}

// node_modules/kysely/dist/esm/schema/column-definition-builder.js
class ColumnDefinitionBuilder {
  #node;
  constructor(node) {
    this.#node = node;
  }
  autoIncrement() {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, { autoIncrement: true }));
  }
  identity() {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, { identity: true }));
  }
  primaryKey() {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, { primaryKey: true }));
  }
  references(ref) {
    const references = parseStringReference(ref);
    if (!references.table || SelectAllNode.is(references.column)) {
      throw new Error(`invalid call references('${ref}'). The reference must have format table.column or schema.table.column`);
    }
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, {
      references: ReferencesNode.create(references.table, [
        references.column
      ])
    }));
  }
  onDelete(onDelete) {
    if (!this.#node.references) {
      throw new Error("on delete constraint can only be added for foreign keys");
    }
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, {
      references: ReferencesNode.cloneWithOnDelete(this.#node.references, parseOnModifyForeignAction(onDelete))
    }));
  }
  onUpdate(onUpdate) {
    if (!this.#node.references) {
      throw new Error("on update constraint can only be added for foreign keys");
    }
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, {
      references: ReferencesNode.cloneWithOnUpdate(this.#node.references, parseOnModifyForeignAction(onUpdate))
    }));
  }
  unique() {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, { unique: true }));
  }
  notNull() {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, { notNull: true }));
  }
  unsigned() {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, { unsigned: true }));
  }
  defaultTo(value) {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, {
      defaultTo: DefaultValueNode.create(parseDefaultValueExpression(value))
    }));
  }
  check(expression) {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, {
      check: CheckConstraintNode.create(expression.toOperationNode())
    }));
  }
  generatedAlwaysAs(expression) {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, {
      generated: GeneratedNode.createWithExpression(expression.toOperationNode())
    }));
  }
  generatedAlwaysAsIdentity() {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, {
      generated: GeneratedNode.create({ identity: true, always: true })
    }));
  }
  generatedByDefaultAsIdentity() {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, {
      generated: GeneratedNode.create({ identity: true, byDefault: true })
    }));
  }
  stored() {
    if (!this.#node.generated) {
      throw new Error("stored() can only be called after generatedAlwaysAs");
    }
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, {
      generated: GeneratedNode.cloneWith(this.#node.generated, {
        stored: true
      })
    }));
  }
  modifyFront(modifier) {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWithFrontModifier(this.#node, modifier.toOperationNode()));
  }
  nullsNotDistinct() {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, { nullsNotDistinct: true }));
  }
  ifNotExists() {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWith(this.#node, { ifNotExists: true }));
  }
  modifyEnd(modifier) {
    return new ColumnDefinitionBuilder(ColumnDefinitionNode.cloneWithEndModifier(this.#node, modifier.toOperationNode()));
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#node;
  }
}
preventAwait(ColumnDefinitionBuilder, "don't await ColumnDefinitionBuilder instances directly.");

// node_modules/kysely/dist/esm/operation-node/modify-column-node.js
var ModifyColumnNode = freeze({
  is(node) {
    return node.kind === "ModifyColumnNode";
  },
  create(column) {
    return freeze({
      kind: "ModifyColumnNode",
      column
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/foreign-key-constraint-node.js
var ForeignKeyConstraintNode = freeze({
  is(node) {
    return node.kind === "ForeignKeyConstraintNode";
  },
  create(sourceColumns, targetTable, targetColumns, constraintName) {
    return freeze({
      kind: "ForeignKeyConstraintNode",
      columns: sourceColumns,
      references: ReferencesNode.create(targetTable, targetColumns),
      name: constraintName ? IdentifierNode.create(constraintName) : undefined
    });
  },
  cloneWith(node, props) {
    return freeze({
      ...node,
      ...props
    });
  }
});

// node_modules/kysely/dist/esm/schema/foreign-key-constraint-builder.js
class ForeignKeyConstraintBuilder {
  #node;
  constructor(node) {
    this.#node = node;
  }
  onDelete(onDelete) {
    return new ForeignKeyConstraintBuilder(ForeignKeyConstraintNode.cloneWith(this.#node, {
      onDelete: parseOnModifyForeignAction(onDelete)
    }));
  }
  onUpdate(onUpdate) {
    return new ForeignKeyConstraintBuilder(ForeignKeyConstraintNode.cloneWith(this.#node, {
      onUpdate: parseOnModifyForeignAction(onUpdate)
    }));
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#node;
  }
}
preventAwait(ForeignKeyConstraintBuilder, "don't await ForeignKeyConstraintBuilder instances directly.");

// node_modules/kysely/dist/esm/operation-node/add-constraint-node.js
var AddConstraintNode = freeze({
  is(node) {
    return node.kind === "AddConstraintNode";
  },
  create(constraint) {
    return freeze({
      kind: "AddConstraintNode",
      constraint
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/unique-constraint-node.js
var UniqueConstraintNode = freeze({
  is(node) {
    return node.kind === "UniqueConstraintNode";
  },
  create(columns, constraintName, nullsNotDistinct) {
    return freeze({
      kind: "UniqueConstraintNode",
      columns: freeze(columns.map(ColumnNode.create)),
      name: constraintName ? IdentifierNode.create(constraintName) : undefined,
      nullsNotDistinct
    });
  },
  cloneWith(node, props) {
    return freeze({
      ...node,
      ...props
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/drop-constraint-node.js
var DropConstraintNode = freeze({
  is(node) {
    return node.kind === "DropConstraintNode";
  },
  create(constraintName) {
    return freeze({
      kind: "DropConstraintNode",
      constraintName: IdentifierNode.create(constraintName)
    });
  },
  cloneWith(dropConstraint, props) {
    return freeze({
      ...dropConstraint,
      ...props
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/alter-column-node.js
var AlterColumnNode = freeze({
  is(node) {
    return node.kind === "AlterColumnNode";
  },
  create(column, prop, value) {
    return freeze({
      kind: "AlterColumnNode",
      column: ColumnNode.create(column),
      [prop]: value
    });
  }
});

// node_modules/kysely/dist/esm/schema/alter-column-builder.js
class AlterColumnBuilder {
  #column;
  constructor(column) {
    this.#column = column;
  }
  setDataType(dataType) {
    return new AlteredColumnBuilder(AlterColumnNode.create(this.#column, "dataType", parseDataTypeExpression(dataType)));
  }
  setDefault(value) {
    return new AlteredColumnBuilder(AlterColumnNode.create(this.#column, "setDefault", parseDefaultValueExpression(value)));
  }
  dropDefault() {
    return new AlteredColumnBuilder(AlterColumnNode.create(this.#column, "dropDefault", true));
  }
  setNotNull() {
    return new AlteredColumnBuilder(AlterColumnNode.create(this.#column, "setNotNull", true));
  }
  dropNotNull() {
    return new AlteredColumnBuilder(AlterColumnNode.create(this.#column, "dropNotNull", true));
  }
  $call(func) {
    return func(this);
  }
}
preventAwait(AlterColumnBuilder, "don't await AlterColumnBuilder instances");

class AlteredColumnBuilder {
  #alterColumnNode;
  constructor(alterColumnNode) {
    this.#alterColumnNode = alterColumnNode;
  }
  toOperationNode() {
    return this.#alterColumnNode;
  }
}
preventAwait(AlteredColumnBuilder, "don't await AlteredColumnBuilder instances");

// node_modules/kysely/dist/esm/schema/alter-table-executor.js
class AlterTableExecutor {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(AlterTableExecutor, "don't await AlterTableExecutor instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/schema/alter-table-add-foreign-key-constraint-builder.js
class AlterTableAddForeignKeyConstraintBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  onDelete(onDelete) {
    return new AlterTableAddForeignKeyConstraintBuilder({
      ...this.#props,
      constraintBuilder: this.#props.constraintBuilder.onDelete(onDelete)
    });
  }
  onUpdate(onUpdate) {
    return new AlterTableAddForeignKeyConstraintBuilder({
      ...this.#props,
      constraintBuilder: this.#props.constraintBuilder.onUpdate(onUpdate)
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(AlterTableNode.cloneWithTableProps(this.#props.node, {
      addConstraint: AddConstraintNode.create(this.#props.constraintBuilder.toOperationNode())
    }), this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(AlterTableAddForeignKeyConstraintBuilder, "don't await AlterTableAddForeignKeyConstraintBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/schema/alter-table-drop-constraint-builder.js
class AlterTableDropConstraintBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  ifExists() {
    return new AlterTableDropConstraintBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        dropConstraint: DropConstraintNode.cloneWith(this.#props.node.dropConstraint, {
          ifExists: true
        })
      })
    });
  }
  cascade() {
    return new AlterTableDropConstraintBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        dropConstraint: DropConstraintNode.cloneWith(this.#props.node.dropConstraint, {
          modifier: "cascade"
        })
      })
    });
  }
  restrict() {
    return new AlterTableDropConstraintBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        dropConstraint: DropConstraintNode.cloneWith(this.#props.node.dropConstraint, {
          modifier: "restrict"
        })
      })
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(AlterTableDropConstraintBuilder, "don't await AlterTableDropConstraintBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/operation-node/primary-constraint-node.js
var PrimaryConstraintNode = freeze({
  is(node) {
    return node.kind === "PrimaryKeyConstraintNode";
  },
  create(columns, constraintName) {
    return freeze({
      kind: "PrimaryKeyConstraintNode",
      columns: freeze(columns.map(ColumnNode.create)),
      name: constraintName ? IdentifierNode.create(constraintName) : undefined
    });
  }
});

// node_modules/kysely/dist/esm/operation-node/add-index-node.js
var AddIndexNode = freeze({
  is(node) {
    return node.kind === "AddIndexNode";
  },
  create(name) {
    return freeze({
      kind: "AddIndexNode",
      name: IdentifierNode.create(name)
    });
  },
  cloneWith(node, props) {
    return freeze({
      ...node,
      ...props
    });
  },
  cloneWithColumns(node, columns) {
    return freeze({
      ...node,
      columns: [...node.columns || [], ...columns]
    });
  }
});

// node_modules/kysely/dist/esm/schema/alter-table-add-index-builder.js
class AlterTableAddIndexBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  unique() {
    return new AlterTableAddIndexBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        addIndex: AddIndexNode.cloneWith(this.#props.node.addIndex, {
          unique: true
        })
      })
    });
  }
  column(column) {
    return new AlterTableAddIndexBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        addIndex: AddIndexNode.cloneWithColumns(this.#props.node.addIndex, [
          parseOrderedColumnName(column)
        ])
      })
    });
  }
  columns(columns) {
    return new AlterTableAddIndexBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        addIndex: AddIndexNode.cloneWithColumns(this.#props.node.addIndex, columns.map(parseOrderedColumnName))
      })
    });
  }
  expression(expression) {
    return new AlterTableAddIndexBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        addIndex: AddIndexNode.cloneWithColumns(this.#props.node.addIndex, [
          expression.toOperationNode()
        ])
      })
    });
  }
  using(indexType) {
    return new AlterTableAddIndexBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        addIndex: AddIndexNode.cloneWith(this.#props.node.addIndex, {
          using: RawNode.createWithSql(indexType)
        })
      })
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(AlterTableAddIndexBuilder, "don't await AlterTableAddIndexBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/schema/unique-constraint-builder.js
class UniqueConstraintNodeBuilder {
  #node;
  constructor(node) {
    this.#node = node;
  }
  toOperationNode() {
    return this.#node;
  }
  nullsNotDistinct() {
    return new UniqueConstraintNodeBuilder(UniqueConstraintNode.cloneWith(this.#node, { nullsNotDistinct: true }));
  }
}
preventAwait(UniqueConstraintNodeBuilder, "don't await UniqueConstraintNodeBuilder instances directly.");

// node_modules/kysely/dist/esm/schema/alter-table-builder.js
class AlterTableBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  renameTo(newTableName) {
    return new AlterTableExecutor({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        renameTo: parseTable(newTableName)
      })
    });
  }
  setSchema(newSchema) {
    return new AlterTableExecutor({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        setSchema: IdentifierNode.create(newSchema)
      })
    });
  }
  alterColumn(column, alteration) {
    const builder = alteration(new AlterColumnBuilder(column));
    return new AlterTableColumnAlteringBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithColumnAlteration(this.#props.node, builder.toOperationNode())
    });
  }
  dropColumn(column) {
    return new AlterTableColumnAlteringBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithColumnAlteration(this.#props.node, DropColumnNode.create(column))
    });
  }
  renameColumn(column, newColumn) {
    return new AlterTableColumnAlteringBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithColumnAlteration(this.#props.node, RenameColumnNode.create(column, newColumn))
    });
  }
  addColumn(columnName, dataType, build = noop) {
    const builder = build(new ColumnDefinitionBuilder(ColumnDefinitionNode.create(columnName, parseDataTypeExpression(dataType))));
    return new AlterTableColumnAlteringBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithColumnAlteration(this.#props.node, AddColumnNode.create(builder.toOperationNode()))
    });
  }
  modifyColumn(columnName, dataType, build = noop) {
    const builder = build(new ColumnDefinitionBuilder(ColumnDefinitionNode.create(columnName, parseDataTypeExpression(dataType))));
    return new AlterTableColumnAlteringBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithColumnAlteration(this.#props.node, ModifyColumnNode.create(builder.toOperationNode()))
    });
  }
  addUniqueConstraint(constraintName, columns, build = noop) {
    const uniqueConstraintBuilder = build(new UniqueConstraintNodeBuilder(UniqueConstraintNode.create(columns, constraintName)));
    return new AlterTableExecutor({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        addConstraint: AddConstraintNode.create(uniqueConstraintBuilder.toOperationNode())
      })
    });
  }
  addCheckConstraint(constraintName, checkExpression) {
    return new AlterTableExecutor({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        addConstraint: AddConstraintNode.create(CheckConstraintNode.create(checkExpression.toOperationNode(), constraintName))
      })
    });
  }
  addForeignKeyConstraint(constraintName, columns, targetTable, targetColumns) {
    return new AlterTableAddForeignKeyConstraintBuilder({
      ...this.#props,
      constraintBuilder: new ForeignKeyConstraintBuilder(ForeignKeyConstraintNode.create(columns.map(ColumnNode.create), parseTable(targetTable), targetColumns.map(ColumnNode.create), constraintName))
    });
  }
  addPrimaryKeyConstraint(constraintName, columns) {
    return new AlterTableExecutor({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        addConstraint: AddConstraintNode.create(PrimaryConstraintNode.create(columns, constraintName))
      })
    });
  }
  dropConstraint(constraintName) {
    return new AlterTableDropConstraintBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        dropConstraint: DropConstraintNode.create(constraintName)
      })
    });
  }
  addIndex(indexName) {
    return new AlterTableAddIndexBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        addIndex: AddIndexNode.create(indexName)
      })
    });
  }
  dropIndex(indexName) {
    return new AlterTableExecutor({
      ...this.#props,
      node: AlterTableNode.cloneWithTableProps(this.#props.node, {
        dropIndex: DropIndexNode.create(indexName)
      })
    });
  }
  $call(func) {
    return func(this);
  }
}
preventAwait(AlterTableBuilder, "don't await AlterTableBuilder instances");

class AlterTableColumnAlteringBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  alterColumn(column, alteration) {
    const builder = alteration(new AlterColumnBuilder(column));
    return new AlterTableColumnAlteringBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithColumnAlteration(this.#props.node, builder.toOperationNode())
    });
  }
  dropColumn(column) {
    return new AlterTableColumnAlteringBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithColumnAlteration(this.#props.node, DropColumnNode.create(column))
    });
  }
  renameColumn(column, newColumn) {
    return new AlterTableColumnAlteringBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithColumnAlteration(this.#props.node, RenameColumnNode.create(column, newColumn))
    });
  }
  addColumn(columnName, dataType, build = noop) {
    const builder = build(new ColumnDefinitionBuilder(ColumnDefinitionNode.create(columnName, parseDataTypeExpression(dataType))));
    return new AlterTableColumnAlteringBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithColumnAlteration(this.#props.node, AddColumnNode.create(builder.toOperationNode()))
    });
  }
  modifyColumn(columnName, dataType, build = noop) {
    const builder = build(new ColumnDefinitionBuilder(ColumnDefinitionNode.create(columnName, parseDataTypeExpression(dataType))));
    return new AlterTableColumnAlteringBuilder({
      ...this.#props,
      node: AlterTableNode.cloneWithColumnAlteration(this.#props.node, ModifyColumnNode.create(builder.toOperationNode()))
    });
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(AlterTableColumnAlteringBuilder, "don't await AlterTableColumnAlteringBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/plugin/immediate-value/immediate-value-transformer.js
class ImmediateValueTransformer extends OperationNodeTransformer {
  transformValue(node) {
    return {
      ...super.transformValue(node),
      immediate: true
    };
  }
}

// node_modules/kysely/dist/esm/schema/create-index-builder.js
class CreateIndexBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  ifNotExists() {
    return new CreateIndexBuilder({
      ...this.#props,
      node: CreateIndexNode.cloneWith(this.#props.node, {
        ifNotExists: true
      })
    });
  }
  unique() {
    return new CreateIndexBuilder({
      ...this.#props,
      node: CreateIndexNode.cloneWith(this.#props.node, {
        unique: true
      })
    });
  }
  nullsNotDistinct() {
    return new CreateIndexBuilder({
      ...this.#props,
      node: CreateIndexNode.cloneWith(this.#props.node, {
        nullsNotDistinct: true
      })
    });
  }
  on(table) {
    return new CreateIndexBuilder({
      ...this.#props,
      node: CreateIndexNode.cloneWith(this.#props.node, {
        table: parseTable(table)
      })
    });
  }
  column(column) {
    return new CreateIndexBuilder({
      ...this.#props,
      node: CreateIndexNode.cloneWithColumns(this.#props.node, [
        parseOrderedColumnName(column)
      ])
    });
  }
  columns(columns) {
    return new CreateIndexBuilder({
      ...this.#props,
      node: CreateIndexNode.cloneWithColumns(this.#props.node, columns.map(parseOrderedColumnName))
    });
  }
  expression(expression) {
    return new CreateIndexBuilder({
      ...this.#props,
      node: CreateIndexNode.cloneWithColumns(this.#props.node, [
        expression.toOperationNode()
      ])
    });
  }
  using(indexType) {
    return new CreateIndexBuilder({
      ...this.#props,
      node: CreateIndexNode.cloneWith(this.#props.node, {
        using: RawNode.createWithSql(indexType)
      })
    });
  }
  where(...args) {
    const transformer = new ImmediateValueTransformer;
    return new CreateIndexBuilder({
      ...this.#props,
      node: QueryNode.cloneWithWhere(this.#props.node, transformer.transformNode(parseValueBinaryOperationOrExpression(args)))
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(CreateIndexBuilder, "don't await CreateIndexBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/schema/create-schema-builder.js
class CreateSchemaBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  ifNotExists() {
    return new CreateSchemaBuilder({
      ...this.#props,
      node: CreateSchemaNode.cloneWith(this.#props.node, { ifNotExists: true })
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(CreateSchemaBuilder, "don't await CreateSchemaBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/parser/on-commit-action-parse.js
function parseOnCommitAction(action) {
  if (ON_COMMIT_ACTIONS.includes(action)) {
    return action;
  }
  throw new Error(`invalid OnCommitAction ${action}`);
}

// node_modules/kysely/dist/esm/schema/create-table-builder.js
class CreateTableBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  temporary() {
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWith(this.#props.node, {
        temporary: true
      })
    });
  }
  onCommit(onCommit) {
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWith(this.#props.node, {
        onCommit: parseOnCommitAction(onCommit)
      })
    });
  }
  ifNotExists() {
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWith(this.#props.node, {
        ifNotExists: true
      })
    });
  }
  addColumn(columnName, dataType, build = noop) {
    const columnBuilder = build(new ColumnDefinitionBuilder(ColumnDefinitionNode.create(columnName, parseDataTypeExpression(dataType))));
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWithColumn(this.#props.node, columnBuilder.toOperationNode())
    });
  }
  addPrimaryKeyConstraint(constraintName, columns) {
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWithConstraint(this.#props.node, PrimaryConstraintNode.create(columns, constraintName))
    });
  }
  addUniqueConstraint(constraintName, columns, build = noop) {
    const uniqueConstraintBuilder = build(new UniqueConstraintNodeBuilder(UniqueConstraintNode.create(columns, constraintName)));
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWithConstraint(this.#props.node, uniqueConstraintBuilder.toOperationNode())
    });
  }
  addCheckConstraint(constraintName, checkExpression) {
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWithConstraint(this.#props.node, CheckConstraintNode.create(checkExpression.toOperationNode(), constraintName))
    });
  }
  addForeignKeyConstraint(constraintName, columns, targetTable, targetColumns, build = noop) {
    const builder = build(new ForeignKeyConstraintBuilder(ForeignKeyConstraintNode.create(columns.map(ColumnNode.create), parseTable(targetTable), targetColumns.map(ColumnNode.create), constraintName)));
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWithConstraint(this.#props.node, builder.toOperationNode())
    });
  }
  modifyFront(modifier) {
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWithFrontModifier(this.#props.node, modifier.toOperationNode())
    });
  }
  modifyEnd(modifier) {
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWithEndModifier(this.#props.node, modifier.toOperationNode())
    });
  }
  as(expression) {
    return new CreateTableBuilder({
      ...this.#props,
      node: CreateTableNode.cloneWith(this.#props.node, {
        selectQuery: parseExpression(expression)
      })
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(CreateTableBuilder, "don't await CreateTableBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/schema/drop-index-builder.js
class DropIndexBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  on(table) {
    return new DropIndexBuilder({
      ...this.#props,
      node: DropIndexNode.cloneWith(this.#props.node, {
        table: parseTable(table)
      })
    });
  }
  ifExists() {
    return new DropIndexBuilder({
      ...this.#props,
      node: DropIndexNode.cloneWith(this.#props.node, {
        ifExists: true
      })
    });
  }
  cascade() {
    return new DropIndexBuilder({
      ...this.#props,
      node: DropIndexNode.cloneWith(this.#props.node, {
        cascade: true
      })
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(DropIndexBuilder, "don't await DropIndexBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/schema/drop-schema-builder.js
class DropSchemaBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  ifExists() {
    return new DropSchemaBuilder({
      ...this.#props,
      node: DropSchemaNode.cloneWith(this.#props.node, {
        ifExists: true
      })
    });
  }
  cascade() {
    return new DropSchemaBuilder({
      ...this.#props,
      node: DropSchemaNode.cloneWith(this.#props.node, {
        cascade: true
      })
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(DropSchemaBuilder, "don't await DropSchemaBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/schema/drop-table-builder.js
class DropTableBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  ifExists() {
    return new DropTableBuilder({
      ...this.#props,
      node: DropTableNode.cloneWith(this.#props.node, {
        ifExists: true
      })
    });
  }
  cascade() {
    return new DropTableBuilder({
      ...this.#props,
      node: DropTableNode.cloneWith(this.#props.node, {
        cascade: true
      })
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(DropTableBuilder, "don't await DropTableBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/operation-node/create-view-node.js
var CreateViewNode = freeze({
  is(node) {
    return node.kind === "CreateViewNode";
  },
  create(name) {
    return freeze({
      kind: "CreateViewNode",
      name: SchemableIdentifierNode.create(name)
    });
  },
  cloneWith(createView, params) {
    return freeze({
      ...createView,
      ...params
    });
  }
});

// node_modules/kysely/dist/esm/plugin/immediate-value/immediate-value-plugin.js
class ImmediateValuePlugin {
  #transformer = new ImmediateValueTransformer;
  transformQuery(args) {
    return this.#transformer.transformNode(args.node);
  }
  transformResult(args) {
    return Promise.resolve(args.result);
  }
}

// node_modules/kysely/dist/esm/schema/create-view-builder.js
class CreateViewBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  temporary() {
    return new CreateViewBuilder({
      ...this.#props,
      node: CreateViewNode.cloneWith(this.#props.node, {
        temporary: true
      })
    });
  }
  materialized() {
    return new CreateViewBuilder({
      ...this.#props,
      node: CreateViewNode.cloneWith(this.#props.node, {
        materialized: true
      })
    });
  }
  ifNotExists() {
    return new CreateViewBuilder({
      ...this.#props,
      node: CreateViewNode.cloneWith(this.#props.node, {
        ifNotExists: true
      })
    });
  }
  orReplace() {
    return new CreateViewBuilder({
      ...this.#props,
      node: CreateViewNode.cloneWith(this.#props.node, {
        orReplace: true
      })
    });
  }
  columns(columns) {
    return new CreateViewBuilder({
      ...this.#props,
      node: CreateViewNode.cloneWith(this.#props.node, {
        columns: columns.map(parseColumnName)
      })
    });
  }
  as(query) {
    const queryNode = query.withPlugin(new ImmediateValuePlugin).toOperationNode();
    return new CreateViewBuilder({
      ...this.#props,
      node: CreateViewNode.cloneWith(this.#props.node, {
        as: queryNode
      })
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(CreateViewBuilder, "don't await CreateViewBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/operation-node/drop-view-node.js
var DropViewNode = freeze({
  is(node) {
    return node.kind === "DropViewNode";
  },
  create(name) {
    return freeze({
      kind: "DropViewNode",
      name: SchemableIdentifierNode.create(name)
    });
  },
  cloneWith(dropView, params) {
    return freeze({
      ...dropView,
      ...params
    });
  }
});

// node_modules/kysely/dist/esm/schema/drop-view-builder.js
class DropViewBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  materialized() {
    return new DropViewBuilder({
      ...this.#props,
      node: DropViewNode.cloneWith(this.#props.node, {
        materialized: true
      })
    });
  }
  ifExists() {
    return new DropViewBuilder({
      ...this.#props,
      node: DropViewNode.cloneWith(this.#props.node, {
        ifExists: true
      })
    });
  }
  cascade() {
    return new DropViewBuilder({
      ...this.#props,
      node: DropViewNode.cloneWith(this.#props.node, {
        cascade: true
      })
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(DropViewBuilder, "don't await DropViewBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/operation-node/create-type-node.js
var CreateTypeNode = freeze({
  is(node) {
    return node.kind === "CreateTypeNode";
  },
  create(name) {
    return freeze({
      kind: "CreateTypeNode",
      name
    });
  },
  cloneWithEnum(createType, values) {
    return freeze({
      ...createType,
      enum: ValueListNode.create(values.map((value) => ValueNode.createImmediate(value)))
    });
  }
});

// node_modules/kysely/dist/esm/schema/create-type-builder.js
class CreateTypeBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  asEnum(values) {
    return new CreateTypeBuilder({
      ...this.#props,
      node: CreateTypeNode.cloneWithEnum(this.#props.node, values)
    });
  }
  $call(func) {
    return func(this);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(CreateTypeBuilder, "don't await CreateTypeBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/operation-node/drop-type-node.js
var DropTypeNode = freeze({
  is(node) {
    return node.kind === "DropTypeNode";
  },
  create(name) {
    return freeze({
      kind: "DropTypeNode",
      name
    });
  },
  cloneWith(dropType, params) {
    return freeze({
      ...dropType,
      ...params
    });
  }
});

// node_modules/kysely/dist/esm/schema/drop-type-builder.js
class DropTypeBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  ifExists() {
    return new DropTypeBuilder({
      ...this.#props,
      node: DropTypeNode.cloneWith(this.#props.node, {
        ifExists: true
      })
    });
  }
  $call(func) {
    return func(this);
  }
  toOperationNode() {
    return this.#props.executor.transformQuery(this.#props.node, this.#props.queryId);
  }
  compile() {
    return this.#props.executor.compileQuery(this.toOperationNode(), this.#props.queryId);
  }
  async execute() {
    await this.#props.executor.executeQuery(this.compile(), this.#props.queryId);
  }
}
preventAwait(DropTypeBuilder, "don't await DropTypeBuilder instances directly. To execute the query you need to call `execute`");

// node_modules/kysely/dist/esm/parser/identifier-parser.js
function parseSchemableIdentifier(id) {
  const SCHEMA_SEPARATOR = ".";
  if (id.includes(SCHEMA_SEPARATOR)) {
    const parts = id.split(SCHEMA_SEPARATOR).map(trim3);
    if (parts.length === 2) {
      return SchemableIdentifierNode.createWithSchema(parts[0], parts[1]);
    } else {
      throw new Error(`invalid schemable identifier ${id}`);
    }
  } else {
    return SchemableIdentifierNode.create(id);
  }
}
function trim3(str) {
  return str.trim();
}

// node_modules/kysely/dist/esm/schema/schema.js
class SchemaModule {
  #executor;
  constructor(executor) {
    this.#executor = executor;
  }
  createTable(table) {
    return new CreateTableBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: CreateTableNode.create(parseTable(table))
    });
  }
  dropTable(table) {
    return new DropTableBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: DropTableNode.create(parseTable(table))
    });
  }
  createIndex(indexName) {
    return new CreateIndexBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: CreateIndexNode.create(indexName)
    });
  }
  dropIndex(indexName) {
    return new DropIndexBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: DropIndexNode.create(indexName)
    });
  }
  createSchema(schema) {
    return new CreateSchemaBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: CreateSchemaNode.create(schema)
    });
  }
  dropSchema(schema) {
    return new DropSchemaBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: DropSchemaNode.create(schema)
    });
  }
  alterTable(table) {
    return new AlterTableBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: AlterTableNode.create(parseTable(table))
    });
  }
  createView(viewName) {
    return new CreateViewBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: CreateViewNode.create(viewName)
    });
  }
  dropView(viewName) {
    return new DropViewBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: DropViewNode.create(viewName)
    });
  }
  createType(typeName) {
    return new CreateTypeBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: CreateTypeNode.create(parseSchemableIdentifier(typeName))
    });
  }
  dropType(typeName) {
    return new DropTypeBuilder({
      queryId: createQueryId(),
      executor: this.#executor,
      node: DropTypeNode.create(parseSchemableIdentifier(typeName))
    });
  }
  withPlugin(plugin) {
    return new SchemaModule(this.#executor.withPlugin(plugin));
  }
  withoutPlugins() {
    return new SchemaModule(this.#executor.withoutPlugins());
  }
  withSchema(schema) {
    return new SchemaModule(this.#executor.withPluginAtFront(new WithSchemaPlugin(schema)));
  }
}

// node_modules/kysely/dist/esm/dynamic/dynamic.js
class DynamicModule {
  ref(reference) {
    return new DynamicReferenceBuilder(reference);
  }
}

// node_modules/kysely/dist/esm/driver/default-connection-provider.js
class DefaultConnectionProvider {
  #driver;
  constructor(driver) {
    this.#driver = driver;
  }
  async provideConnection(consumer) {
    const connection = await this.#driver.acquireConnection();
    try {
      return await consumer(connection);
    } finally {
      await this.#driver.releaseConnection(connection);
    }
  }
}

// node_modules/kysely/dist/esm/query-executor/default-query-executor.js
class DefaultQueryExecutor extends QueryExecutorBase {
  #compiler;
  #adapter;
  #connectionProvider;
  constructor(compiler, adapter, connectionProvider, plugins = []) {
    super(plugins);
    this.#compiler = compiler;
    this.#adapter = adapter;
    this.#connectionProvider = connectionProvider;
  }
  get adapter() {
    return this.#adapter;
  }
  compileQuery(node) {
    return this.#compiler.compileQuery(node);
  }
  provideConnection(consumer) {
    return this.#connectionProvider.provideConnection(consumer);
  }
  withPlugins(plugins) {
    return new DefaultQueryExecutor(this.#compiler, this.#adapter, this.#connectionProvider, [...this.plugins, ...plugins]);
  }
  withPlugin(plugin) {
    return new DefaultQueryExecutor(this.#compiler, this.#adapter, this.#connectionProvider, [...this.plugins, plugin]);
  }
  withPluginAtFront(plugin) {
    return new DefaultQueryExecutor(this.#compiler, this.#adapter, this.#connectionProvider, [plugin, ...this.plugins]);
  }
  withConnectionProvider(connectionProvider) {
    return new DefaultQueryExecutor(this.#compiler, this.#adapter, connectionProvider, [...this.plugins]);
  }
  withoutPlugins() {
    return new DefaultQueryExecutor(this.#compiler, this.#adapter, this.#connectionProvider, []);
  }
}

// node_modules/kysely/dist/esm/util/performance-now.js
function performanceNow() {
  if (typeof performance !== "undefined" && isFunction(performance.now)) {
    return performance.now();
  } else {
    return Date.now();
  }
}

// node_modules/kysely/dist/esm/driver/runtime-driver.js
class RuntimeDriver {
  #driver;
  #log;
  #initPromise;
  #initDone;
  #destroyPromise;
  #connections = new WeakSet;
  constructor(driver, log) {
    this.#initDone = false;
    this.#driver = driver;
    this.#log = log;
  }
  async init() {
    if (this.#destroyPromise) {
      throw new Error("driver has already been destroyed");
    }
    if (!this.#initPromise) {
      this.#initPromise = this.#driver.init().then(() => {
        this.#initDone = true;
      }).catch((err) => {
        this.#initPromise = undefined;
        return Promise.reject(err);
      });
    }
    await this.#initPromise;
  }
  async acquireConnection() {
    if (this.#destroyPromise) {
      throw new Error("driver has already been destroyed");
    }
    if (!this.#initDone) {
      await this.init();
    }
    const connection = await this.#driver.acquireConnection();
    if (!this.#connections.has(connection)) {
      if (this.#needsLogging()) {
        this.#addLogging(connection);
      }
      this.#connections.add(connection);
    }
    return connection;
  }
  async releaseConnection(connection) {
    await this.#driver.releaseConnection(connection);
  }
  beginTransaction(connection, settings) {
    return this.#driver.beginTransaction(connection, settings);
  }
  commitTransaction(connection) {
    return this.#driver.commitTransaction(connection);
  }
  rollbackTransaction(connection) {
    return this.#driver.rollbackTransaction(connection);
  }
  async destroy() {
    if (!this.#initPromise) {
      return;
    }
    await this.#initPromise;
    if (!this.#destroyPromise) {
      this.#destroyPromise = this.#driver.destroy().catch((err) => {
        this.#destroyPromise = undefined;
        return Promise.reject(err);
      });
    }
    await this.#destroyPromise;
  }
  #needsLogging() {
    return this.#log.isLevelEnabled("query") || this.#log.isLevelEnabled("error");
  }
  #addLogging(connection) {
    const executeQuery = connection.executeQuery;
    connection.executeQuery = async (compiledQuery) => {
      let caughtError;
      const startTime = performanceNow();
      try {
        return await executeQuery.call(connection, compiledQuery);
      } catch (error) {
        caughtError = error;
        await this.#logError(error, compiledQuery, startTime);
        throw error;
      } finally {
        if (!caughtError) {
          await this.#logQuery(compiledQuery, startTime);
        }
      }
    };
  }
  async#logError(error, compiledQuery, startTime) {
    await this.#log.error(() => ({
      level: "error",
      error,
      query: compiledQuery,
      queryDurationMillis: this.#calculateDurationMillis(startTime)
    }));
  }
  async#logQuery(compiledQuery, startTime) {
    await this.#log.query(() => ({
      level: "query",
      query: compiledQuery,
      queryDurationMillis: this.#calculateDurationMillis(startTime)
    }));
  }
  #calculateDurationMillis(startTime) {
    return performanceNow() - startTime;
  }
}

// node_modules/kysely/dist/esm/driver/single-connection-provider.js
var ignoreError = () => {};

class SingleConnectionProvider {
  #connection;
  #runningPromise;
  constructor(connection) {
    this.#connection = connection;
  }
  async provideConnection(consumer) {
    while (this.#runningPromise) {
      await this.#runningPromise.catch(ignoreError);
    }
    this.#runningPromise = this.#run(consumer).finally(() => {
      this.#runningPromise = undefined;
    });
    return this.#runningPromise;
  }
  async#run(runner) {
    return await runner(this.#connection);
  }
}

// node_modules/kysely/dist/esm/driver/driver.js
var TRANSACTION_ISOLATION_LEVELS = [
  "read uncommitted",
  "read committed",
  "repeatable read",
  "serializable",
  "snapshot"
];

// node_modules/kysely/dist/esm/util/log.js
var LOG_LEVELS = freeze(["query", "error"]);

class Log {
  #levels;
  #logger;
  constructor(config) {
    if (isFunction(config)) {
      this.#logger = config;
      this.#levels = freeze({
        query: true,
        error: true
      });
    } else {
      this.#logger = defaultLogger;
      this.#levels = freeze({
        query: config.includes("query"),
        error: config.includes("error")
      });
    }
  }
  isLevelEnabled(level) {
    return this.#levels[level];
  }
  async query(getEvent) {
    if (this.#levels.query) {
      await this.#logger(getEvent());
    }
  }
  async error(getEvent) {
    if (this.#levels.error) {
      await this.#logger(getEvent());
    }
  }
}
function defaultLogger(event) {
  if (event.level === "query") {
    console.log(`kysely:query: ${event.query.sql}`);
    console.log(`kysely:query: duration: ${event.queryDurationMillis.toFixed(1)}ms`);
  } else if (event.level === "error") {
    if (event.error instanceof Error) {
      console.error(`kysely:error: ${event.error.stack ?? event.error.message}`);
    } else {
      console.error(`kysely:error: ${JSON.stringify({
        error: event.error,
        query: event.query.sql,
        queryDurationMillis: event.queryDurationMillis
      })}`);
    }
  }
}

// node_modules/kysely/dist/esm/util/compilable.js
function isCompilable(value) {
  return isObject(value) && isFunction(value.compile);
}

// node_modules/kysely/dist/esm/kysely.js
class Kysely extends QueryCreator {
  #props;
  constructor(args) {
    let superProps;
    let props;
    if (isKyselyProps(args)) {
      superProps = { executor: args.executor };
      props = { ...args };
    } else {
      const dialect = args.dialect;
      const driver = dialect.createDriver();
      const compiler = dialect.createQueryCompiler();
      const adapter = dialect.createAdapter();
      const log = new Log(args.log ?? []);
      const runtimeDriver = new RuntimeDriver(driver, log);
      const connectionProvider = new DefaultConnectionProvider(runtimeDriver);
      const executor = new DefaultQueryExecutor(compiler, adapter, connectionProvider, args.plugins ?? []);
      superProps = { executor };
      props = {
        config: args,
        executor,
        dialect,
        driver: runtimeDriver
      };
    }
    super(superProps);
    this.#props = freeze(props);
  }
  get schema() {
    return new SchemaModule(this.#props.executor);
  }
  get dynamic() {
    return new DynamicModule;
  }
  get introspection() {
    return this.#props.dialect.createIntrospector(this.withoutPlugins());
  }
  case(value) {
    return new CaseBuilder({
      node: CaseNode.create(isUndefined(value) ? undefined : parseExpression(value))
    });
  }
  get fn() {
    return createFunctionModule();
  }
  transaction() {
    return new TransactionBuilder({ ...this.#props });
  }
  connection() {
    return new ConnectionBuilder({ ...this.#props });
  }
  withPlugin(plugin) {
    return new Kysely({
      ...this.#props,
      executor: this.#props.executor.withPlugin(plugin)
    });
  }
  withoutPlugins() {
    return new Kysely({
      ...this.#props,
      executor: this.#props.executor.withoutPlugins()
    });
  }
  withSchema(schema) {
    return new Kysely({
      ...this.#props,
      executor: this.#props.executor.withPluginAtFront(new WithSchemaPlugin(schema))
    });
  }
  withTables() {
    return new Kysely({ ...this.#props });
  }
  async destroy() {
    await this.#props.driver.destroy();
  }
  get isTransaction() {
    return false;
  }
  getExecutor() {
    return this.#props.executor;
  }
  executeQuery(query, queryId = createQueryId()) {
    const compiledQuery = isCompilable(query) ? query.compile() : query;
    return this.getExecutor().executeQuery(compiledQuery, queryId);
  }
}

class Transaction extends Kysely {
  #props;
  constructor(props) {
    super(props);
    this.#props = props;
  }
  get isTransaction() {
    return true;
  }
  transaction() {
    throw new Error("calling the transaction method for a Transaction is not supported");
  }
  connection() {
    throw new Error("calling the connection method for a Transaction is not supported");
  }
  async destroy() {
    throw new Error("calling the destroy method for a Transaction is not supported");
  }
  withPlugin(plugin) {
    return new Transaction({
      ...this.#props,
      executor: this.#props.executor.withPlugin(plugin)
    });
  }
  withoutPlugins() {
    return new Transaction({
      ...this.#props,
      executor: this.#props.executor.withoutPlugins()
    });
  }
  withSchema(schema) {
    return new Transaction({
      ...this.#props,
      executor: this.#props.executor.withPluginAtFront(new WithSchemaPlugin(schema))
    });
  }
  withTables() {
    return new Transaction({ ...this.#props });
  }
}
function isKyselyProps(obj) {
  return isObject(obj) && isObject(obj.config) && isObject(obj.driver) && isObject(obj.executor) && isObject(obj.dialect);
}

class ConnectionBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  async execute(callback) {
    return this.#props.executor.provideConnection(async (connection) => {
      const executor = this.#props.executor.withConnectionProvider(new SingleConnectionProvider(connection));
      const db = new Kysely({
        ...this.#props,
        executor
      });
      return await callback(db);
    });
  }
}
preventAwait(ConnectionBuilder, "don't await ConnectionBuilder instances directly. To execute the query you need to call the `execute` method");

class TransactionBuilder {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  setIsolationLevel(isolationLevel) {
    return new TransactionBuilder({
      ...this.#props,
      isolationLevel
    });
  }
  async execute(callback) {
    const { isolationLevel, ...kyselyProps } = this.#props;
    const settings = { isolationLevel };
    validateTransactionSettings(settings);
    return this.#props.executor.provideConnection(async (connection) => {
      const executor = this.#props.executor.withConnectionProvider(new SingleConnectionProvider(connection));
      const transaction = new Transaction({
        ...kyselyProps,
        executor
      });
      try {
        await this.#props.driver.beginTransaction(connection, settings);
        const result = await callback(transaction);
        await this.#props.driver.commitTransaction(connection);
        return result;
      } catch (error) {
        await this.#props.driver.rollbackTransaction(connection);
        throw error;
      }
    });
  }
}
preventAwait(TransactionBuilder, "don't await TransactionBuilder instances directly. To execute the transaction you need to call the `execute` method");
function validateTransactionSettings(settings) {
  if (settings.isolationLevel && !TRANSACTION_ISOLATION_LEVELS.includes(settings.isolationLevel)) {
    throw new Error(`invalid transaction isolation level ${settings.isolationLevel}`);
  }
}

// node_modules/kysely/dist/esm/raw-builder/raw-builder.js
class RawBuilderImpl {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  get expressionType() {
    return;
  }
  get isRawBuilder() {
    return true;
  }
  as(alias) {
    return new AliasedRawBuilderImpl(this, alias);
  }
  $castTo() {
    return new RawBuilderImpl({ ...this.#props });
  }
  $notNull() {
    return new RawBuilderImpl(this.#props);
  }
  withPlugin(plugin) {
    return new RawBuilderImpl({
      ...this.#props,
      plugins: this.#props.plugins !== undefined ? freeze([...this.#props.plugins, plugin]) : freeze([plugin])
    });
  }
  toOperationNode() {
    return this.#toOperationNode(this.#getExecutor());
  }
  compile(executorProvider) {
    return this.#compile(this.#getExecutor(executorProvider));
  }
  async execute(executorProvider) {
    const executor = this.#getExecutor(executorProvider);
    return executor.executeQuery(this.#compile(executor), this.#props.queryId);
  }
  #getExecutor(executorProvider) {
    const executor = executorProvider !== undefined ? executorProvider.getExecutor() : NOOP_QUERY_EXECUTOR;
    return this.#props.plugins !== undefined ? executor.withPlugins(this.#props.plugins) : executor;
  }
  #toOperationNode(executor) {
    return executor.transformQuery(this.#props.rawNode, this.#props.queryId);
  }
  #compile(executor) {
    return executor.compileQuery(this.#toOperationNode(executor), this.#props.queryId);
  }
}
function createRawBuilder(props) {
  return new RawBuilderImpl(props);
}
preventAwait(RawBuilderImpl, "don't await RawBuilder instances directly. To execute the query you need to call `execute`");

class AliasedRawBuilderImpl {
  #rawBuilder;
  #alias;
  constructor(rawBuilder, alias) {
    this.#rawBuilder = rawBuilder;
    this.#alias = alias;
  }
  get expression() {
    return this.#rawBuilder;
  }
  get alias() {
    return this.#alias;
  }
  get rawBuilder() {
    return this.#rawBuilder;
  }
  toOperationNode() {
    return AliasNode.create(this.#rawBuilder.toOperationNode(), isOperationNodeSource(this.#alias) ? this.#alias.toOperationNode() : IdentifierNode.create(this.#alias));
  }
}
preventAwait(AliasedRawBuilderImpl, "don't await AliasedRawBuilder instances directly. AliasedRawBuilder should never be executed directly since it's always a part of another query.");

// node_modules/kysely/dist/esm/raw-builder/sql.js
var sql = Object.assign((sqlFragments, ...parameters) => {
  return createRawBuilder({
    queryId: createQueryId(),
    rawNode: RawNode.create(sqlFragments, parameters?.map(parseParameter) ?? [])
  });
}, {
  ref(columnReference) {
    return createRawBuilder({
      queryId: createQueryId(),
      rawNode: RawNode.createWithChild(parseStringReference(columnReference))
    });
  },
  val(value) {
    return createRawBuilder({
      queryId: createQueryId(),
      rawNode: RawNode.createWithChild(parseValueExpression(value))
    });
  },
  value(value) {
    return this.val(value);
  },
  table(tableReference) {
    return createRawBuilder({
      queryId: createQueryId(),
      rawNode: RawNode.createWithChild(parseTable(tableReference))
    });
  },
  id(...ids) {
    const fragments = new Array(ids.length + 1).fill(".");
    fragments[0] = "";
    fragments[fragments.length - 1] = "";
    return createRawBuilder({
      queryId: createQueryId(),
      rawNode: RawNode.create(fragments, ids.map(IdentifierNode.create))
    });
  },
  lit(value) {
    return createRawBuilder({
      queryId: createQueryId(),
      rawNode: RawNode.createWithChild(ValueNode.createImmediate(value))
    });
  },
  literal(value) {
    return this.lit(value);
  },
  raw(sql2) {
    return createRawBuilder({
      queryId: createQueryId(),
      rawNode: RawNode.createWithSql(sql2)
    });
  },
  join(array, separator = sql`, `) {
    const nodes = new Array(2 * array.length - 1);
    const sep = separator.toOperationNode();
    for (let i = 0;i < array.length; ++i) {
      nodes[2 * i] = parseParameter(array[i]);
      if (i !== array.length - 1) {
        nodes[2 * i + 1] = sep;
      }
    }
    return createRawBuilder({
      queryId: createQueryId(),
      rawNode: RawNode.createWithChildren(nodes)
    });
  }
});
function parseParameter(param) {
  if (isOperationNodeSource(param)) {
    return param.toOperationNode();
  }
  return parseValueExpression(param);
}

// node_modules/kysely/dist/esm/operation-node/operation-node-visitor.js
class OperationNodeVisitor {
  nodeStack = [];
  get parentNode() {
    return this.nodeStack[this.nodeStack.length - 2];
  }
  #visitors = freeze({
    AliasNode: this.visitAlias.bind(this),
    ColumnNode: this.visitColumn.bind(this),
    IdentifierNode: this.visitIdentifier.bind(this),
    SchemableIdentifierNode: this.visitSchemableIdentifier.bind(this),
    RawNode: this.visitRaw.bind(this),
    ReferenceNode: this.visitReference.bind(this),
    SelectQueryNode: this.visitSelectQuery.bind(this),
    SelectionNode: this.visitSelection.bind(this),
    TableNode: this.visitTable.bind(this),
    FromNode: this.visitFrom.bind(this),
    SelectAllNode: this.visitSelectAll.bind(this),
    AndNode: this.visitAnd.bind(this),
    OrNode: this.visitOr.bind(this),
    ValueNode: this.visitValue.bind(this),
    ValueListNode: this.visitValueList.bind(this),
    PrimitiveValueListNode: this.visitPrimitiveValueList.bind(this),
    ParensNode: this.visitParens.bind(this),
    JoinNode: this.visitJoin.bind(this),
    OperatorNode: this.visitOperator.bind(this),
    WhereNode: this.visitWhere.bind(this),
    InsertQueryNode: this.visitInsertQuery.bind(this),
    DeleteQueryNode: this.visitDeleteQuery.bind(this),
    ReturningNode: this.visitReturning.bind(this),
    CreateTableNode: this.visitCreateTable.bind(this),
    AddColumnNode: this.visitAddColumn.bind(this),
    ColumnDefinitionNode: this.visitColumnDefinition.bind(this),
    DropTableNode: this.visitDropTable.bind(this),
    DataTypeNode: this.visitDataType.bind(this),
    OrderByNode: this.visitOrderBy.bind(this),
    OrderByItemNode: this.visitOrderByItem.bind(this),
    GroupByNode: this.visitGroupBy.bind(this),
    GroupByItemNode: this.visitGroupByItem.bind(this),
    UpdateQueryNode: this.visitUpdateQuery.bind(this),
    ColumnUpdateNode: this.visitColumnUpdate.bind(this),
    LimitNode: this.visitLimit.bind(this),
    OffsetNode: this.visitOffset.bind(this),
    OnConflictNode: this.visitOnConflict.bind(this),
    OnDuplicateKeyNode: this.visitOnDuplicateKey.bind(this),
    CreateIndexNode: this.visitCreateIndex.bind(this),
    DropIndexNode: this.visitDropIndex.bind(this),
    ListNode: this.visitList.bind(this),
    PrimaryKeyConstraintNode: this.visitPrimaryKeyConstraint.bind(this),
    UniqueConstraintNode: this.visitUniqueConstraint.bind(this),
    ReferencesNode: this.visitReferences.bind(this),
    CheckConstraintNode: this.visitCheckConstraint.bind(this),
    WithNode: this.visitWith.bind(this),
    CommonTableExpressionNode: this.visitCommonTableExpression.bind(this),
    CommonTableExpressionNameNode: this.visitCommonTableExpressionName.bind(this),
    HavingNode: this.visitHaving.bind(this),
    CreateSchemaNode: this.visitCreateSchema.bind(this),
    DropSchemaNode: this.visitDropSchema.bind(this),
    AlterTableNode: this.visitAlterTable.bind(this),
    DropColumnNode: this.visitDropColumn.bind(this),
    RenameColumnNode: this.visitRenameColumn.bind(this),
    AlterColumnNode: this.visitAlterColumn.bind(this),
    ModifyColumnNode: this.visitModifyColumn.bind(this),
    AddConstraintNode: this.visitAddConstraint.bind(this),
    DropConstraintNode: this.visitDropConstraint.bind(this),
    ForeignKeyConstraintNode: this.visitForeignKeyConstraint.bind(this),
    CreateViewNode: this.visitCreateView.bind(this),
    DropViewNode: this.visitDropView.bind(this),
    GeneratedNode: this.visitGenerated.bind(this),
    DefaultValueNode: this.visitDefaultValue.bind(this),
    OnNode: this.visitOn.bind(this),
    ValuesNode: this.visitValues.bind(this),
    SelectModifierNode: this.visitSelectModifier.bind(this),
    CreateTypeNode: this.visitCreateType.bind(this),
    DropTypeNode: this.visitDropType.bind(this),
    ExplainNode: this.visitExplain.bind(this),
    DefaultInsertValueNode: this.visitDefaultInsertValue.bind(this),
    AggregateFunctionNode: this.visitAggregateFunction.bind(this),
    OverNode: this.visitOver.bind(this),
    PartitionByNode: this.visitPartitionBy.bind(this),
    PartitionByItemNode: this.visitPartitionByItem.bind(this),
    SetOperationNode: this.visitSetOperation.bind(this),
    BinaryOperationNode: this.visitBinaryOperation.bind(this),
    UnaryOperationNode: this.visitUnaryOperation.bind(this),
    UsingNode: this.visitUsing.bind(this),
    FunctionNode: this.visitFunction.bind(this),
    CaseNode: this.visitCase.bind(this),
    WhenNode: this.visitWhen.bind(this),
    JSONReferenceNode: this.visitJSONReference.bind(this),
    JSONPathNode: this.visitJSONPath.bind(this),
    JSONPathLegNode: this.visitJSONPathLeg.bind(this),
    JSONOperatorChainNode: this.visitJSONOperatorChain.bind(this),
    TupleNode: this.visitTuple.bind(this),
    MergeQueryNode: this.visitMergeQuery.bind(this),
    MatchedNode: this.visitMatched.bind(this),
    AddIndexNode: this.visitAddIndex.bind(this),
    CastNode: this.visitCast.bind(this),
    FetchNode: this.visitFetch.bind(this),
    TopNode: this.visitTop.bind(this),
    OutputNode: this.visitOutput.bind(this)
  });
  visitNode = (node) => {
    this.nodeStack.push(node);
    this.#visitors[node.kind](node);
    this.nodeStack.pop();
  };
}

// node_modules/kysely/dist/esm/query-compiler/default-query-compiler.js
class DefaultQueryCompiler extends OperationNodeVisitor {
  #sql = "";
  #parameters = [];
  get numParameters() {
    return this.#parameters.length;
  }
  compileQuery(node) {
    this.#sql = "";
    this.#parameters = [];
    this.nodeStack.splice(0, this.nodeStack.length);
    this.visitNode(node);
    return freeze({
      query: node,
      sql: this.getSql(),
      parameters: [...this.#parameters]
    });
  }
  getSql() {
    return this.#sql;
  }
  visitSelectQuery(node) {
    const wrapInParens = this.parentNode !== undefined && !ParensNode.is(this.parentNode) && !InsertQueryNode.is(this.parentNode) && !CreateTableNode.is(this.parentNode) && !CreateViewNode.is(this.parentNode) && !SetOperationNode.is(this.parentNode);
    if (this.parentNode === undefined && node.explain) {
      this.visitNode(node.explain);
      this.append(" ");
    }
    if (wrapInParens) {
      this.append("(");
    }
    if (node.with) {
      this.visitNode(node.with);
      this.append(" ");
    }
    this.append("select");
    if (node.distinctOn) {
      this.append(" ");
      this.compileDistinctOn(node.distinctOn);
    }
    if (node.frontModifiers?.length) {
      this.append(" ");
      this.compileList(node.frontModifiers, " ");
    }
    if (node.top) {
      this.append(" ");
      this.visitNode(node.top);
    }
    if (node.selections) {
      this.append(" ");
      this.compileList(node.selections);
    }
    if (node.from) {
      this.append(" ");
      this.visitNode(node.from);
    }
    if (node.joins) {
      this.append(" ");
      this.compileList(node.joins, " ");
    }
    if (node.where) {
      this.append(" ");
      this.visitNode(node.where);
    }
    if (node.groupBy) {
      this.append(" ");
      this.visitNode(node.groupBy);
    }
    if (node.having) {
      this.append(" ");
      this.visitNode(node.having);
    }
    if (node.setOperations) {
      this.append(" ");
      this.compileList(node.setOperations, " ");
    }
    if (node.orderBy) {
      this.append(" ");
      this.visitNode(node.orderBy);
    }
    if (node.limit) {
      this.append(" ");
      this.visitNode(node.limit);
    }
    if (node.offset) {
      this.append(" ");
      this.visitNode(node.offset);
    }
    if (node.fetch) {
      this.append(" ");
      this.visitNode(node.fetch);
    }
    if (node.endModifiers?.length) {
      this.append(" ");
      this.compileList(this.sortSelectModifiers([...node.endModifiers]), " ");
    }
    if (wrapInParens) {
      this.append(")");
    }
  }
  visitFrom(node) {
    this.append("from ");
    this.compileList(node.froms);
  }
  visitSelection(node) {
    this.visitNode(node.selection);
  }
  visitColumn(node) {
    this.visitNode(node.column);
  }
  compileDistinctOn(expressions) {
    this.append("distinct on (");
    this.compileList(expressions);
    this.append(")");
  }
  compileList(nodes, separator = ", ") {
    const lastIndex = nodes.length - 1;
    for (let i = 0;i <= lastIndex; i++) {
      this.visitNode(nodes[i]);
      if (i < lastIndex) {
        this.append(separator);
      }
    }
  }
  visitWhere(node) {
    this.append("where ");
    this.visitNode(node.where);
  }
  visitHaving(node) {
    this.append("having ");
    this.visitNode(node.having);
  }
  visitInsertQuery(node) {
    const rootQueryNode = this.nodeStack.find(QueryNode.is);
    const isSubQuery = rootQueryNode !== node;
    if (!isSubQuery && node.explain) {
      this.visitNode(node.explain);
      this.append(" ");
    }
    if (isSubQuery && !MergeQueryNode.is(rootQueryNode)) {
      this.append("(");
    }
    if (node.with) {
      this.visitNode(node.with);
      this.append(" ");
    }
    this.append(node.replace ? "replace" : "insert");
    if (node.ignore) {
      this.append(" ignore");
    }
    if (node.top) {
      this.append(" ");
      this.visitNode(node.top);
    }
    if (node.into) {
      this.append(" into ");
      this.visitNode(node.into);
    }
    if (node.columns) {
      this.append(" (");
      this.compileList(node.columns);
      this.append(")");
    }
    if (node.output) {
      this.append(" ");
      this.visitNode(node.output);
    }
    if (node.values) {
      this.append(" ");
      this.visitNode(node.values);
    }
    if (node.defaultValues) {
      this.append(" ");
      this.append("default values");
    }
    if (node.onConflict) {
      this.append(" ");
      this.visitNode(node.onConflict);
    }
    if (node.onDuplicateKey) {
      this.append(" ");
      this.visitNode(node.onDuplicateKey);
    }
    if (node.returning) {
      this.append(" ");
      this.visitNode(node.returning);
    }
    if (isSubQuery && !MergeQueryNode.is(rootQueryNode)) {
      this.append(")");
    }
  }
  visitValues(node) {
    this.append("values ");
    this.compileList(node.values);
  }
  visitDeleteQuery(node) {
    const isSubQuery = this.nodeStack.find(QueryNode.is) !== node;
    if (!isSubQuery && node.explain) {
      this.visitNode(node.explain);
      this.append(" ");
    }
    if (isSubQuery) {
      this.append("(");
    }
    if (node.with) {
      this.visitNode(node.with);
      this.append(" ");
    }
    this.append("delete ");
    if (node.top) {
      this.visitNode(node.top);
      this.append(" ");
    }
    this.visitNode(node.from);
    if (node.output) {
      this.append(" ");
      this.visitNode(node.output);
    }
    if (node.using) {
      this.append(" ");
      this.visitNode(node.using);
    }
    if (node.joins) {
      this.append(" ");
      this.compileList(node.joins, " ");
    }
    if (node.where) {
      this.append(" ");
      this.visitNode(node.where);
    }
    if (node.orderBy) {
      this.append(" ");
      this.visitNode(node.orderBy);
    }
    if (node.limit) {
      this.append(" ");
      this.visitNode(node.limit);
    }
    if (node.returning) {
      this.append(" ");
      this.visitNode(node.returning);
    }
    if (isSubQuery) {
      this.append(")");
    }
  }
  visitReturning(node) {
    this.append("returning ");
    this.compileList(node.selections);
  }
  visitAlias(node) {
    this.visitNode(node.node);
    this.append(" as ");
    this.visitNode(node.alias);
  }
  visitReference(node) {
    if (node.table) {
      this.visitNode(node.table);
      this.append(".");
    }
    this.visitNode(node.column);
  }
  visitSelectAll(_) {
    this.append("*");
  }
  visitIdentifier(node) {
    this.append(this.getLeftIdentifierWrapper());
    this.compileUnwrappedIdentifier(node);
    this.append(this.getRightIdentifierWrapper());
  }
  compileUnwrappedIdentifier(node) {
    if (!isString(node.name)) {
      throw new Error("a non-string identifier was passed to compileUnwrappedIdentifier.");
    }
    this.append(this.sanitizeIdentifier(node.name));
  }
  visitAnd(node) {
    this.visitNode(node.left);
    this.append(" and ");
    this.visitNode(node.right);
  }
  visitOr(node) {
    this.visitNode(node.left);
    this.append(" or ");
    this.visitNode(node.right);
  }
  visitValue(node) {
    if (node.immediate) {
      this.appendImmediateValue(node.value);
    } else {
      this.appendValue(node.value);
    }
  }
  visitValueList(node) {
    this.append("(");
    this.compileList(node.values);
    this.append(")");
  }
  visitTuple(node) {
    this.append("(");
    this.compileList(node.values);
    this.append(")");
  }
  visitPrimitiveValueList(node) {
    this.append("(");
    const { values } = node;
    for (let i = 0;i < values.length; ++i) {
      this.appendValue(values[i]);
      if (i !== values.length - 1) {
        this.append(", ");
      }
    }
    this.append(")");
  }
  visitParens(node) {
    this.append("(");
    this.visitNode(node.node);
    this.append(")");
  }
  visitJoin(node) {
    this.append(JOIN_TYPE_SQL[node.joinType]);
    this.append(" ");
    this.visitNode(node.table);
    if (node.on) {
      this.append(" ");
      this.visitNode(node.on);
    }
  }
  visitOn(node) {
    this.append("on ");
    this.visitNode(node.on);
  }
  visitRaw(node) {
    const { sqlFragments, parameters: params } = node;
    for (let i = 0;i < sqlFragments.length; ++i) {
      this.append(sqlFragments[i]);
      if (params.length > i) {
        this.visitNode(params[i]);
      }
    }
  }
  visitOperator(node) {
    this.append(node.operator);
  }
  visitTable(node) {
    this.visitNode(node.table);
  }
  visitSchemableIdentifier(node) {
    if (node.schema) {
      this.visitNode(node.schema);
      this.append(".");
    }
    this.visitNode(node.identifier);
  }
  visitCreateTable(node) {
    this.append("create ");
    if (node.frontModifiers && node.frontModifiers.length > 0) {
      this.compileList(node.frontModifiers, " ");
      this.append(" ");
    }
    if (node.temporary) {
      this.append("temporary ");
    }
    this.append("table ");
    if (node.ifNotExists) {
      this.append("if not exists ");
    }
    this.visitNode(node.table);
    if (node.selectQuery) {
      this.append(" as ");
      this.visitNode(node.selectQuery);
    } else {
      this.append(" (");
      this.compileList([...node.columns, ...node.constraints ?? []]);
      this.append(")");
      if (node.onCommit) {
        this.append(" on commit ");
        this.append(node.onCommit);
      }
      if (node.endModifiers && node.endModifiers.length > 0) {
        this.append(" ");
        this.compileList(node.endModifiers, " ");
      }
    }
  }
  visitColumnDefinition(node) {
    if (node.ifNotExists) {
      this.append("if not exists ");
    }
    this.visitNode(node.column);
    this.append(" ");
    this.visitNode(node.dataType);
    if (node.unsigned) {
      this.append(" unsigned");
    }
    if (node.frontModifiers && node.frontModifiers.length > 0) {
      this.append(" ");
      this.compileList(node.frontModifiers, " ");
    }
    if (node.generated) {
      this.append(" ");
      this.visitNode(node.generated);
    }
    if (node.identity) {
      this.append(" identity");
    }
    if (node.defaultTo) {
      this.append(" ");
      this.visitNode(node.defaultTo);
    }
    if (node.notNull) {
      this.append(" not null");
    }
    if (node.unique) {
      this.append(" unique");
    }
    if (node.nullsNotDistinct) {
      this.append(" nulls not distinct");
    }
    if (node.primaryKey) {
      this.append(" primary key");
    }
    if (node.autoIncrement) {
      this.append(" ");
      this.append(this.getAutoIncrement());
    }
    if (node.references) {
      this.append(" ");
      this.visitNode(node.references);
    }
    if (node.check) {
      this.append(" ");
      this.visitNode(node.check);
    }
    if (node.endModifiers && node.endModifiers.length > 0) {
      this.append(" ");
      this.compileList(node.endModifiers, " ");
    }
  }
  getAutoIncrement() {
    return "auto_increment";
  }
  visitReferences(node) {
    this.append("references ");
    this.visitNode(node.table);
    this.append(" (");
    this.compileList(node.columns);
    this.append(")");
    if (node.onDelete) {
      this.append(" on delete ");
      this.append(node.onDelete);
    }
    if (node.onUpdate) {
      this.append(" on update ");
      this.append(node.onUpdate);
    }
  }
  visitDropTable(node) {
    this.append("drop table ");
    if (node.ifExists) {
      this.append("if exists ");
    }
    this.visitNode(node.table);
    if (node.cascade) {
      this.append(" cascade");
    }
  }
  visitDataType(node) {
    this.append(node.dataType);
  }
  visitOrderBy(node) {
    this.append("order by ");
    this.compileList(node.items);
  }
  visitOrderByItem(node) {
    this.visitNode(node.orderBy);
    if (node.direction) {
      this.append(" ");
      this.visitNode(node.direction);
    }
  }
  visitGroupBy(node) {
    this.append("group by ");
    this.compileList(node.items);
  }
  visitGroupByItem(node) {
    this.visitNode(node.groupBy);
  }
  visitUpdateQuery(node) {
    const rootQueryNode = this.nodeStack.find(QueryNode.is);
    const isSubQuery = rootQueryNode !== node;
    if (!isSubQuery && node.explain) {
      this.visitNode(node.explain);
      this.append(" ");
    }
    if (isSubQuery && !MergeQueryNode.is(rootQueryNode)) {
      this.append("(");
    }
    if (node.with) {
      this.visitNode(node.with);
      this.append(" ");
    }
    this.append("update ");
    if (node.top) {
      this.visitNode(node.top);
      this.append(" ");
    }
    if (node.table) {
      this.visitNode(node.table);
      this.append(" ");
    }
    this.append("set ");
    if (node.updates) {
      this.compileList(node.updates);
    }
    if (node.output) {
      this.append(" ");
      this.visitNode(node.output);
    }
    if (node.from) {
      this.append(" ");
      this.visitNode(node.from);
    }
    if (node.joins) {
      this.append(" ");
      this.compileList(node.joins, " ");
    }
    if (node.where) {
      this.append(" ");
      this.visitNode(node.where);
    }
    if (node.limit) {
      this.append(" ");
      this.visitNode(node.limit);
    }
    if (node.returning) {
      this.append(" ");
      this.visitNode(node.returning);
    }
    if (isSubQuery && !MergeQueryNode.is(rootQueryNode)) {
      this.append(")");
    }
  }
  visitColumnUpdate(node) {
    this.visitNode(node.column);
    this.append(" = ");
    this.visitNode(node.value);
  }
  visitLimit(node) {
    this.append("limit ");
    this.visitNode(node.limit);
  }
  visitOffset(node) {
    this.append("offset ");
    this.visitNode(node.offset);
  }
  visitOnConflict(node) {
    this.append("on conflict");
    if (node.columns) {
      this.append(" (");
      this.compileList(node.columns);
      this.append(")");
    } else if (node.constraint) {
      this.append(" on constraint ");
      this.visitNode(node.constraint);
    } else if (node.indexExpression) {
      this.append(" (");
      this.visitNode(node.indexExpression);
      this.append(")");
    }
    if (node.indexWhere) {
      this.append(" ");
      this.visitNode(node.indexWhere);
    }
    if (node.doNothing === true) {
      this.append(" do nothing");
    } else if (node.updates) {
      this.append(" do update set ");
      this.compileList(node.updates);
      if (node.updateWhere) {
        this.append(" ");
        this.visitNode(node.updateWhere);
      }
    }
  }
  visitOnDuplicateKey(node) {
    this.append("on duplicate key update ");
    this.compileList(node.updates);
  }
  visitCreateIndex(node) {
    this.append("create ");
    if (node.unique) {
      this.append("unique ");
    }
    this.append("index ");
    if (node.ifNotExists) {
      this.append("if not exists ");
    }
    this.visitNode(node.name);
    if (node.table) {
      this.append(" on ");
      this.visitNode(node.table);
    }
    if (node.using) {
      this.append(" using ");
      this.visitNode(node.using);
    }
    if (node.columns) {
      this.append(" (");
      this.compileList(node.columns);
      this.append(")");
    }
    if (node.nullsNotDistinct) {
      this.append(" nulls not distinct");
    }
    if (node.where) {
      this.append(" ");
      this.visitNode(node.where);
    }
  }
  visitDropIndex(node) {
    this.append("drop index ");
    if (node.ifExists) {
      this.append("if exists ");
    }
    this.visitNode(node.name);
    if (node.table) {
      this.append(" on ");
      this.visitNode(node.table);
    }
    if (node.cascade) {
      this.append(" cascade");
    }
  }
  visitCreateSchema(node) {
    this.append("create schema ");
    if (node.ifNotExists) {
      this.append("if not exists ");
    }
    this.visitNode(node.schema);
  }
  visitDropSchema(node) {
    this.append("drop schema ");
    if (node.ifExists) {
      this.append("if exists ");
    }
    this.visitNode(node.schema);
    if (node.cascade) {
      this.append(" cascade");
    }
  }
  visitPrimaryKeyConstraint(node) {
    if (node.name) {
      this.append("constraint ");
      this.visitNode(node.name);
      this.append(" ");
    }
    this.append("primary key (");
    this.compileList(node.columns);
    this.append(")");
  }
  visitUniqueConstraint(node) {
    if (node.name) {
      this.append("constraint ");
      this.visitNode(node.name);
      this.append(" ");
    }
    this.append("unique");
    if (node.nullsNotDistinct) {
      this.append(" nulls not distinct");
    }
    this.append(" (");
    this.compileList(node.columns);
    this.append(")");
  }
  visitCheckConstraint(node) {
    if (node.name) {
      this.append("constraint ");
      this.visitNode(node.name);
      this.append(" ");
    }
    this.append("check (");
    this.visitNode(node.expression);
    this.append(")");
  }
  visitForeignKeyConstraint(node) {
    if (node.name) {
      this.append("constraint ");
      this.visitNode(node.name);
      this.append(" ");
    }
    this.append("foreign key (");
    this.compileList(node.columns);
    this.append(") ");
    this.visitNode(node.references);
    if (node.onDelete) {
      this.append(" on delete ");
      this.append(node.onDelete);
    }
    if (node.onUpdate) {
      this.append(" on update ");
      this.append(node.onUpdate);
    }
  }
  visitList(node) {
    this.compileList(node.items);
  }
  visitWith(node) {
    this.append("with ");
    if (node.recursive) {
      this.append("recursive ");
    }
    this.compileList(node.expressions);
  }
  visitCommonTableExpression(node) {
    this.visitNode(node.name);
    this.append(" as ");
    if (isBoolean(node.materialized)) {
      if (!node.materialized) {
        this.append("not ");
      }
      this.append("materialized ");
    }
    this.visitNode(node.expression);
  }
  visitCommonTableExpressionName(node) {
    this.visitNode(node.table);
    if (node.columns) {
      this.append("(");
      this.compileList(node.columns);
      this.append(")");
    }
  }
  visitAlterTable(node) {
    this.append("alter table ");
    this.visitNode(node.table);
    this.append(" ");
    if (node.renameTo) {
      this.append("rename to ");
      this.visitNode(node.renameTo);
    }
    if (node.setSchema) {
      this.append("set schema ");
      this.visitNode(node.setSchema);
    }
    if (node.addConstraint) {
      this.visitNode(node.addConstraint);
    }
    if (node.dropConstraint) {
      this.visitNode(node.dropConstraint);
    }
    if (node.columnAlterations) {
      this.compileColumnAlterations(node.columnAlterations);
    }
    if (node.addIndex) {
      this.visitNode(node.addIndex);
    }
    if (node.dropIndex) {
      this.visitNode(node.dropIndex);
    }
  }
  visitAddColumn(node) {
    this.append("add column ");
    this.visitNode(node.column);
  }
  visitRenameColumn(node) {
    this.append("rename column ");
    this.visitNode(node.column);
    this.append(" to ");
    this.visitNode(node.renameTo);
  }
  visitDropColumn(node) {
    this.append("drop column ");
    this.visitNode(node.column);
  }
  visitAlterColumn(node) {
    this.append("alter column ");
    this.visitNode(node.column);
    this.append(" ");
    if (node.dataType) {
      if (this.announcesNewColumnDataType()) {
        this.append("type ");
      }
      this.visitNode(node.dataType);
      if (node.dataTypeExpression) {
        this.append("using ");
        this.visitNode(node.dataTypeExpression);
      }
    }
    if (node.setDefault) {
      this.append("set default ");
      this.visitNode(node.setDefault);
    }
    if (node.dropDefault) {
      this.append("drop default");
    }
    if (node.setNotNull) {
      this.append("set not null");
    }
    if (node.dropNotNull) {
      this.append("drop not null");
    }
  }
  visitModifyColumn(node) {
    this.append("modify column ");
    this.visitNode(node.column);
  }
  visitAddConstraint(node) {
    this.append("add ");
    this.visitNode(node.constraint);
  }
  visitDropConstraint(node) {
    this.append("drop constraint ");
    if (node.ifExists) {
      this.append("if exists ");
    }
    this.visitNode(node.constraintName);
    if (node.modifier === "cascade") {
      this.append(" cascade");
    } else if (node.modifier === "restrict") {
      this.append(" restrict");
    }
  }
  visitSetOperation(node) {
    this.append(node.operator);
    this.append(" ");
    if (node.all) {
      this.append("all ");
    }
    this.visitNode(node.expression);
  }
  visitCreateView(node) {
    this.append("create ");
    if (node.orReplace) {
      this.append("or replace ");
    }
    if (node.materialized) {
      this.append("materialized ");
    }
    if (node.temporary) {
      this.append("temporary ");
    }
    this.append("view ");
    if (node.ifNotExists) {
      this.append("if not exists ");
    }
    this.visitNode(node.name);
    this.append(" ");
    if (node.columns) {
      this.append("(");
      this.compileList(node.columns);
      this.append(") ");
    }
    if (node.as) {
      this.append("as ");
      this.visitNode(node.as);
    }
  }
  visitDropView(node) {
    this.append("drop ");
    if (node.materialized) {
      this.append("materialized ");
    }
    this.append("view ");
    if (node.ifExists) {
      this.append("if exists ");
    }
    this.visitNode(node.name);
    if (node.cascade) {
      this.append(" cascade");
    }
  }
  visitGenerated(node) {
    this.append("generated ");
    if (node.always) {
      this.append("always ");
    }
    if (node.byDefault) {
      this.append("by default ");
    }
    this.append("as ");
    if (node.identity) {
      this.append("identity");
    }
    if (node.expression) {
      this.append("(");
      this.visitNode(node.expression);
      this.append(")");
    }
    if (node.stored) {
      this.append(" stored");
    }
  }
  visitDefaultValue(node) {
    this.append("default ");
    this.visitNode(node.defaultValue);
  }
  visitSelectModifier(node) {
    if (node.rawModifier) {
      this.visitNode(node.rawModifier);
    } else {
      this.append(SELECT_MODIFIER_SQL[node.modifier]);
    }
    if (node.of) {
      this.append(" of ");
      this.compileList(node.of, ", ");
    }
  }
  visitCreateType(node) {
    this.append("create type ");
    this.visitNode(node.name);
    if (node.enum) {
      this.append(" as enum ");
      this.visitNode(node.enum);
    }
  }
  visitDropType(node) {
    this.append("drop type ");
    if (node.ifExists) {
      this.append("if exists ");
    }
    this.visitNode(node.name);
  }
  visitExplain(node) {
    this.append("explain");
    if (node.options || node.format) {
      this.append(" ");
      this.append(this.getLeftExplainOptionsWrapper());
      if (node.options) {
        this.visitNode(node.options);
        if (node.format) {
          this.append(this.getExplainOptionsDelimiter());
        }
      }
      if (node.format) {
        this.append("format");
        this.append(this.getExplainOptionAssignment());
        this.append(node.format);
      }
      this.append(this.getRightExplainOptionsWrapper());
    }
  }
  visitDefaultInsertValue(_) {
    this.append("default");
  }
  visitAggregateFunction(node) {
    this.append(node.func);
    this.append("(");
    if (node.distinct) {
      this.append("distinct ");
    }
    this.compileList(node.aggregated);
    this.append(")");
    if (node.filter) {
      this.append(" filter(");
      this.visitNode(node.filter);
      this.append(")");
    }
    if (node.over) {
      this.append(" ");
      this.visitNode(node.over);
    }
  }
  visitOver(node) {
    this.append("over(");
    if (node.partitionBy) {
      this.visitNode(node.partitionBy);
      if (node.orderBy) {
        this.append(" ");
      }
    }
    if (node.orderBy) {
      this.visitNode(node.orderBy);
    }
    this.append(")");
  }
  visitPartitionBy(node) {
    this.append("partition by ");
    this.compileList(node.items);
  }
  visitPartitionByItem(node) {
    this.visitNode(node.partitionBy);
  }
  visitBinaryOperation(node) {
    this.visitNode(node.leftOperand);
    this.append(" ");
    this.visitNode(node.operator);
    this.append(" ");
    this.visitNode(node.rightOperand);
  }
  visitUnaryOperation(node) {
    this.visitNode(node.operator);
    if (!this.isMinusOperator(node.operator)) {
      this.append(" ");
    }
    this.visitNode(node.operand);
  }
  isMinusOperator(node) {
    return OperatorNode.is(node) && node.operator === "-";
  }
  visitUsing(node) {
    this.append("using ");
    this.compileList(node.tables);
  }
  visitFunction(node) {
    this.append(node.func);
    this.append("(");
    this.compileList(node.arguments);
    this.append(")");
  }
  visitCase(node) {
    this.append("case");
    if (node.value) {
      this.append(" ");
      this.visitNode(node.value);
    }
    if (node.when) {
      this.append(" ");
      this.compileList(node.when, " ");
    }
    if (node.else) {
      this.append(" else ");
      this.visitNode(node.else);
    }
    this.append(" end");
    if (node.isStatement) {
      this.append(" case");
    }
  }
  visitWhen(node) {
    this.append("when ");
    this.visitNode(node.condition);
    if (node.result) {
      this.append(" then ");
      this.visitNode(node.result);
    }
  }
  visitJSONReference(node) {
    this.visitNode(node.reference);
    this.visitNode(node.traversal);
  }
  visitJSONPath(node) {
    if (node.inOperator) {
      this.visitNode(node.inOperator);
    }
    this.append("'$");
    for (const pathLeg of node.pathLegs) {
      this.visitNode(pathLeg);
    }
    this.append("'");
  }
  visitJSONPathLeg(node) {
    const isArrayLocation = node.type === "ArrayLocation";
    this.append(isArrayLocation ? "[" : ".");
    this.append(String(node.value));
    if (isArrayLocation) {
      this.append("]");
    }
  }
  visitJSONOperatorChain(node) {
    for (let i = 0, len = node.values.length;i < len; i++) {
      if (i === len - 1) {
        this.visitNode(node.operator);
      } else {
        this.append("->");
      }
      this.visitNode(node.values[i]);
    }
  }
  visitMergeQuery(node) {
    if (node.with) {
      this.visitNode(node.with);
      this.append(" ");
    }
    this.append("merge ");
    if (node.top) {
      this.visitNode(node.top);
      this.append(" ");
    }
    this.append("into ");
    this.visitNode(node.into);
    if (node.using) {
      this.append(" ");
      this.visitNode(node.using);
    }
    if (node.whens) {
      this.append(" ");
      this.compileList(node.whens, " ");
    }
    if (node.output) {
      this.append(" ");
      this.visitNode(node.output);
    }
  }
  visitMatched(node) {
    if (node.not) {
      this.append("not ");
    }
    this.append("matched");
    if (node.bySource) {
      this.append(" by source");
    }
  }
  visitAddIndex(node) {
    this.append("add ");
    if (node.unique) {
      this.append("unique ");
    }
    this.append("index ");
    this.visitNode(node.name);
    if (node.columns) {
      this.append(" (");
      this.compileList(node.columns);
      this.append(")");
    }
    if (node.using) {
      this.append(" using ");
      this.visitNode(node.using);
    }
  }
  visitCast(node) {
    this.append("cast(");
    this.visitNode(node.expression);
    this.append(" as ");
    this.visitNode(node.dataType);
    this.append(")");
  }
  visitFetch(node) {
    this.append("fetch next ");
    this.visitNode(node.rowCount);
    this.append(` rows ${node.modifier}`);
  }
  visitOutput(node) {
    this.append("output ");
    this.compileList(node.selections);
  }
  visitTop(node) {
    this.append(`top(${node.expression})`);
    if (node.modifiers) {
      this.append(` ${node.modifiers}`);
    }
  }
  append(str) {
    this.#sql += str;
  }
  appendValue(parameter) {
    this.addParameter(parameter);
    this.append(this.getCurrentParameterPlaceholder());
  }
  getLeftIdentifierWrapper() {
    return '"';
  }
  getRightIdentifierWrapper() {
    return '"';
  }
  getCurrentParameterPlaceholder() {
    return "$" + this.numParameters;
  }
  getLeftExplainOptionsWrapper() {
    return "(";
  }
  getExplainOptionAssignment() {
    return " ";
  }
  getExplainOptionsDelimiter() {
    return ", ";
  }
  getRightExplainOptionsWrapper() {
    return ")";
  }
  sanitizeIdentifier(identifier) {
    const leftWrap = this.getLeftIdentifierWrapper();
    const rightWrap = this.getRightIdentifierWrapper();
    let sanitized = "";
    for (const c of identifier) {
      sanitized += c;
      if (c === leftWrap) {
        sanitized += leftWrap;
      } else if (c === rightWrap) {
        sanitized += rightWrap;
      }
    }
    return sanitized;
  }
  addParameter(parameter) {
    this.#parameters.push(parameter);
  }
  appendImmediateValue(value) {
    if (isString(value)) {
      this.append(`'${value}'`);
    } else if (isNumber(value) || isBoolean(value)) {
      this.append(value.toString());
    } else if (isNull(value)) {
      this.append("null");
    } else if (isDate(value)) {
      this.appendImmediateValue(value.toISOString());
    } else if (isBigInt(value)) {
      this.appendImmediateValue(value.toString());
    } else {
      throw new Error(`invalid immediate value ${value}`);
    }
  }
  sortSelectModifiers(arr) {
    arr.sort((left, right) => left.modifier && right.modifier ? SELECT_MODIFIER_PRIORITY[left.modifier] - SELECT_MODIFIER_PRIORITY[right.modifier] : 1);
    return freeze(arr);
  }
  compileColumnAlterations(columnAlterations) {
    this.compileList(columnAlterations);
  }
  announcesNewColumnDataType() {
    return true;
  }
}
var SELECT_MODIFIER_SQL = freeze({
  ForKeyShare: "for key share",
  ForNoKeyUpdate: "for no key update",
  ForUpdate: "for update",
  ForShare: "for share",
  NoWait: "nowait",
  SkipLocked: "skip locked",
  Distinct: "distinct"
});
var SELECT_MODIFIER_PRIORITY = freeze({
  ForKeyShare: 1,
  ForNoKeyUpdate: 1,
  ForUpdate: 1,
  ForShare: 1,
  NoWait: 2,
  SkipLocked: 2,
  Distinct: 0
});
var JOIN_TYPE_SQL = freeze({
  InnerJoin: "inner join",
  LeftJoin: "left join",
  RightJoin: "right join",
  FullJoin: "full join",
  LateralInnerJoin: "inner join lateral",
  LateralLeftJoin: "left join lateral",
  Using: "using"
});

// node_modules/kysely/dist/esm/query-compiler/compiled-query.js
var CompiledQuery = freeze({
  raw(sql2, parameters = []) {
    return freeze({
      sql: sql2,
      query: RawNode.createWithSql(sql2),
      parameters: freeze(parameters)
    });
  }
});

// node_modules/kysely/dist/esm/dialect/dialect-adapter-base.js
class DialectAdapterBase {
  get supportsCreateIfNotExists() {
    return true;
  }
  get supportsTransactionalDdl() {
    return false;
  }
  get supportsReturning() {
    return false;
  }
  get supportsOutput() {
    return false;
  }
}

// node_modules/kysely/dist/esm/dialect/sqlite/sqlite-driver.js
class SqliteDriver {
  #config;
  #connectionMutex = new ConnectionMutex;
  #db;
  #connection;
  constructor(config) {
    this.#config = freeze({ ...config });
  }
  async init() {
    this.#db = isFunction(this.#config.database) ? await this.#config.database() : this.#config.database;
    this.#connection = new SqliteConnection(this.#db);
    if (this.#config.onCreateConnection) {
      await this.#config.onCreateConnection(this.#connection);
    }
  }
  async acquireConnection() {
    await this.#connectionMutex.lock();
    return this.#connection;
  }
  async beginTransaction(connection) {
    await connection.executeQuery(CompiledQuery.raw("begin"));
  }
  async commitTransaction(connection) {
    await connection.executeQuery(CompiledQuery.raw("commit"));
  }
  async rollbackTransaction(connection) {
    await connection.executeQuery(CompiledQuery.raw("rollback"));
  }
  async releaseConnection() {
    this.#connectionMutex.unlock();
  }
  async destroy() {
    this.#db?.close();
  }
}

class SqliteConnection {
  #db;
  constructor(db) {
    this.#db = db;
  }
  executeQuery(compiledQuery) {
    const { sql: sql2, parameters } = compiledQuery;
    const stmt = this.#db.prepare(sql2);
    if (stmt.reader) {
      return Promise.resolve({
        rows: stmt.all(parameters)
      });
    } else {
      const { changes, lastInsertRowid } = stmt.run(parameters);
      const numAffectedRows = changes !== undefined && changes !== null ? BigInt(changes) : undefined;
      return Promise.resolve({
        numUpdatedOrDeletedRows: numAffectedRows,
        numAffectedRows,
        insertId: lastInsertRowid !== undefined && lastInsertRowid !== null ? BigInt(lastInsertRowid) : undefined,
        rows: []
      });
    }
  }
  async* streamQuery(compiledQuery, _chunkSize) {
    const { sql: sql2, parameters, query } = compiledQuery;
    const stmt = this.#db.prepare(sql2);
    if (SelectQueryNode.is(query)) {
      const iter = stmt.iterate(parameters);
      for (const row of iter) {
        yield {
          rows: [row]
        };
      }
    } else {
      throw new Error("Sqlite driver only supports streaming of select queries");
    }
  }
}

class ConnectionMutex {
  #promise;
  #resolve;
  async lock() {
    while (this.#promise) {
      await this.#promise;
    }
    this.#promise = new Promise((resolve) => {
      this.#resolve = resolve;
    });
  }
  unlock() {
    const resolve = this.#resolve;
    this.#promise = undefined;
    this.#resolve = undefined;
    resolve?.();
  }
}

// node_modules/kysely/dist/esm/dialect/sqlite/sqlite-query-compiler.js
var ID_WRAP_REGEX = /"/g;

class SqliteQueryCompiler extends DefaultQueryCompiler {
  getCurrentParameterPlaceholder() {
    return "?";
  }
  getLeftExplainOptionsWrapper() {
    return "";
  }
  getRightExplainOptionsWrapper() {
    return "";
  }
  getLeftIdentifierWrapper() {
    return '"';
  }
  getRightIdentifierWrapper() {
    return '"';
  }
  getAutoIncrement() {
    return "autoincrement";
  }
  sanitizeIdentifier(identifier) {
    return identifier.replace(ID_WRAP_REGEX, '""');
  }
  visitDefaultInsertValue(_) {
    this.append("null");
  }
}

// node_modules/kysely/dist/esm/plugin/noop-plugin.js
class NoopPlugin {
  transformQuery(args) {
    return args.node;
  }
  async transformResult(args) {
    return args.result;
  }
}

// node_modules/kysely/dist/esm/migration/migrator.js
var DEFAULT_MIGRATION_TABLE = "kysely_migration";
var DEFAULT_MIGRATION_LOCK_TABLE = "kysely_migration_lock";
var DEFAULT_ALLOW_UNORDERED_MIGRATIONS = false;
var MIGRATION_LOCK_ID = "migration_lock";
var NO_MIGRATIONS = freeze({ __noMigrations__: true });

class Migrator {
  #props;
  constructor(props) {
    this.#props = freeze(props);
  }
  async getMigrations() {
    const executedMigrations = await this.#doesTableExists(this.#migrationTable) ? await this.#props.db.withPlugin(this.#schemaPlugin).selectFrom(this.#migrationTable).select(["name", "timestamp"]).execute() : [];
    const migrations = await this.#resolveMigrations();
    return migrations.map(({ name, ...migration }) => {
      const executed = executedMigrations.find((it) => it.name === name);
      return {
        name,
        migration,
        executedAt: executed ? new Date(executed.timestamp) : undefined
      };
    });
  }
  async migrateToLatest() {
    return this.#migrate(() => ({ direction: "Up", step: Infinity }));
  }
  async migrateTo(targetMigrationName) {
    return this.#migrate(({ migrations, executedMigrations, pendingMigrations }) => {
      if (targetMigrationName === NO_MIGRATIONS) {
        return { direction: "Down", step: Infinity };
      }
      if (!migrations.find((m) => m.name === targetMigrationName)) {
        throw new Error(`migration "${targetMigrationName}" doesn't exist`);
      }
      const executedIndex = executedMigrations.indexOf(targetMigrationName);
      const pendingIndex = pendingMigrations.findIndex((m) => m.name === targetMigrationName);
      if (executedIndex !== -1) {
        return {
          direction: "Down",
          step: executedMigrations.length - executedIndex - 1
        };
      } else if (pendingIndex !== -1) {
        return { direction: "Up", step: pendingIndex + 1 };
      } else {
        throw new Error(`migration "${targetMigrationName}" isn't executed or pending`);
      }
    });
  }
  async migrateUp() {
    return this.#migrate(() => ({ direction: "Up", step: 1 }));
  }
  async migrateDown() {
    return this.#migrate(() => ({ direction: "Down", step: 1 }));
  }
  async#migrate(getMigrationDirectionAndStep) {
    try {
      await this.#ensureMigrationTablesExists();
      return await this.#runMigrations(getMigrationDirectionAndStep);
    } catch (error) {
      if (error instanceof MigrationResultSetError) {
        return error.resultSet;
      }
      return { error };
    }
  }
  get #migrationTableSchema() {
    return this.#props.migrationTableSchema;
  }
  get #migrationTable() {
    return this.#props.migrationTableName ?? DEFAULT_MIGRATION_TABLE;
  }
  get #migrationLockTable() {
    return this.#props.migrationLockTableName ?? DEFAULT_MIGRATION_LOCK_TABLE;
  }
  get #allowUnorderedMigrations() {
    return this.#props.allowUnorderedMigrations ?? DEFAULT_ALLOW_UNORDERED_MIGRATIONS;
  }
  get #schemaPlugin() {
    if (this.#migrationTableSchema) {
      return new WithSchemaPlugin(this.#migrationTableSchema);
    }
    return new NoopPlugin;
  }
  async#ensureMigrationTablesExists() {
    await this.#ensureMigrationTableSchemaExists();
    await this.#ensureMigrationTableExists();
    await this.#ensureMigrationLockTableExists();
    await this.#ensureLockRowExists();
  }
  async#ensureMigrationTableSchemaExists() {
    if (!this.#migrationTableSchema) {
      return;
    }
    if (!await this.#doesSchemaExists()) {
      try {
        await this.#createIfNotExists(this.#props.db.schema.createSchema(this.#migrationTableSchema));
      } catch (error) {
        if (!await this.#doesSchemaExists()) {
          throw error;
        }
      }
    }
  }
  async#ensureMigrationTableExists() {
    if (!await this.#doesTableExists(this.#migrationTable)) {
      try {
        if (this.#migrationTableSchema) {
          await this.#createIfNotExists(this.#props.db.schema.createSchema(this.#migrationTableSchema));
        }
        await this.#createIfNotExists(this.#props.db.schema.withPlugin(this.#schemaPlugin).createTable(this.#migrationTable).addColumn("name", "varchar(255)", (col) => col.notNull().primaryKey()).addColumn("timestamp", "varchar(255)", (col) => col.notNull()));
      } catch (error) {
        if (!await this.#doesTableExists(this.#migrationTable)) {
          throw error;
        }
      }
    }
  }
  async#ensureMigrationLockTableExists() {
    if (!await this.#doesTableExists(this.#migrationLockTable)) {
      try {
        await this.#createIfNotExists(this.#props.db.schema.withPlugin(this.#schemaPlugin).createTable(this.#migrationLockTable).addColumn("id", "varchar(255)", (col) => col.notNull().primaryKey()).addColumn("is_locked", "integer", (col) => col.notNull().defaultTo(0)));
      } catch (error) {
        if (!await this.#doesTableExists(this.#migrationLockTable)) {
          throw error;
        }
      }
    }
  }
  async#ensureLockRowExists() {
    if (!await this.#doesLockRowExists()) {
      try {
        await this.#props.db.withPlugin(this.#schemaPlugin).insertInto(this.#migrationLockTable).values({ id: MIGRATION_LOCK_ID, is_locked: 0 }).execute();
      } catch (error) {
        if (!await this.#doesLockRowExists()) {
          throw error;
        }
      }
    }
  }
  async#doesSchemaExists() {
    const schemas = await this.#props.db.introspection.getSchemas();
    return schemas.some((it) => it.name === this.#migrationTableSchema);
  }
  async#doesTableExists(tableName) {
    const schema = this.#migrationTableSchema;
    const tables = await this.#props.db.introspection.getTables({
      withInternalKyselyTables: true
    });
    return tables.some((it) => it.name === tableName && (!schema || it.schema === schema));
  }
  async#doesLockRowExists() {
    const lockRow = await this.#props.db.withPlugin(this.#schemaPlugin).selectFrom(this.#migrationLockTable).where("id", "=", MIGRATION_LOCK_ID).select("id").executeTakeFirst();
    return !!lockRow;
  }
  async#runMigrations(getMigrationDirectionAndStep) {
    const adapter = this.#props.db.getExecutor().adapter;
    const lockOptions = freeze({
      lockTable: this.#props.migrationLockTableName ?? DEFAULT_MIGRATION_LOCK_TABLE,
      lockRowId: MIGRATION_LOCK_ID,
      lockTableSchema: this.#props.migrationTableSchema
    });
    const run = async (db) => {
      try {
        await adapter.acquireMigrationLock(db, lockOptions);
        const state = await this.#getState(db);
        if (state.migrations.length === 0) {
          return { results: [] };
        }
        const { direction, step } = getMigrationDirectionAndStep(state);
        if (step <= 0) {
          return { results: [] };
        }
        if (direction === "Down") {
          return await this.#migrateDown(db, state, step);
        } else if (direction === "Up") {
          return await this.#migrateUp(db, state, step);
        }
        return { results: [] };
      } finally {
        await adapter.releaseMigrationLock(db, lockOptions);
      }
    };
    if (adapter.supportsTransactionalDdl) {
      return this.#props.db.transaction().execute(run);
    } else {
      return this.#props.db.connection().execute(run);
    }
  }
  async#getState(db) {
    const migrations = await this.#resolveMigrations();
    const executedMigrations = await this.#getExecutedMigrations(db);
    this.#ensureNoMissingMigrations(migrations, executedMigrations);
    if (!this.#allowUnorderedMigrations) {
      this.#ensureMigrationsInOrder(migrations, executedMigrations);
    }
    const pendingMigrations = this.#getPendingMigrations(migrations, executedMigrations);
    return freeze({
      migrations,
      executedMigrations,
      lastMigration: getLast(executedMigrations),
      pendingMigrations
    });
  }
  #getPendingMigrations(migrations, executedMigrations) {
    return migrations.filter((migration) => {
      return !executedMigrations.includes(migration.name);
    });
  }
  async#resolveMigrations() {
    const allMigrations = await this.#props.provider.getMigrations();
    return Object.keys(allMigrations).sort().map((name) => ({
      ...allMigrations[name],
      name
    }));
  }
  async#getExecutedMigrations(db) {
    const executedMigrations = await db.withPlugin(this.#schemaPlugin).selectFrom(this.#migrationTable).select("name").orderBy(["timestamp", "name"]).execute();
    return executedMigrations.map((it) => it.name);
  }
  #ensureNoMissingMigrations(migrations, executedMigrations) {
    for (const executed of executedMigrations) {
      if (!migrations.some((it) => it.name === executed)) {
        throw new Error(`corrupted migrations: previously executed migration ${executed} is missing`);
      }
    }
  }
  #ensureMigrationsInOrder(migrations, executedMigrations) {
    for (let i = 0;i < executedMigrations.length; ++i) {
      if (migrations[i].name !== executedMigrations[i]) {
        throw new Error(`corrupted migrations: expected previously executed migration ${executedMigrations[i]} to be at index ${i} but ${migrations[i].name} was found in its place. New migrations must always have a name that comes alphabetically after the last executed migration.`);
      }
    }
  }
  async#migrateDown(db, state, step) {
    const migrationsToRollback = state.executedMigrations.slice().reverse().slice(0, step).map((name) => {
      return state.migrations.find((it) => it.name === name);
    });
    const results = migrationsToRollback.map((migration) => {
      return {
        migrationName: migration.name,
        direction: "Down",
        status: "NotExecuted"
      };
    });
    for (let i = 0;i < results.length; ++i) {
      const migration = migrationsToRollback[i];
      try {
        if (migration.down) {
          await migration.down(db);
          await db.withPlugin(this.#schemaPlugin).deleteFrom(this.#migrationTable).where("name", "=", migration.name).execute();
          results[i] = {
            migrationName: migration.name,
            direction: "Down",
            status: "Success"
          };
        }
      } catch (error) {
        results[i] = {
          migrationName: migration.name,
          direction: "Down",
          status: "Error"
        };
        throw new MigrationResultSetError({
          error,
          results
        });
      }
    }
    return { results };
  }
  async#migrateUp(db, state, step) {
    const migrationsToRun = state.pendingMigrations.slice(0, step);
    const results = migrationsToRun.map((migration) => {
      return {
        migrationName: migration.name,
        direction: "Up",
        status: "NotExecuted"
      };
    });
    for (let i = 0;i < results.length; i++) {
      const migration = state.pendingMigrations[i];
      try {
        await migration.up(db);
        await db.withPlugin(this.#schemaPlugin).insertInto(this.#migrationTable).values({
          name: migration.name,
          timestamp: new Date().toISOString()
        }).execute();
        results[i] = {
          migrationName: migration.name,
          direction: "Up",
          status: "Success"
        };
      } catch (error) {
        results[i] = {
          migrationName: migration.name,
          direction: "Up",
          status: "Error"
        };
        throw new MigrationResultSetError({
          error,
          results
        });
      }
    }
    return { results };
  }
  async#createIfNotExists(qb) {
    if (this.#props.db.getExecutor().adapter.supportsCreateIfNotExists) {
      qb = qb.ifNotExists();
    }
    await qb.execute();
  }
}

class MigrationResultSetError extends Error {
  #resultSet;
  constructor(result) {
    super();
    this.#resultSet = result;
  }
  get resultSet() {
    return this.#resultSet;
  }
}

// node_modules/kysely/dist/esm/dialect/sqlite/sqlite-introspector.js
class SqliteIntrospector {
  #db;
  constructor(db) {
    this.#db = db;
  }
  async getSchemas() {
    return [];
  }
  async getTables(options = { withInternalKyselyTables: false }) {
    let query = this.#db.selectFrom("sqlite_master").where("type", "in", ["table", "view"]).where("name", "not like", "sqlite_%").select("name").orderBy("name").$castTo();
    if (!options.withInternalKyselyTables) {
      query = query.where("name", "!=", DEFAULT_MIGRATION_TABLE).where("name", "!=", DEFAULT_MIGRATION_LOCK_TABLE);
    }
    const tables = await query.execute();
    return Promise.all(tables.map(({ name }) => this.#getTableMetadata(name)));
  }
  async getMetadata(options) {
    return {
      tables: await this.getTables(options)
    };
  }
  async#getTableMetadata(table) {
    const db = this.#db;
    const tableDefinition = await db.selectFrom("sqlite_master").where("name", "=", table).select(["sql", "type"]).$castTo().executeTakeFirstOrThrow();
    const autoIncrementCol = tableDefinition.sql?.split(/[\(\),]/)?.find((it) => it.toLowerCase().includes("autoincrement"))?.trimStart()?.split(/\s+/)?.[0]?.replace(/["`]/g, "");
    const columns = await db.selectFrom(sql`pragma_table_info(${table})`.as("table_info")).select(["name", "type", "notnull", "dflt_value"]).orderBy("cid").execute();
    return {
      name: table,
      isView: tableDefinition.type === "view",
      columns: columns.map((col) => ({
        name: col.name,
        dataType: col.type,
        isNullable: !col.notnull,
        isAutoIncrementing: col.name === autoIncrementCol,
        hasDefaultValue: col.dflt_value != null,
        comment: undefined
      }))
    };
  }
}

// node_modules/kysely/dist/esm/dialect/sqlite/sqlite-adapter.js
class SqliteAdapter extends DialectAdapterBase {
  get supportsTransactionalDdl() {
    return false;
  }
  get supportsReturning() {
    return true;
  }
  async acquireMigrationLock(_db, _opt) {}
  async releaseMigrationLock(_db, _opt) {}
}

// node_modules/kysely/dist/esm/dialect/sqlite/sqlite-dialect.js
class SqliteDialect {
  #config;
  constructor(config) {
    this.#config = freeze({ ...config });
  }
  createDriver() {
    return new SqliteDriver(this.#config);
  }
  createQueryCompiler() {
    return new SqliteQueryCompiler;
  }
  createAdapter() {
    return new SqliteAdapter;
  }
  createIntrospector(db) {
    return new SqliteIntrospector(db);
  }
}

// query-default.mjs
var dialect = new SqliteDialect({
  database: {
    prepare: () => {
      throw new Error("client never executes");
    },
    close: () => {}
  }
});
var db = new Kysely({ dialect });
var query = db.selectFrom("notes").selectAll().where("priority", ">=", 3).where("pinned", "=", 1).orderBy("priority", "desc").limit(10);

// __entry.mjs
globalThis.__queryAst = JSON.stringify(query.toOperationNode());
