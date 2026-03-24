import * as vscode from 'vscode'
import * as path from 'path'

interface ParseError {
  message: string
  span: { start: number; end: number }
  hint?: string | null
}

interface PortDef {
  name: string
  direction: string
  connector?: string
  attributes?: string[]
  range_start?: number
  range_end?: number
}

interface TemplateDef {
  type: 'Template'
  name: string
  ports: PortDef[]
}

interface InstanceDef {
  type: 'Instance'
  name: string
  template_name: string
}

interface Statement {
  type: string
  name?: string
  template_name?: string
  ports?: PortDef[]
}

interface DrcDiagnostic {
  severity: 'error' | 'warning' | 'info'
  layer: string
  message: string
  span?: { start: number; end: number } | null
  source?: string | null
  target?: string | null
  fix?: string | null
}

interface CheckResult {
  program: { statements: Statement[] } | null
  errors: ParseError[]
  diagnostics: DrcDiagnostic[]
}

interface WasmModule {
  parse: (source: string) => string
  check: (source: string) => string
}

let wasmModule: WasmModule | null = null

async function loadWasm(): Promise<typeof wasmModule> {
  if (wasmModule) return wasmModule
  // Load from pkg-node bundled inside the extension (../pkg-node from out/)
  const wasmPath = path.resolve(__dirname, '..', 'pkg-node', 'patchlang_wasm.js')
  try {
    wasmModule = require(wasmPath)
    return wasmModule
  } catch (err) {
    vscode.window.showErrorMessage(
      `PatchLang: Failed to load WASM parser at ${wasmPath}. Run wasm-pack build first.`
    )
    return null
  }
}

/** Convert a byte offset in source text to a VS Code Position (line, character). */
function offsetToPosition(text: string, offset: number): vscode.Position {
  let line = 0
  let col = 0
  for (let i = 0; i < offset && i < text.length; i++) {
    if (text[i] === '\n') {
      line++
      col = 0
    } else {
      col++
    }
  }
  return new vscode.Position(line, col)
}

const DRC_SEVERITY_MAP: Record<string, vscode.DiagnosticSeverity> = {
  error: vscode.DiagnosticSeverity.Error,
  warning: vscode.DiagnosticSeverity.Warning,
  info: vscode.DiagnosticSeverity.Information,
}

function parseDiagnostics(
  document: vscode.TextDocument,
  diagnosticCollection: vscode.DiagnosticCollection,
): void {
  if (!wasmModule) return

  const source = document.getText()
  let result: CheckResult

  try {
    result = JSON.parse(wasmModule.check(source))
  } catch (err) {
    diagnosticCollection.set(document.uri, [
      new vscode.Diagnostic(
        new vscode.Range(0, 0, 0, 0),
        `PatchLang parser error: ${err}`,
        vscode.DiagnosticSeverity.Error,
      ),
    ])
    return
  }

  const diagnostics: vscode.Diagnostic[] = []

  // Parse errors (syntax)
  for (const error of result.errors) {
    const start = offsetToPosition(source, error.span.start)
    const end = offsetToPosition(source, error.span.end)
    const range = new vscode.Range(start, end)
    const message = error.hint
      ? `${error.message}\n\nHint: ${error.hint}`
      : error.message
    diagnostics.push(new vscode.Diagnostic(range, message, vscode.DiagnosticSeverity.Error))
  }

  // DRC diagnostics (structural, electrical, etc.)
  for (const drc of result.diagnostics ?? []) {
    const severity = DRC_SEVERITY_MAP[drc.severity] ?? vscode.DiagnosticSeverity.Warning
    let range: vscode.Range
    if (drc.span) {
      range = new vscode.Range(
        offsetToPosition(source, drc.span.start),
        offsetToPosition(source, drc.span.end),
      )
    } else {
      range = new vscode.Range(0, 0, 0, 0)
    }
    let message = `[DRC/${drc.layer}] ${drc.message}`
    if (drc.fix) message += `\n\nFix: ${drc.fix}`
    const diag = new vscode.Diagnostic(range, message, severity)
    diag.source = 'patchlang-drc'
    diagnostics.push(diag)
  }

  diagnosticCollection.set(document.uri, diagnostics)
}

const DEBOUNCE_MS = 300
const debounceTimers = new Map<string, NodeJS.Timeout>()

function debouncedParse(
  document: vscode.TextDocument,
  diagnosticCollection: vscode.DiagnosticCollection,
): void {
  const key = document.uri.toString()
  const existing = debounceTimers.get(key)
  if (existing) clearTimeout(existing)

  debounceTimers.set(
    key,
    setTimeout(() => {
      debounceTimers.delete(key)
      parseDiagnostics(document, diagnosticCollection)
    }, DEBOUNCE_MS),
  )
}

