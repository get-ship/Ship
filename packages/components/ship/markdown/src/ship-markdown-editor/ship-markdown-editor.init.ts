/**
 * TipTap initialization for <ship-markdown-editor>.
 *
 * Dynamic-imports every TipTap subpath so the editor engine lazy-loads on
 * first mount rather than at registration time. Returns a configured Editor
 * instance; lifecycle and event plumbing remain in the controller.
 */

import type { Editor } from '@tiptap/core'

export type MarkdownEditor = Editor & { storage: { markdown?: { getMarkdown(): string } } }

export interface EditorInitOptions {
  element: HTMLElement
  content: string
  placeholder: string
  editable: boolean
  onUpdate: (ed: MarkdownEditor) => void
  onSelectionUpdate: () => void
  onTransaction: () => void
  onBlur: () => void
}

export async function createMarkdownEditor(opts: EditorInitOptions): Promise<MarkdownEditor> {
  const [core, starter, Placeholder, tableSuite, Link, Image, TaskList,
    TaskItem, Typography, CodeBlockLowlight, markdownMod, lowlightMod] = await Promise.all([
    import('@tiptap/core'),
    import('@tiptap/starter-kit').then(m => m.default ?? m),
    import('@tiptap/extension-placeholder').then(m => m.default ?? m),
    import('@tiptap/extension-table'),
    import('@tiptap/extension-link').then(m => m.default ?? m),
    import('@tiptap/extension-image').then(m => m.default ?? m),
    import('@tiptap/extension-task-list').then(m => m.default ?? m),
    import('@tiptap/extension-task-item').then(m => m.default ?? m),
    import('@tiptap/extension-typography').then(m => m.default ?? m),
    import('@tiptap/extension-code-block-lowlight').then(m => m.default ?? m),
    import('tiptap-markdown'),
    import('lowlight'),
  ])
  const lowlight = lowlightMod.createLowlight(lowlightMod.common)
  const { Table, TableRow, TableCell, TableHeader } = tableSuite as {
    Table: unknown; TableRow: unknown; TableCell: unknown; TableHeader: unknown
  }
  const { Markdown } = markdownMod
  const Starter = starter as { configure: (o: unknown) => unknown }
  const PH = Placeholder as { configure: (o: unknown) => unknown }
  const LinkExt = Link as { configure: (o: unknown) => unknown }
  const TaskItemExt = TaskItem as { configure: (o: unknown) => unknown }
  const TableExt = Table as { configure: (o: unknown) => unknown }
  const CodeBlock = CodeBlockLowlight as { configure: (o: unknown) => unknown }

  return new core.Editor({
    element: opts.element,
    extensions: [
      Starter.configure({ codeBlock: false, link: false }),
      PH.configure({ placeholder: opts.placeholder ?? '' }),
      TableExt.configure({ resizable: false }),
      TableRow, TableCell, TableHeader,
      LinkExt.configure({ openOnClick: false, HTMLAttributes: { rel: 'noopener noreferrer' } }),
      Image, TaskList,
      TaskItemExt.configure({ nested: true }),
      Typography,
      CodeBlock.configure({ lowlight }),
      Markdown.configure({
        html: false, transformPastedText: true, transformCopiedText: true, linkify: false,
      }),
    ] as never,
    content: opts.content,
    editable: opts.editable,
    editorProps: {
      attributes: { spellcheck: 'false', role: 'textbox', 'aria-multiline': 'true' },
    },
    onUpdate: ({ editor: ed }) => opts.onUpdate(ed as MarkdownEditor),
    onSelectionUpdate: opts.onSelectionUpdate,
    onTransaction: opts.onTransaction,
    onBlur: opts.onBlur,
  }) as MarkdownEditor
}

export function readMarkdown(ed: MarkdownEditor): string {
  const s = ed.storage?.markdown
  return s && typeof s.getMarkdown === 'function' ? s.getMarkdown() : ''
}
