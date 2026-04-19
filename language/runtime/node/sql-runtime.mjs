import { mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";

class SqlRuntimeError extends Error {
  constructor(kind, message) {
    super(String(message ?? ""));
    this.name = "SqlRuntimeError";
    this.sigilSqlKind = String(kind ?? "InvalidQuery");
  }
}

let sqliteModulePromise = null;
let postgresModulePromise = null;

async function loadSqliteModule() {
  if (!sqliteModulePromise) {
    const originalEmitWarning = process.emitWarning.bind(process);
    process.emitWarning = (warning, type, ...rest) => {
      const warningName =
        typeof type === "string"
          ? type
          : warning && typeof warning === "object" && "name" in warning
            ? String(warning.name)
            : "";
      if (warningName === "ExperimentalWarning") {
        return;
      }
      return originalEmitWarning(warning, type, ...rest);
    };
    sqliteModulePromise = import("node:sqlite").finally(() => {
      process.emitWarning = originalEmitWarning;
    });
  }
  return sqliteModulePromise;
}

async function loadPostgresModule() {
  if (!postgresModulePromise) {
    postgresModulePromise = import("pg");
  }
  return postgresModulePromise;
}

function sqlRuntimeError(kind, message) {
  return new SqlRuntimeError(kind, message);
}

function sqlErrorKind(error) {
  if (error instanceof SqlRuntimeError) {
    return String(error.sigilSqlKind ?? "InvalidQuery");
  }
  const code = String(error?.code ?? "");
  if (code.startsWith("08")) return "Connection";
  if (code.startsWith("23") || code.startsWith("SQLITE_CONSTRAINT")) {
    return "Constraint";
  }
  if (code.startsWith("0A")) return "Unsupported";
  if (code.startsWith("42") || code.startsWith("SQLITE_ERROR")) {
    return "InvalidQuery";
  }
  return "InvalidQuery";
}

function sqlErrorMessage(error) {
  return error instanceof Error ? String(error.message ?? error) : String(error);
}

function quotedIdentifier(name) {
  const text = String(name ?? "");
  if (!text) {
    throw sqlRuntimeError("InvalidQuery", "SQL identifier must be non-empty");
  }
  return `"${text.replaceAll(`"`, `""`)}"`;
}

function some(value) {
  return { __tag: "Some", __fields: [value] };
}

function none() {
  return { __tag: "None", __fields: [] };
}

function sigilMapEntries(value) {
  if (value && typeof value === "object" && Array.isArray(value.__sigil_map)) {
    return value.__sigil_map;
  }
  return null;
}

function sigilMapFromEntries(entries) {
  return { __sigil_map: Array.from(entries ?? []) };
}

function sqlValueNull() {
  return { __tag: "NullValue", __fields: [] };
}

function sqlValueBool(value) {
  return { __tag: "BoolValue", __fields: [Boolean(value)] };
}

function sqlValueInt(value) {
  return { __tag: "IntValue", __fields: [Math.trunc(Number(value ?? 0))] };
}

function sqlValueFloat(value) {
  return { __tag: "FloatValue", __fields: [Number(value ?? 0)] };
}

function sqlValueText(value) {
  return { __tag: "TextValue", __fields: [String(value ?? "")] };
}

function sqlValueBytes(buffer) {
  return {
    __tag: "BytesValue",
    __fields: [
      {
        base64: Buffer.from(buffer ?? []).toString("base64"),
      },
    ],
  };
}

function optionValue(value) {
  return value?.__tag === "Some" ? value.__fields?.[0] : undefined;
}

function isNone(value) {
  return value?.__tag === "None";
}

function boolFromDatabase(value) {
  if (typeof value === "boolean") {
    return value;
  }
  if (typeof value === "number") {
    return value !== 0;
  }
  if (typeof value === "string") {
    const normalized = value.trim().toLowerCase();
    if (normalized === "true" || normalized === "t" || normalized === "1") {
      return true;
    }
    if (normalized === "false" || normalized === "f" || normalized === "0") {
      return false;
    }
  }
  throw sqlRuntimeError("Decode", `expected Bool value, found ${typeof value}`);
}

function intFromDatabase(value) {
  if (typeof value === "number" && Number.isInteger(value)) {
    return value;
  }
  if (typeof value === "bigint") {
    const number = Number(value);
    if (Number.isSafeInteger(number)) {
      return number;
    }
  }
  throw sqlRuntimeError("Decode", `expected Int value, found ${typeof value}`);
}

function floatFromDatabase(value) {
  if (typeof value === "number") {
    return value;
  }
  if (typeof value === "bigint") {
    return Number(value);
  }
  throw sqlRuntimeError("Decode", `expected Float value, found ${typeof value}`);
}

function textFromDatabase(value) {
  if (typeof value === "string") {
    return value;
  }
  throw sqlRuntimeError("Decode", `expected Text value, found ${typeof value}`);
}

function bytesFromDatabase(value) {
  if (Buffer.isBuffer(value) || value instanceof Uint8Array) {
    return { base64: Buffer.from(value).toString("base64") };
  }
  throw sqlRuntimeError("Decode", `expected Bytes value, found ${typeof value}`);
}

function decodeColumnValue(column, value) {
  if (value == null) {
    if (column.nullable) {
      return none();
    }
    throw sqlRuntimeError(
      "Decode",
      `column '${String(column?.name ?? "")}' returned NULL for a non-null value`,
    );
  }

  let decoded;
  switch (String(column?.scalar ?? "")) {
    case "bool":
      decoded = boolFromDatabase(value);
      break;
    case "int":
      decoded = intFromDatabase(value);
      break;
    case "float":
      decoded = floatFromDatabase(value);
      break;
    case "text":
      decoded = textFromDatabase(value);
      break;
    case "bytes":
      decoded = bytesFromDatabase(value);
      break;
    default:
      throw sqlRuntimeError(
        "Unsupported",
        `unsupported column scalar '${String(column?.scalar ?? "")}'`,
      );
  }

  return column.nullable ? some(decoded) : decoded;
}

function encodeScalarValue(scalar, value, context) {
  switch (String(scalar ?? "")) {
    case "bool":
      if (typeof value !== "boolean") {
        throw sqlRuntimeError("InvalidQuery", `${context} expects Bool`);
      }
      return value;
    case "int":
      if (typeof value !== "number" || !Number.isInteger(value)) {
        throw sqlRuntimeError("InvalidQuery", `${context} expects Int`);
      }
      return value;
    case "float":
      if (typeof value !== "number") {
        throw sqlRuntimeError("InvalidQuery", `${context} expects Float`);
      }
      return value;
    case "text":
      if (typeof value !== "string") {
        throw sqlRuntimeError("InvalidQuery", `${context} expects Text`);
      }
      return value;
    case "bytes":
      if (
        !value ||
        typeof value !== "object" ||
        typeof value.base64 !== "string"
      ) {
        throw sqlRuntimeError("InvalidQuery", `${context} expects Bytes`);
      }
      return Buffer.from(String(value.base64), "base64");
    default:
      throw sqlRuntimeError(
        "Unsupported",
        `unsupported column scalar '${String(scalar ?? "")}'`,
      );
  }
}

function encodeColumnValue(column, value) {
  const context = `column '${String(column?.name ?? "")}'`;
  if (column?.nullable) {
    if (isNone(value)) {
      return null;
    }
    if (value?.__tag === "Some") {
      return encodeScalarValue(column.scalar, optionValue(value), context);
    }
    if (value == null) {
      return null;
    }
    return encodeScalarValue(column.scalar, value, context);
  }
  if (value?.__tag === "Some" || isNone(value) || value == null) {
    throw sqlRuntimeError("InvalidQuery", `${context} expects a non-null value`);
  }
  return encodeScalarValue(column.scalar, value, context);
}

function parseRawValue(value) {
  switch (value?.__tag) {
    case "NullValue":
      return null;
    case "BoolValue":
      return Boolean(value.__fields?.[0]);
    case "IntValue":
      return Math.trunc(Number(value.__fields?.[0] ?? 0));
    case "FloatValue":
      return Number(value.__fields?.[0] ?? 0);
    case "TextValue":
      return String(value.__fields?.[0] ?? "");
    case "BytesValue": {
      const payload = value.__fields?.[0] ?? {};
      if (typeof payload?.base64 !== "string") {
        throw sqlRuntimeError("InvalidQuery", "raw Bytes value requires base64");
      }
      return Buffer.from(String(payload.base64), "base64");
    }
    default:
      throw sqlRuntimeError(
        "InvalidQuery",
        "raw SQL parameters must use §sql.Value constructors",
      );
  }
}

function valueFromDatabase(value) {
  if (value == null) return sqlValueNull();
  if (typeof value === "boolean") return sqlValueBool(value);
  if (typeof value === "number") {
    return Number.isInteger(value) ? sqlValueInt(value) : sqlValueFloat(value);
  }
  if (typeof value === "bigint") {
    const number = Number(value);
    return Number.isSafeInteger(number)
      ? sqlValueInt(number)
      : sqlValueFloat(number);
  }
  if (typeof value === "string") return sqlValueText(value);
  if (Buffer.isBuffer(value) || value instanceof Uint8Array) {
    return sqlValueBytes(value);
  }
  throw sqlRuntimeError(
    "Decode",
    `raw SQL returned an unsupported value of type ${typeof value}`,
  );
}

function decodeRow(table, row) {
  const result = {};
  for (const column of Array.isArray(table?.columns) ? table.columns : []) {
    result[String(column.field)] = decodeColumnValue(column, row?.[column.field]);
  }
  return result;
}

function decodeRawRow(row) {
  return sigilMapFromEntries(
    Object.entries(row ?? {}).map(([key, value]) => [
      String(key),
      valueFromDatabase(value),
    ]),
  );
}

function sqliteRawValueFromDeclaredType(type, value) {
  if (value == null) {
    return sqlValueNull();
  }
  const normalized = String(type ?? "").trim().toLowerCase();
  if (
    normalized === "bool" ||
    normalized === "boolean" ||
    normalized === "bit"
  ) {
    return sqlValueBool(boolFromDatabase(value));
  }
  return valueFromDatabase(value);
}

function decodeSqliteRawRow(row, columns) {
  const declaredTypes = Object.create(null);
  for (const column of Array.isArray(columns) ? columns : []) {
    declaredTypes[String(column?.name ?? "")] = String(column?.type ?? "");
  }
  return sigilMapFromEntries(
    Object.entries(row ?? {}).map(([key, value]) => [
      String(key),
      sqliteRawValueFromDeclaredType(declaredTypes[String(key)], value),
    ]),
  );
}

function normalizeRawParams(params) {
  const result = Object.create(null);
  const sigilEntries = sigilMapEntries(params);
  if (sigilEntries) {
    for (const [key, value] of sigilEntries) {
      result[String(key)] = parseRawValue(value);
    }
    return result;
  }
  for (const [key, value] of Object.entries(params ?? {})) {
    result[String(key)] = parseRawValue(value);
  }
  return result;
}

function compileRawStatement(statement) {
  return {
    params: normalizeRawParams(statement?.params ?? {}),
    sql: String(statement?.sql ?? ""),
  };
}

function ensureColumns(columns) {
  if (!Array.isArray(columns) || columns.length === 0) {
    throw sqlRuntimeError("InvalidQuery", "SQL table requires at least one column");
  }
  return columns;
}

function makeCompileState() {
  return {
    nextParam: 1,
    params: Object.create(null),
  };
}

function allocParam(state, value) {
  const name = `p${state.nextParam}`;
  state.nextParam += 1;
  state.params[name] = value;
  return `:${name}`;
}

function compilePredicate(predicate, state) {
  if (!predicate) {
    return "";
  }
  switch (String(predicate.kind ?? "")) {
    case "and":
      return `(${compilePredicate(predicate.left, state)} and ${compilePredicate(predicate.right, state)})`;
    case "or":
      return `(${compilePredicate(predicate.left, state)} or ${compilePredicate(predicate.right, state)})`;
    case "not":
      return `(not ${compilePredicate(predicate.predicate, state)})`;
    case "eq":
    case "neq":
    case "lt":
    case "lte":
    case "gt":
    case "gte": {
      const operator = {
        eq: "=",
        neq: "!=",
        lt: "<",
        lte: "<=",
        gt: ">",
        gte: ">=",
      }[predicate.kind];
      const column = predicate.column ?? {};
      const encoded = encodeColumnValue(column, predicate.value);
      if (encoded == null) {
        if (predicate.kind === "eq") {
          return `${quotedIdentifier(column.name)} is null`;
        }
        if (predicate.kind === "neq") {
          return `${quotedIdentifier(column.name)} is not null`;
        }
        throw sqlRuntimeError(
          "InvalidQuery",
          `${predicate.kind} cannot compare NULL values`,
        );
      }
      return `${quotedIdentifier(column.name)} ${operator} ${allocParam(state, encoded)}`;
    }
    default:
      throw sqlRuntimeError(
        "Unsupported",
        `unsupported predicate kind '${String(predicate?.kind ?? "")}'`,
      );
  }
}

function compileSelect(select) {
  const table = select?.table ?? {};
  const columns = ensureColumns(table.columns);
  const state = makeCompileState();
  const projection = columns
    .map(
      (column) =>
        `${quotedIdentifier(column.name)} as ${quotedIdentifier(column.field)}`,
    )
    .join(", ");
  let sql = `select ${projection} from ${quotedIdentifier(table.name)}`;
  if (select?.predicate) {
    sql += ` where ${compilePredicate(select.predicate, state)}`;
  }
  if (select?.order?.column) {
    const direction =
      String(select.order.direction ?? "Asc") === "Desc" ? "desc" : "asc";
    sql += ` order by ${quotedIdentifier(select.order.column.name)} ${direction}`;
  }
  if (select?.limit != null) {
    const count = Math.trunc(Number(select.limit));
    if (!Number.isInteger(count) || count < 0) {
      throw sqlRuntimeError("InvalidQuery", "select limit must be a non-negative Int");
    }
    sql += ` limit ${String(count)}`;
  }
  return { params: state.params, sql, table };
}

function compileInsert(insert) {
  const table = insert?.table ?? {};
  const columns = ensureColumns(table.columns);
  const state = makeCompileState();
  const row = insert?.row ?? {};
  const names = [];
  const placeholders = [];
  for (const column of columns) {
    if (!Object.prototype.hasOwnProperty.call(row, column.field)) {
      throw sqlRuntimeError(
        "InvalidQuery",
        `insert row is missing field '${String(column.field)}'`,
      );
    }
    names.push(quotedIdentifier(column.name));
    placeholders.push(allocParam(state, encodeColumnValue(column, row[column.field])));
  }
  return {
    params: state.params,
    sql: `insert into ${quotedIdentifier(table.name)} (${names.join(", ")}) values (${placeholders.join(", ")})`,
  };
}

function compileUpdate(update) {
  const table = update?.table ?? {};
  const assignments = Array.isArray(update?.assignments) ? update.assignments : [];
  if (assignments.length === 0) {
    throw sqlRuntimeError("InvalidQuery", "update requires at least one assignment");
  }
  const state = makeCompileState();
  const setClause = assignments.map((assignment) => {
    const column = assignment?.column ?? {};
    return `${quotedIdentifier(column.name)} = ${allocParam(state, encodeColumnValue(column, assignment?.value))}`;
  });
  let sql = `update ${quotedIdentifier(table.name)} set ${setClause.join(", ")}`;
  if (update?.predicate) {
    sql += ` where ${compilePredicate(update.predicate, state)}`;
  }
  return { params: state.params, sql };
}

function compileDelete(statement) {
  const table = statement?.table ?? {};
  const state = makeCompileState();
  let sql = `delete from ${quotedIdentifier(table.name)}`;
  if (statement?.predicate) {
    sql += ` where ${compilePredicate(statement.predicate, state)}`;
  }
  return { params: state.params, sql };
}

function scanNamedParameters(sql) {
  const text = String(sql ?? "");
  const tokens = [];
  let index = 0;
  let dollarQuote = null;
  let mode = "normal";
  while (index < text.length) {
    const char = text[index];
    const next = text[index + 1] ?? "";

    if (mode === "single") {
      if (char === "'" && next === "'") {
        index += 2;
        continue;
      }
      if (char === "'") {
        mode = "normal";
      }
      index += 1;
      continue;
    }

    if (mode === "double") {
      if (char === '"') {
        mode = "normal";
      }
      index += 1;
      continue;
    }

    if (mode === "line_comment") {
      if (char === "\n") {
        mode = "normal";
      }
      index += 1;
      continue;
    }

    if (mode === "block_comment") {
      if (char === "*" && next === "/") {
        mode = "normal";
        index += 2;
        continue;
      }
      index += 1;
      continue;
    }

    if (mode === "dollar_quote") {
      if (dollarQuote && text.startsWith(dollarQuote, index)) {
        index += dollarQuote.length;
        mode = "normal";
        dollarQuote = null;
        continue;
      }
      index += 1;
      continue;
    }

    if (char === "'" ) {
      mode = "single";
      index += 1;
      continue;
    }
    if (char === '"') {
      mode = "double";
      index += 1;
      continue;
    }
    if (char === "-" && next === "-") {
      mode = "line_comment";
      index += 2;
      continue;
    }
    if (char === "/" && next === "*") {
      mode = "block_comment";
      index += 2;
      continue;
    }
    if (char === "$") {
      const match = text.slice(index).match(/^\$[A-Za-z_][A-Za-z0-9_]*\$/) ?? text.slice(index).match(/^\$\$/);
      if (match) {
        dollarQuote = match[0];
        mode = "dollar_quote";
        index += dollarQuote.length;
        continue;
      }
    }
    if (char === ":" && next !== ":" && /[A-Za-z_]/.test(next)) {
      let end = index + 2;
      while (end < text.length && /[A-Za-z0-9_]/.test(text[end])) {
        end += 1;
      }
      tokens.push({
        end,
        name: text.slice(index + 1, end),
        start: index,
      });
      index = end;
      continue;
    }
    index += 1;
  }
  return tokens;
}

function sqliteBindings(sql, params) {
  const bindings = Object.create(null);
  for (const token of scanNamedParameters(sql)) {
    if (!Object.prototype.hasOwnProperty.call(params, token.name)) {
      throw sqlRuntimeError(
        "InvalidQuery",
        `missing SQL parameter ':${String(token.name)}'`,
      );
    }
    const value = params[token.name];
    bindings[`:${token.name}`] =
      typeof value === "boolean" ? (value ? 1 : 0) : value;
  }
  return bindings;
}

function rewriteNamedParamsForPostgres(sql, params) {
  const tokens = scanNamedParameters(sql);
  const values = [];
  const indexes = new Map();
  let offset = 0;
  let rewritten = String(sql ?? "");
  for (const token of tokens) {
    if (!Object.prototype.hasOwnProperty.call(params, token.name)) {
      throw sqlRuntimeError(
        "InvalidQuery",
        `missing SQL parameter ':${String(token.name)}'`,
      );
    }
    let index = indexes.get(token.name);
    if (index == null) {
      values.push(params[token.name]);
      index = values.length;
      indexes.set(token.name, index);
    }
    const replacement = `$${String(index)}`;
    const start = token.start + offset;
    const end = token.end + offset;
    rewritten = `${rewritten.slice(0, start)}${replacement}${rewritten.slice(end)}`;
    offset += replacement.length - (token.end - token.start);
  }
  return { sql: rewritten, values };
}

async function executeRunner(runner, sql, params, operation) {
  const compiled = {
    params: params ?? Object.create(null),
    sql: String(sql ?? ""),
  };
  switch (String(operation ?? "")) {
    case "all":
      return runner.all(compiled);
    case "exec":
      return runner.exec(compiled);
    case "one":
      return runner.one(compiled);
    default:
      throw sqlRuntimeError(
        "Unsupported",
        `unsupported SQL runner operation '${String(operation ?? "")}'`,
      );
  }
}

async function ensureSqliteParent(path) {
  const absolute = resolve(String(path ?? ""));
  await mkdir(dirname(absolute), { recursive: true });
  return absolute;
}

async function executeSqlite(db, compiled) {
  const statement = db.prepare(compiled.sql);
  return statement.all(sqliteBindings(compiled.sql, compiled.params ?? {}));
}

async function executeSqliteRaw(db, compiled) {
  const statement = db.prepare(compiled.sql);
  const columns =
    typeof statement.columns === "function" ? statement.columns() : [];
  const rows = statement.all(sqliteBindings(compiled.sql, compiled.params ?? {}));
  return rows.map((row) => decodeSqliteRawRow(row, columns));
}

async function executeSqliteOne(db, compiled) {
  const rows = await executeSqlite(db, compiled);
  if (rows.length > 1) {
    throw sqlRuntimeError("InvalidQuery", "query expected at most one row");
  }
  return rows[0] ?? null;
}

async function executeSqliteRawOne(db, compiled) {
  const rows = await executeSqliteRaw(db, compiled);
  if (rows.length > 1) {
    throw sqlRuntimeError("InvalidQuery", "query expected at most one row");
  }
  return rows[0] ?? null;
}

async function execSqlite(db, compiled) {
  const statement = db.prepare(compiled.sql);
  const result = statement.run(sqliteBindings(compiled.sql, compiled.params ?? {}));
  return Number(result?.changes ?? 0);
}

async function executePostgres(poolOrClient, compiled) {
  const rewritten = rewriteNamedParamsForPostgres(compiled.sql, compiled.params ?? {});
  const result = await poolOrClient.query(rewritten.sql, rewritten.values);
  return Array.isArray(result?.rows) ? result.rows : [];
}

async function executePostgresOne(poolOrClient, compiled) {
  const rows = await executePostgres(poolOrClient, compiled);
  if (rows.length > 1) {
    throw sqlRuntimeError("InvalidQuery", "query expected at most one row");
  }
  return rows[0] ?? null;
}

async function execPostgres(poolOrClient, compiled) {
  const rewritten = rewriteNamedParamsForPostgres(compiled.sql, compiled.params ?? {});
  const result = await poolOrClient.query(rewritten.sql, rewritten.values);
  return Number(result?.rowCount ?? 0);
}

function backendRunner(backend) {
  if (backend?.kind === "sqlite") {
    return {
      all: (compiled) => executeSqlite(backend.db, compiled),
      exec: (compiled) => execSqlite(backend.db, compiled),
      one: (compiled) => executeSqliteOne(backend.db, compiled),
    };
  }
  if (backend?.kind === "postgres") {
    return {
      all: (compiled) => executePostgres(backend.pool, compiled),
      exec: (compiled) => execPostgres(backend.pool, compiled),
      one: (compiled) => executePostgresOne(backend.pool, compiled),
    };
  }
  throw sqlRuntimeError("Denied", "SQL is denied by the current world");
}

function transactionRunner(transaction) {
  if (!transaction || transaction.active !== true) {
    throw sqlRuntimeError("Transaction", "transaction is not active");
  }
  if (transaction.kind === "sqlite") {
    return {
      all: (compiled) => executeSqlite(transaction.db, compiled),
      exec: (compiled) => execSqlite(transaction.db, compiled),
      one: (compiled) => executeSqliteOne(transaction.db, compiled),
    };
  }
  if (transaction.kind === "postgres") {
    return {
      all: (compiled) => executePostgres(transaction.client, compiled),
      exec: (compiled) => execPostgres(transaction.client, compiled),
      one: (compiled) => executePostgresOne(transaction.client, compiled),
    };
  }
  throw sqlRuntimeError("Transaction", "transaction backend is invalid");
}

export async function connect(entry) {
  switch (String(entry?.kind ?? "")) {
    case "deny":
      return { kind: "deny" };
    case "fixture": {
      const { DatabaseSync } = await loadSqliteModule();
      const db = new DatabaseSync(":memory:");
      for (const statement of Array.isArray(entry?.seed) ? entry.seed : []) {
        const compiled = compileRawStatement(statement);
        await execSqlite(db, compiled);
      }
      return { db, kind: "sqlite" };
    }
    case "sqlite": {
      const { DatabaseSync } = await loadSqliteModule();
      const absolutePath = await ensureSqliteParent(entry?.path ?? "");
      return { db: new DatabaseSync(absolutePath), kind: "sqlite" };
    }
    case "postgres": {
      const { Pool } = await loadPostgresModule();
      return {
        kind: "postgres",
        pool: new Pool({
          allowExitOnIdle: true,
          connectionString: String(entry?.connection ?? ""),
          max: 1,
        }),
      };
    }
    default:
      throw sqlRuntimeError(
        "Unsupported",
        `unsupported SQL entry kind '${String(entry?.kind ?? "")}'`,
      );
  }
}

export async function all(backend, select) {
  const compiled = compileSelect(select);
  const rows = await backendRunner(backend).all(compiled);
  return rows.map((row) => decodeRow(compiled.table, row));
}

export async function one(backend, select) {
  const compiled = compileSelect(select);
  const row = await backendRunner(backend).one(compiled);
  return row == null ? none() : some(decodeRow(compiled.table, row));
}

export async function execInsert(backend, insert) {
  return backendRunner(backend).exec(compileInsert(insert));
}

export async function execUpdate(backend, update) {
  return backendRunner(backend).exec(compileUpdate(update));
}

export async function execDelete(backend, statement) {
  return backendRunner(backend).exec(compileDelete(statement));
}

export async function rawExec(backend, statement) {
  const compiled = compileRawStatement(statement);
  return backendRunner(backend).exec(compiled);
}

export async function rawQuery(backend, statement) {
  const compiled = compileRawStatement(statement);
  if (backend?.kind === "sqlite") {
    return executeSqliteRaw(backend.db, compiled);
  }
  const rows = await backendRunner(backend).all(compiled);
  return rows.map(decodeRawRow);
}

export async function rawQueryOne(backend, statement) {
  const compiled = compileRawStatement(statement);
  if (backend?.kind === "sqlite") {
    const row = await executeSqliteRawOne(backend.db, compiled);
    return row == null ? none() : some(row);
  }
  const row = await backendRunner(backend).one(compiled);
  return row == null ? none() : some(decodeRawRow(row));
}

export async function begin(backend) {
  if (backend?.kind === "sqlite") {
    backend.db.exec("begin");
    return { active: true, db: backend.db, kind: "sqlite" };
  }
  if (backend?.kind === "postgres") {
    const client = await backend.pool.connect();
    await client.query("begin");
    return { active: true, client, kind: "postgres" };
  }
  throw sqlRuntimeError("Denied", "SQL is denied by the current world");
}

export async function commit(transaction) {
  if (!transaction || transaction.active !== true) {
    throw sqlRuntimeError("Transaction", "transaction is not active");
  }
  if (transaction.kind === "sqlite") {
    transaction.db.exec("commit");
    transaction.active = false;
    return null;
  }
  if (transaction.kind === "postgres") {
    await transaction.client.query("commit");
    transaction.client.release();
    transaction.active = false;
    return null;
  }
  throw sqlRuntimeError("Transaction", "transaction backend is invalid");
}

export async function rollback(transaction) {
  if (!transaction || transaction.active !== true) {
    throw sqlRuntimeError("Transaction", "transaction is not active");
  }
  if (transaction.kind === "sqlite") {
    transaction.db.exec("rollback");
    transaction.active = false;
    return null;
  }
  if (transaction.kind === "postgres") {
    await transaction.client.query("rollback");
    transaction.client.release();
    transaction.active = false;
    return null;
  }
  throw sqlRuntimeError("Transaction", "transaction backend is invalid");
}

export async function allIn(transaction, select) {
  const compiled = compileSelect(select);
  const rows = await transactionRunner(transaction).all(compiled);
  return rows.map((row) => decodeRow(compiled.table, row));
}

export async function oneIn(transaction, select) {
  const compiled = compileSelect(select);
  const row = await transactionRunner(transaction).one(compiled);
  return row == null ? none() : some(decodeRow(compiled.table, row));
}

export async function execInsertIn(transaction, insert) {
  return transactionRunner(transaction).exec(compileInsert(insert));
}

export async function execUpdateIn(transaction, update) {
  return transactionRunner(transaction).exec(compileUpdate(update));
}

export async function execDeleteIn(transaction, statement) {
  return transactionRunner(transaction).exec(compileDelete(statement));
}

export async function rawExecIn(transaction, statement) {
  const runner = transactionRunner(transaction);
  const result = compileRawStatement(statement);
  return executeRunner(runner, result.sql, result.params, "exec");
}

export async function rawQueryIn(transaction, statement) {
  const compiled = compileRawStatement(statement);
  if (transaction?.kind === "sqlite") {
    return executeSqliteRaw(transaction.db, compiled);
  }
  const rows = await transactionRunner(transaction).all(compiled);
  return rows.map(decodeRawRow);
}

export async function rawQueryOneIn(transaction, statement) {
  const compiled = compileRawStatement(statement);
  if (transaction?.kind === "sqlite") {
    const row = await executeSqliteRawOne(transaction.db, compiled);
    return row == null ? none() : some(row);
  }
  const row = await transactionRunner(transaction).one(compiled);
  return row == null ? none() : some(decodeRawRow(row));
}

export function failureKind(error) {
  return sqlErrorKind(error);
}

export function failureMessage(error) {
  return sqlErrorMessage(error);
}
