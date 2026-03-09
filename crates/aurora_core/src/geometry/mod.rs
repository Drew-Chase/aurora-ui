/// Geometry primitives for 2D layout and rendering.
///
/// This module provides the core geometric types used throughout AuroraUI:
///
/// - [`Point`](point::Point) — A 2D position.
/// - [`Size`](size::Size) — Width and height dimensions.
/// - [`Rect`](rect::Rect) — An axis-aligned rectangle.
/// - [`Edges`](edges::Edges) — Edge insets (padding, margins, borders).
/// - [`Corners`](corners::Corners) — Corner radii for rounded rectangles.
pub mod rect;
pub mod point;
pub mod size;
pub mod edges;
pub mod corners;
