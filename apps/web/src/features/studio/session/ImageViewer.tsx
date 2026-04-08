// Zoomable image viewer with stepped zoom controls, pointer-drag pan, and annotation support.

import { useState, useRef, useCallback } from 'react'
import { Image as ImageIcon, ZoomIn, ZoomOut, RotateCcw } from 'lucide-react'
import { AnnotationMarker } from './AnnotationMarker'
import { AnnotationInput } from './AnnotationInput'
import type { SessionFile, ClickAnnotation } from './types'

const ZOOM_STEPS = [0.25, 0.5, 0.75, 1, 1.5, 2, 3, 4]
const DEFAULT_ZOOM = 1

interface ImageViewerProps {
  file: SessionFile
  content: string
  annotationMode?: boolean
  annotations?: ClickAnnotation[]
  activeAnnotationId?: string | null
  onAddAnnotation?: (x: number, y: number, note: string) => void
  onAnnotationClick?: (id: string) => void
  onRemoveAnnotation?: (id: string) => void
}

interface PendingAnnotation { x: number; y: number }

export function ImageViewer({
  file, content,
  annotationMode, annotations, activeAnnotationId,
  onAddAnnotation, onAnnotationClick, onRemoveAnnotation,
}: ImageViewerProps) {
  const [scale, setScale] = useState(DEFAULT_ZOOM)
  const [pan, setPan] = useState({ x: 0, y: 0 })
  const [pending, setPending] = useState<PendingAnnotation | null>(null)
  const dragging = useRef<{ startX: number; startY: number; panX: number; panY: number } | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  const zoomIn = useCallback(() => setScale((s) => ZOOM_STEPS.find((z) => z > s) ?? s), [])
  const zoomOut = useCallback(() => setScale((s) => [...ZOOM_STEPS].reverse().find((z) => z < s) ?? s), [])
  const resetZoom = useCallback(() => { setScale(DEFAULT_ZOOM); setPan({ x: 0, y: 0 }) }, [])

  const onPointerDown = useCallback((e: React.PointerEvent) => {
    if (annotationMode) return
    if (scale <= 1) return
    e.preventDefault()
    dragging.current = { startX: e.clientX, startY: e.clientY, panX: pan.x, panY: pan.y }
    ;(e.target as HTMLElement).setPointerCapture(e.pointerId)
  }, [scale, pan, annotationMode])

  const onPointerMove = useCallback((e: React.PointerEvent) => {
    if (!dragging.current) return
    setPan({
      x: dragging.current.panX + (e.clientX - dragging.current.startX),
      y: dragging.current.panY + (e.clientY - dragging.current.startY),
    })
  }, [])

  const onPointerUp = useCallback(() => { dragging.current = null }, [])

  const handleAnnotationClick = useCallback((e: React.MouseEvent) => {
    if (!annotationMode || !onAddAnnotation) return
    if (pending) { setPending(null); return }
    const rect = containerRef.current?.getBoundingClientRect()
    if (!rect) return
    setPending({ x: e.clientX - rect.left, y: e.clientY - rect.top })
  }, [annotationMode, onAddAnnotation, pending])

  const isFit = scale === DEFAULT_ZOOM && pan.x === 0 && pan.y === 0
  const pct = Math.round(scale * 100)

  if (!content || !content.startsWith('data:')) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3 p-8">
        <ImageIcon className="size-12 text-muted-foreground/40" />
        <p className="text-sm text-muted-foreground">{file.name}</p>
      </div>
    )
  }

  const hasAnnotations = annotations && annotations.length > 0
  const showOverlay = annotationMode || hasAnnotations

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
        <ImageIcon className="size-3.5 text-amber-500" />
        <span className="text-xs font-medium text-foreground">{file.name}</span>
        <div className="flex-1" />
        <div className="flex items-center gap-0.5">
          <button onClick={zoomOut} disabled={scale <= ZOOM_STEPS[0]} className="rounded p-1 text-muted-foreground hover:text-foreground hover:bg-muted disabled:opacity-30 transition" title="Zoom out">
            <ZoomOut className="size-3" />
          </button>
          <span className="text-[10px] tabular-nums text-muted-foreground w-8 text-center">{pct}%</span>
          <button onClick={zoomIn} disabled={scale >= ZOOM_STEPS[ZOOM_STEPS.length - 1]} className="rounded p-1 text-muted-foreground hover:text-foreground hover:bg-muted disabled:opacity-30 transition" title="Zoom in">
            <ZoomIn className="size-3" />
          </button>
          <button onClick={resetZoom} disabled={isFit} className="rounded p-1 text-muted-foreground hover:text-foreground hover:bg-muted disabled:opacity-30 transition ml-1" title="Reset zoom">
            <RotateCcw className="size-3" />
          </button>
        </div>
      </div>
      <div
        ref={containerRef}
        className={`flex-1 overflow-auto bg-black/5 dark:bg-black/20 relative ${
          annotationMode ? 'cursor-crosshair' : scale > 1 ? 'cursor-grab active:cursor-grabbing' : ''
        }`}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
      >
        <div className="flex items-center justify-center h-full p-4">
          <img
            src={content}
            alt={file.name}
            draggable={false}
            className="rounded max-w-full max-h-full object-contain select-none"
            style={{
              transform: `scale(${scale}) translate(${pan.x / scale}px, ${pan.y / scale}px)`,
              transformOrigin: 'center center',
            }}
          />
        </div>

        {/* Annotation overlay — pointer capture + markers */}
        {showOverlay && (
          <div
            className={`absolute inset-0 z-10 ${annotationMode ? 'pointer-events-auto' : 'pointer-events-none'}`}
            onClick={annotationMode ? handleAnnotationClick : undefined}
          >
            {annotations?.map((ann, i) => (
              <AnnotationMarker
                key={ann.id}
                annotation={ann}
                index={i}
                isActive={activeAnnotationId === ann.id}
                scrollTop={0}
                scrollLeft={0}
                onClick={() => onAnnotationClick?.(ann.id)}
                onRemove={() => onRemoveAnnotation?.(ann.id)}
              />
            ))}

            {pending && onAddAnnotation && (
              <AnnotationInput
                pending={{ type: 'click', x: pending.x, y: pending.y }}
                onConfirm={(note) => {
                  onAddAnnotation(pending.x, pending.y, note)
                  setPending(null)
                }}
                onCancel={() => setPending(null)}
              />
            )}
          </div>
        )}
      </div>
    </div>
  )
}
