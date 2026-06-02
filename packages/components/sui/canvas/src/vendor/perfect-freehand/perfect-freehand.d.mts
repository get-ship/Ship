// VENDORED TYPES — perfect-freehand 1.2.3, MIT (see ./LICENSE).
// Vendored 2026-06-02. Self-contained; derived from the tarball's
// `dist/types/*.d.ts`, inlined into one declaration file. DO NOT EDIT.

export type Vec2 = [number, number];

export interface StrokeOptions {
	size?: number;
	thinning?: number;
	smoothing?: number;
	streamline?: number;
	easing?: (pressure: number) => number;
	simulatePressure?: boolean;
	start?: {
		cap?: boolean;
		taper?: number | boolean;
		easing?: (distance: number) => number;
	};
	end?: {
		cap?: boolean;
		taper?: number | boolean;
		easing?: (distance: number) => number;
	};
	last?: boolean;
}

export interface StrokePoint {
	point: Vec2;
	pressure: number;
	distance: number;
	vector: Vec2;
	runningLength: number;
}

export declare function getStroke(points: (number[] | {
	x: number;
	y: number;
	pressure?: number;
})[], options?: StrokeOptions): Vec2[];

export declare function getStrokePoints<T extends number[], K extends {
	x: number;
	y: number;
	pressure?: number;
}>(points: (T | K)[], options?: StrokeOptions): StrokePoint[];

export declare function getStrokeOutlinePoints(points: StrokePoint[], options?: Partial<StrokeOptions>): Vec2[];

export default getStroke;
