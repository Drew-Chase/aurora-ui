use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_render::canvas::Canvas;
use aurora_syntax::{SyntaxSet, TokenType};
use aurora_theme::slots;
use std::ops::Range;

use super::colors;

/// A code block widget with syntax highlighting and line numbers.
///
/// # Example
///
/// ```ignore
/// CodeBlock::new()
///     .code("fn main() {\n    println!(\"hello\");\n}")
///     .language("rust")
///     .font_size(13.0)
///     .show_line_numbers(true)
/// ```
pub struct CodeBlock {
    code: String,
    language: String,
    font_size: f32,
    line_number_width: f32,
    show_line_numbers: bool,
    background: Option<Color>,
    corners: Corners,
    padding: Edges,
    width: Option<f32>,
    // Cached layouts
    line_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    line_number_layouts: Vec<Option<aurora_text::text_layout::TextLayout>>,
    line_tokens: Vec<Vec<(Range<usize>, TokenType)>>,
    line_height: f32,
    cached_code: String,
    cached_language: String,
}

impl CodeBlock {
    pub fn new() -> Self {
        Self {
            code: String::new(),
            language: String::new(),
            font_size: 13.0,
            line_number_width: 40.0,
            show_line_numbers: true,
            background: None,
            corners: Corners::all(8.0),
            padding: Edges::new(16.0, 16.0, 16.0, 16.0),
            width: None,
            line_layouts: Vec::new(),
            line_number_layouts: Vec::new(),
            line_tokens: Vec::new(),
            line_height: 0.0,
            cached_code: String::new(),
            cached_language: String::new(),
        }
    }

    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn corners(mut self, corners: Corners) -> Self {
        self.corners = corners;
        self
    }

    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    fn token_color(token: TokenType) -> Color {
        match token {
            TokenType::Keyword => aurora_theme::color(slots::SYNTAX_KEYWORD),
            TokenType::String => aurora_theme::color(slots::SYNTAX_STRING),
            TokenType::Comment => aurora_theme::color(slots::SYNTAX_COMMENT),
            TokenType::Number => aurora_theme::color(slots::SYNTAX_NUMBER),
            TokenType::Function => aurora_theme::color(slots::SYNTAX_FUNCTION),
            TokenType::Type => aurora_theme::color(slots::SYNTAX_TYPE),
            TokenType::Operator => aurora_theme::color(slots::SYNTAX_OPERATOR),
            TokenType::Punctuation => aurora_theme::color(slots::SYNTAX_PUNCTUATION),
            TokenType::Attribute => aurora_theme::color(slots::SYNTAX_ATTRIBUTE),
            TokenType::Tag => aurora_theme::color(slots::SYNTAX_TAG),
            TokenType::Constant => aurora_theme::color(slots::SYNTAX_CONSTANT),
            TokenType::Plain => aurora_theme::color(slots::SYNTAX_PLAIN),
        }
    }
}

impl Default for CodeBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for CodeBlock {
    fn layout(&mut self, available: Size, ctx: &mut LayoutCtx) -> Size {
        let w = self.width.unwrap_or(available.width);
        let needs_retokenize =
            self.cached_code != self.code || self.cached_language != self.language;

        if needs_retokenize {
            self.cached_code = self.code.clone();
            self.cached_language = self.language.clone();

            // Tokenize per line
            let syntax = SyntaxSet::builtin();
            let full_tokens = syntax.highlight(&self.code, &self.language);
            let lines: Vec<&str> = self.code.split('\n').collect();

            self.line_tokens.clear();
            let mut line_start = 0usize;
            for line in &lines {
                let line_end = line_start + line.len();
                // Collect tokens that overlap this line, adjusting ranges to be line-relative
                let line_toks: Vec<(Range<usize>, TokenType)> = full_tokens
                    .iter()
                    .filter(|(r, _)| r.start < line_end && r.end > line_start)
                    .map(|(r, t)| {
                        let s = r.start.saturating_sub(line_start);
                        let e = (r.end - line_start).min(line.len());
                        (s..e, *t)
                    })
                    .collect();
                self.line_tokens.push(line_toks);
                line_start = line_end + 1; // +1 for the newline
            }
        }

        // Build text layouts
        let lines: Vec<&str> = self.code.split('\n').collect();
        let mut opts = ctx.font_options.clone();
        opts.size = Some(self.font_size);
        opts.family = Some("Consolas".into());

        self.line_layouts.clear();
        self.line_number_layouts.clear();

        let plain_color = aurora_theme::color(slots::SYNTAX_PLAIN);
        let muted_fg = colors::muted_foreground();

        for (i, line) in lines.iter().enumerate() {
            let text = if line.is_empty() { " " } else { line };
            let tl = aurora_text::text_layout::TextLayout::new(
                ctx.font_manager,
                text,
                &opts,
                plain_color,
                None,
            );
            if i == 0 {
                self.line_height = tl.size().height.max(self.font_size * 1.5);
            }
            self.line_layouts.push(Some(tl));

            if self.show_line_numbers {
                let num = format!("{}", i + 1);
                let nl = aurora_text::text_layout::TextLayout::new(
                    ctx.font_manager,
                    &num,
                    &opts,
                    muted_fg,
                    Some(aurora_text::cosmic_text::Align::Right),
                );
                self.line_number_layouts.push(Some(nl));
            }
        }

        let num_lines = lines.len();
        let content_height =
            num_lines as f32 * self.line_height + self.padding.vertical();

        Size::new(w, content_height)
    }

    fn paint(&self, canvas: &mut Canvas, rect: Rect) {
        let bg = self.background.unwrap_or(colors::muted());
        canvas.fill_rounded_rect(rect, self.corners, bg);

        let x_start = rect.x1 + self.padding.left;
        let y_start = rect.y1 + self.padding.top;
        let line_num_x = x_start;
        let code_x = if self.show_line_numbers {
            x_start + self.line_number_width
        } else {
            x_start
        };

        for (i, layout) in self.line_layouts.iter().enumerate() {
            let y = y_start + i as f32 * self.line_height;

            // Draw line number
            if self.show_line_numbers {
                if let Some(Some(nl)) = self.line_number_layouts.get(i) {
                    canvas.draw_text(nl, line_num_x as i32, y as i32);
                }
            }

            // Draw code line with syntax colors
            if let Some(tl) = layout {
                let color_ranges: Vec<(Range<usize>, Color)> = self
                    .line_tokens
                    .get(i)
                    .map(|tokens| {
                        tokens
                            .iter()
                            .map(|(r, t)| (r.clone(), Self::token_color(*t)))
                            .collect()
                    })
                    .unwrap_or_default();

                canvas.draw_rich_text(tl, code_x as i32, y as i32, &color_ranges);
            }
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, _event: &aurora_core::kmi::WidgetEvent, _rect: Rect) -> EventResponse {
        EventResponse::default()
    }
}
