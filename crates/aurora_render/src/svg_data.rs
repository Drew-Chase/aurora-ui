/// A parsed SVG document that can be rasterized at any size.
///
/// Uses `usvg` for SVG parsing and `tiny-skia` for rasterization — no filter
/// support (avoids the upstream `resvg`/`rgb` crate compatibility issue).
///
/// The SVG is parsed once at construction time. Call [`render`](Self::render)
/// to rasterize it into RGBA pixels at the desired dimensions.
///
/// # Example
///
/// ```no_run
/// use aurora_render::svg_data::SvgData;
///
/// let svg = SvgData::from_bytes(include_bytes!("icon.svg")).unwrap();
/// let rgba = svg.render(64, 64);
/// ```
pub struct SvgData {
    tree: usvg::Tree,
}

impl SvgData {
    /// Parses an SVG document from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, usvg::Error> {
        let tree = usvg::Tree::from_data(bytes, &usvg::Options::default())?;
        Ok(Self { tree })
    }

    /// Returns the intrinsic (viewBox) size of the SVG.
    pub fn size(&self) -> (f32, f32) {
        let size = self.tree.size();
        (size.width(), size.height())
    }

    /// Rasterizes the SVG to RGBA pixels at the given dimensions.
    ///
    /// The output is straight (non-premultiplied) RGBA, 4 bytes per pixel,
    /// suitable for passing to [`Canvas::draw_image`](crate::canvas::Canvas::draw_image).
    pub fn render(&self, width: u32, height: u32) -> Vec<u8> {
        let Some(mut pixmap) = tiny_skia::Pixmap::new(width, height) else {
            return Vec::new();
        };

        let size = self.tree.size();
        let scale_x = width as f32 / size.width();
        let scale_y = height as f32 / size.height();

        render_nodes(&self.tree.root(), tiny_skia::Transform::from_scale(scale_x, scale_y), &mut pixmap);

        // Convert from premultiplied to straight alpha
        let data = pixmap.data();
        let mut out = Vec::with_capacity(data.len());
        for chunk in data.chunks_exact(4) {
            let (r, g, b, a) = (chunk[0], chunk[1], chunk[2], chunk[3]);
            if a == 0 {
                out.extend_from_slice(&[0, 0, 0, 0]);
            } else if a == 255 {
                out.extend_from_slice(&[r, g, b, 255]);
            } else {
                let af = a as f32 / 255.0;
                out.push((r as f32 / af).min(255.0) as u8);
                out.push((g as f32 / af).min(255.0) as u8);
                out.push((b as f32 / af).min(255.0) as u8);
                out.push(a);
            }
        }
        out
    }
}

/// Recursively renders usvg nodes into a tiny-skia pixmap.
fn render_nodes(parent: &usvg::Group, transform: tiny_skia::Transform, pixmap: &mut tiny_skia::Pixmap) {
    for node in parent.children() {
        match node {
            usvg::Node::Group(group) => {
                let child_transform = transform.pre_concat(group.transform());
                render_nodes(group, child_transform, pixmap);
            }
            usvg::Node::Path(path) => {
                render_path(path, transform, pixmap);
            }
            usvg::Node::Image(_) | usvg::Node::Text(_) => {
                // Embedded images and text not yet supported
            }
        }
    }
}

/// Renders a single usvg path (fill + stroke) into the pixmap.
fn render_path(path: &usvg::Path, transform: tiny_skia::Transform, pixmap: &mut tiny_skia::Pixmap) {
    let Some(skia_path) = convert_path(path) else {
        return;
    };

    if let Some(ref fill) = path.fill() {
        if let Some(paint) = convert_paint(&fill.paint(), fill.opacity()) {
            let rule = match fill.rule() {
                usvg::FillRule::NonZero => tiny_skia::FillRule::Winding,
                usvg::FillRule::EvenOdd => tiny_skia::FillRule::EvenOdd,
            };
            pixmap.fill_path(&skia_path, &paint, rule, transform, None);
        }
    }

    if let Some(ref stroke) = path.stroke() {
        if let Some(paint) = convert_paint(&stroke.paint(), stroke.opacity()) {
            let skia_stroke = tiny_skia::Stroke {
                width: stroke.width().get(),
                line_cap: match stroke.linecap() {
                    usvg::LineCap::Butt => tiny_skia::LineCap::Butt,
                    usvg::LineCap::Round => tiny_skia::LineCap::Round,
                    usvg::LineCap::Square => tiny_skia::LineCap::Square,
                },
                line_join: match stroke.linejoin() {
                    usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => tiny_skia::LineJoin::Miter,
                    usvg::LineJoin::Round => tiny_skia::LineJoin::Round,
                    usvg::LineJoin::Bevel => tiny_skia::LineJoin::Bevel,
                },
                miter_limit: stroke.miterlimit().get(),
                ..tiny_skia::Stroke::default()
            };
            pixmap.stroke_path(&skia_path, &paint, &skia_stroke, transform, None);
        }
    }
}

/// Converts a usvg path to a tiny-skia path.
fn convert_path(path: &usvg::Path) -> Option<tiny_skia::Path> {
    let mut pb = tiny_skia::PathBuilder::new();
    for seg in path.data().segments() {
        match seg {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => pb.move_to(p.x, p.y),
            usvg::tiny_skia_path::PathSegment::LineTo(p) => pb.line_to(p.x, p.y),
            usvg::tiny_skia_path::PathSegment::QuadTo(p1, p) => pb.quad_to(p1.x, p1.y, p.x, p.y),
            usvg::tiny_skia_path::PathSegment::CubicTo(p1, p2, p) => {
                pb.cubic_to(p1.x, p1.y, p2.x, p2.y, p.x, p.y)
            }
            usvg::tiny_skia_path::PathSegment::Close => pb.close(),
        }
    }
    pb.finish()
}

/// Converts a usvg paint to a tiny-skia paint (solid colors only).
fn convert_paint(paint: &usvg::Paint, opacity: usvg::Opacity) -> Option<tiny_skia::Paint<'static>> {
    match paint {
        usvg::Paint::Color(color) => {
            let mut p = tiny_skia::Paint::default();
            p.set_color_rgba8(color.red, color.green, color.blue, (opacity.get() * 255.0) as u8);
            p.anti_alias = true;
            Some(p)
        }
        // Gradients and patterns not yet supported
        _ => None,
    }
}
