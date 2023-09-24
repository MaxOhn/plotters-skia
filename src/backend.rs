use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, DrawingBackend, DrawingErrorKind,
};
use skia_safe::{
    images, AlphaType, BlendMode, Canvas, Color, ColorType, Data, ImageInfo, Paint, PaintStyle,
    Path, Rect,
};

pub struct SkiaBackend<'a> {
    canvas: &'a mut Canvas,
    width: u32,
    height: u32,
    blend_mode: Option<BlendMode>,
}

#[derive(Debug)]
pub enum SkiaError {
    Typeface,
    ImageFromRaster,
}

impl Display for SkiaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(self, f)
    }
}

impl StdError for SkiaError {}

impl<'a> SkiaBackend<'a> {
    pub fn new(canvas: &'a mut Canvas, w: u32, h: u32) -> Self {
        Self {
            canvas,
            width: w,
            height: h,
            blend_mode: None,
        }
    }

    pub fn set_blend_mode(&mut self, blend_mode: Option<BlendMode>) -> &mut Self {
        self.blend_mode = blend_mode;

        self
    }

    fn paint(&self, color: BackendColor) -> Paint {
        let alpha = (color.alpha * 255.0) as u8;
        let (r, g, b) = color.rgb;
        let color = Color::from_argb(alpha, r, g, b);

        let mut paint = Paint::default();
        paint.set_color(color);

        if let Some(mode) = self.blend_mode {
            paint.set_blend_mode(mode);
        }

        paint
    }

    // fn font<TStyle: BackendTextStyle>(font: &TStyle) -> Result<Font, SkiaError> {
    //     let font_style = match font.style() {
    //         PFontStyle::Normal => FontStyle::new(Weight::NORMAL, Width::NORMAL, Slant::Upright),
    //         PFontStyle::Oblique => FontStyle::new(Weight::NORMAL, Width::NORMAL, Slant::Oblique),
    //         PFontStyle::Italic => FontStyle::new(Weight::NORMAL, Width::NORMAL, Slant::Italic),
    //         PFontStyle::Bold => FontStyle::new(Weight::BOLD, Width::NORMAL, Slant::Upright),
    //     };

    //     let typeface =
    //         Typeface::new(font.family().as_str(), font_style).ok_or(SkiaError::Typeface)?;

    //     Ok(Font::new(typeface, Some(font.size() as f32 * 0.83)))
    // }

    fn draw_path_<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
        filled: bool,
    ) {
        let mut paint = self.paint(style.color());

        paint
            .set_stroke_width(style.stroke_width() as f32)
            .set_anti_alias(true);

        if filled {
            paint.set_style(PaintStyle::Fill);
        } else {
            paint.set_style(PaintStyle::Stroke);
        }

        let mut points = path.into_iter();
        let mut path = Path::new();

        if let Some(point) = points.next() {
            path.move_to(point);

            for point in points {
                path.line_to(point);
            }
        }

        self.canvas.draw_path(&path, &paint);
    }
}

impl<'a> DrawingBackend for SkiaBackend<'a> {
    type ErrorType = SkiaError;

    #[inline]
    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    #[inline]
    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    #[inline]
    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    #[inline]
    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.canvas.draw_point(point, &self.paint(color));

        Ok(())
    }

    #[inline]
    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let mut paint = self.paint(style.color());

        paint
            .set_stroke_width(style.stroke_width() as f32)
            .set_anti_alias(true);

        self.canvas.draw_line(from, to, &paint);

        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let mut paint = self.paint(style.color());

        paint
            .set_stroke_width(style.stroke_width() as f32)
            .set_anti_alias(true);

        if fill {
            paint.set_style(PaintStyle::Fill);
        } else {
            paint.set_style(PaintStyle::Stroke);
        }

        let rect = Rect::new(
            upper_left.0 as f32,
            upper_left.1 as f32,
            bottom_right.0 as f32,
            bottom_right.1 as f32,
        );

        self.canvas.draw_rect(rect, &paint);

        Ok(())
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.draw_path_(path, style, false);

        Ok(())
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let mut paint = self.paint(style.color());

        paint
            .set_stroke_width(style.stroke_width() as f32)
            .set_anti_alias(true);

        if fill {
            paint.set_style(PaintStyle::Fill);
        } else {
            paint.set_style(PaintStyle::Stroke);
        }

        self.canvas.draw_circle(center, radius as f32, &paint);

        Ok(())
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.draw_path_(vert, style, true);

        Ok(())
    }

    fn blit_bitmap(
        &mut self,
        pos: BackendCoord,
        (iw, ih): (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let info = ImageInfo::new(
            (iw as i32, ih as i32),
            // Data has to be provided as an RGBA image buffer
            ColorType::RGBA8888,
            AlphaType::Opaque,
            None,
        );

        // SAFETY: `src` outlives `data`
        let data = unsafe { Data::new_bytes(src) };
        let row_bytes = iw * 4;

        let img = images::raster_from_data(&info, data, row_bytes as usize)
            .ok_or(DrawingErrorKind::DrawingError(SkiaError::ImageFromRaster))?;

        self.canvas.draw_image(img, pos, None);

        Ok(())
    }

    // Couldn't get font drawing to match the original close enough so it's just using the default implementation for text.
    // Much less efficient since it uses draw_pixel internally which is a shame but owell.

    // fn draw_text<TStyle: BackendTextStyle>(
    //     &mut self,
    //     text: &str,
    //     style: &TStyle,
    //     pos: BackendCoord,
    // ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
    //     let paint = Self::paint(style.color());
    //     let font = Self::font(style).map_err(DrawingErrorKind::DrawingError)?;

    //     let (width, rect) = font.measure_str(text, Some(&paint));
    //     let height = rect.height();

    //     let dx = match style.anchor().h_pos {
    //         HPos::Left => 0.0,
    //         HPos::Right => -width,
    //         HPos::Center => -width / 2.0,
    //     };

    //     let dy = match style.anchor().v_pos {
    //         VPos::Top => height,
    //         VPos::Center => height / 2.0,
    //         VPos::Bottom => 0.0,
    //     };

    //     let anchored_pos = (pos.0 as f32 + dx, pos.1 as f32 + dy - 1.0);

    //     match style.transform() {
    //         FontTransform::None => {}
    //         FontTransform::Rotate90 => {
    //             self.canvas.rotate(90.0, Some(pos.into()));
    //         }
    //         FontTransform::Rotate180 => {
    //             self.canvas.rotate(180.0, Some(pos.into()));
    //         }
    //         FontTransform::Rotate270 => {
    //             self.canvas.rotate(270.0, Some(pos.into()));
    //         }
    //     }

    //     self.canvas.draw_str(text, anchored_pos, &font, &paint);

    //     match style.transform() {
    //         FontTransform::None => {}
    //         FontTransform::Rotate90 => {
    //             self.canvas.rotate(-90.0, Some(pos.into()));
    //         }
    //         FontTransform::Rotate180 => {
    //             self.canvas.rotate(-180.0, Some(pos.into()));
    //         }
    //         FontTransform::Rotate270 => {
    //             self.canvas.rotate(-270.0, Some(pos.into()));
    //         }
    //     }

    //     Ok(())
    // }

    // fn estimate_text_size<TStyle: BackendTextStyle>(
    //     &self,
    //     text: &str,
    //     style: &TStyle,
    // ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
    //     let paint = Self::paint(style.color());
    //     let font = Self::font(style).map_err(DrawingErrorKind::DrawingError)?;
    //     let (_, rect) = font.measure_str(text, Some(&paint));

    //     Ok((rect.width() as u32, rect.height() as u32))
    // }
}
