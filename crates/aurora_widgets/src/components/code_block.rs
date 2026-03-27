use crate::widgets::{EventResponse, LayoutCtx, Widget};
use aurora_core::color::Color;
use aurora_core::geometry::corners::Corners;
use aurora_core::geometry::edges::Edges;
use aurora_core::geometry::rect::Rect;
use aurora_core::geometry::size::Size;
use aurora_core::kmi::mouse::{MouseEvent, MouseState};
use aurora_render::canvas::Canvas;
use aurora_syntax::{SyntaxSet, TokenType};
use aurora_theme::slots;
use std::ops::Range;
use std::time::Instant;

use super::colors;

const COPY_BTN_WIDTH: f32 = 56.0;
const COPY_BTN_HEIGHT: f32 = 24.0;
const COPY_BTN_MARGIN: f32 = 8.0;
const COPIED_DISPLAY_SECS: f32 = 1.5;

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
    // Copy button state
    copy_hover: bool,
    copied_at: Option<Instant>,
    copy_label: Option<aurora_text::text_layout::TextLayout>,
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
            copy_hover: false,
            copied_at: None,
            copy_label: None,
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

    fn copy_btn_rect(rect: &Rect) -> Rect {
        Rect::new(
            rect.x2 - COPY_BTN_WIDTH - COPY_BTN_MARGIN,
            rect.y1 + COPY_BTN_MARGIN,
            rect.x2 - COPY_BTN_MARGIN,
            rect.y1 + COPY_BTN_MARGIN + COPY_BTN_HEIGHT,
        )
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

        // Build copy button label
        let is_copied = self
            .copied_at
            .is_some_and(|t| t.elapsed().as_secs_f32() < COPIED_DISPLAY_SECS);
        let copy_text = if is_copied { "Copied!" } else { "Copy" };
        let mut copy_opts = ctx.font_options.clone();
        copy_opts.size = Some(11.0);
        self.copy_label = Some(aurora_text::text_layout::TextLayout::new(
            ctx.font_manager,
            copy_text,
            &copy_opts,
            colors::muted_foreground(),
            None,
        ));

        let num_lines = lines.len();
        let content_height = num_lines as f32 * self.line_height + self.padding.vertical();

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
            if self.show_line_numbers
                && let Some(Some(nl)) = self.line_number_layouts.get(i)
            {
                canvas.draw_text(nl, line_num_x as i32, y as i32);
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

        // Draw copy button
        let btn = Self::copy_btn_rect(&rect);
        let btn_bg = if self.copy_hover {
            colors::border()
        } else {
            colors::muted_foreground().opacity(0.15)
        };
        canvas.fill_rounded_rect(btn, Corners::all(4.0), btn_bg);
        if let Some(ref label) = self.copy_label {
            let ls = label.size();
            let lx = btn.x1 + (btn.width() - ls.width) / 2.0;
            let ly = btn.y1 + (btn.height() - ls.height) / 2.0;
            canvas.draw_text(label, lx as i32, ly as i32);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    fn event(&mut self, event: &aurora_core::kmi::WidgetEvent, rect: Rect) -> EventResponse {
        let mouse = match event {
            aurora_core::kmi::WidgetEvent::Mouse(m) => m,
            _ => return EventResponse::default(),
        };
        let btn = Self::copy_btn_rect(&rect);
        match mouse {
            MouseEvent::MouseMoveEvent(pos) => {
                self.copy_hover = btn.contains(pos);
                if self.copy_hover {
                    return EventResponse {
                        cursor: Some(aurora_core::kmi::cursor_icon::CursorIcon::Pointer),
                        ..Default::default()
                    };
                }
            }
            MouseEvent::MouseClickEvent(click)
                if click.state == MouseState::Released && btn.contains(&click.position) =>
            {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(&self.code);
                }
                self.copied_at = Some(Instant::now());
                return EventResponse {
                    handled: true,
                    ..Default::default()
                };
            }
            _ => {}
        }
        EventResponse::default()
    }

    fn needs_animation(&self) -> bool {
        self.copied_at
            .is_some_and(|t| t.elapsed().as_secs_f32() < COPIED_DISPLAY_SECS)
    }
}