/** Extract template and instance definitions from the current document's AST. */
function extractSymbols(document: vscode.TextDocument): {
  templates: Map<string, PortDef[]>
  instances: Map<string, string>  // instance name → template name
} {
  const templates = new Map<string, PortDef[]>()
  const instances = new Map<string, string>()

  if (!wasmModule) return { templates, instances }

  try {
    const result: CheckResult = JSON.parse(wasmModule.check(document.getText()))
    if (!result.program) return { templates, instances }

    for (const stmt of result.program.statements) {
      if (stmt.type === 'Template' && stmt.name && stmt.ports) {
        templates.set(stmt.name, stmt.ports)
      } else if (stmt.type === 'Instance' && stmt.name && stmt.template_name) {
        instances.set(stmt.name, stmt.template_name)
      }
    }
  } catch {
    // Parse failed — return empty
  }

  return { templates, instances }
}

/** Expand port ranges into individual port names. */
function expandPorts(ports: PortDef[]): string[] {
  const names: string[] = []
  for (const port of ports) {
    if (port.range_start != null && port.range_end != null) {
      for (let i = port.range_start; i <= port.range_end; i++) {
        names.push(`${port.name}[${i}]`)
      }
      // Also suggest the base name with range syntax
      names.push(`${port.name}[${port.range_start}..${port.range_end}]`)
    } else {
      names.push(port.name)
    }
  }
  return names
}

class PatchLangCompletionProvider implements vscode.CompletionItemProvider {
  provideCompletionItems(
    document: vscode.TextDocument,
    position: vscode.Position,
  ): vscode.CompletionItem[] {
    const lineText = document.lineAt(position).text
    const textBefore = lineText.substring(0, position.character)
    const { templates, instances } = extractSymbols(document)

    const items: vscode.CompletionItem[] = []

    // After "is " → suggest template names
    if (/\bis\s+\w*$/.test(textBefore)) {
      for (const name of templates.keys()) {
        const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Class)
        item.detail = 'Template'
        items.push(item)
      }
      return items
    }

    // After "InstanceName." → suggest that instance's template ports
    const dotMatch = textBefore.match(/\b([A-Za-z_][A-Za-z0-9_]*)\.(\w*)$/)
    if (dotMatch) {
      const instanceName = dotMatch[1]
      const templateName = instances.get(instanceName)
      if (templateName) {
        const ports = templates.get(templateName)
        if (ports) {
          for (const portName of expandPorts(ports)) {
            const item = new vscode.CompletionItem(portName, vscode.CompletionItemKind.Field)
            item.detail = `Port on ${templateName}`
            items.push(item)
          }
          return items
        }
      }
      // Even if no template found, suggest all known instance names (for connect/bridge targets)
    }

    // At line start or after whitespace → suggest instance names (for connect/bridge statements)
    if (/\b(connect|bridge|bridge_group)\s+\w*$/.test(textBefore) ||
        /->[\s]*\w*$/.test(textBefore)) {
      for (const name of instances.keys()) {
        const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Variable)
        item.detail = `Instance of ${instances.get(name)}`
        items.push(item)
      }
      return items
    }

    // Inside use "" → suggest .patch files in workspace
    if (/use\s+"[^"]*$/.test(textBefore)) {
      // This is handled asynchronously — return empty for now
      // A proper implementation would use resolveCompletionItem
    }

    return items
  }
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  await loadWasm()

  const diagnosticCollection = vscode.languages.createDiagnosticCollection('patchlang')
  context.subscriptions.push(diagnosticCollection)

  // Register completion provider
  const selector: vscode.DocumentSelector = { language: 'patchlang' }
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(selector, new PatchLangCompletionProvider(), '.', ' ')
  )

  // Parse all already-open PatchLang documents
  for (const document of vscode.workspace.textDocuments) {
    if (document.languageId === 'patchlang') {
      parseDiagnostics(document, diagnosticCollection)
    }
  }

  // Parse on open
  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((document) => {
      if (document.languageId === 'patchlang') {
        parseDiagnostics(document, diagnosticCollection)
      }
    }),
  )

  // Parse on edit (debounced)
  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument((event) => {
      if (event.document.languageId === 'patchlang') {
        debouncedParse(event.document, diagnosticCollection)
      }
    }),
  )

  // Clear diagnostics when document closes
  context.subscriptions.push(
    vscode.workspace.onDidCloseTextDocument((document) => {
      diagnosticCollection.delete(document.uri)
      debounceTimers.delete(document.uri.toString())
    }),
  )
}

export function deactivate(): void {
  debounceTimers.forEach((timer) => clearTimeout(timer))
  debounceTimers.clear()
}
