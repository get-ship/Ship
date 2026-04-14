// Re-exports the shared annotation helpers. The logic now lives in
// `@ship/ui/src/utils/annotation-helpers.ts` so both the React studio
// overlay and the `<ship-annotation-*>` Lit components share one
// implementation. Keep this file as a thin passthrough.

export {
  getCSSSelector,
  getElementsInRect,
  buildAnnotationEvent,
  newUlid,
} from '../../../../../../packages/ui/src/utils/annotation-helpers'
export type {
  AnnotationEvent,
  AnnotationKind,
  AnnotationIntent,
  AuthorKind,
  BuildAnnotationInput,
} from '../../../../../../packages/ui/src/utils/annotation-helpers'
