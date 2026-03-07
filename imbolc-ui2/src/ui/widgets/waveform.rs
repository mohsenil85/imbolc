use crate::ui::{Color, Palette, Rect, RenderBuf, Style};

/// Braille dot pattern offsets (2 columns x 4 rows)
/// Column 0: bits 0,1,2,6  Column 1: bits 3,4,5,7
const BRAILLE_DOT_OFFSETS: [[u8; 4]; 2] = [
    [0, 1, 2, 6], // left column (x=0): rows 0,1,2,3
    [3, 4, 5, 7], // right column (x=1): rows 0,1,2,3
];

/// Dot raster backing a braille plot (2x4 dots per character cell).
pub struct DotRaster {
    dot_width: usize,
    dot_height: usize,
    dots: Vec<bool>,
}

impl DotRaster {
    pub fn new(char_width: u16, char_height: u16) -> Self {
        let dot_width = char_width as usize * 2;
        let dot_height = char_height as usize * 4;
        Self {
            dot_width,
            dot_height,
            dots: vec![false; dot_width.saturating_mul(dot_height)],
        }
    }

    pub fn dot_width(&self) -> usize {
        self.dot_width
    }

    pub fn dot_height(&self) -> usize {
        self.dot_height
    }

    pub fn set(&mut self, x: usize, y: usize) {
        if x >= self.dot_width || y >= self.dot_height {
            return;
        }
        let idx = y * self.dot_width + x;
        if let Some(cell) = self.dots.get_mut(idx) {
            *cell = true;
        }
    }

    pub fn fill_vertical(&mut self, x: usize, y0: usize, y1: usize) {
        if x >= self.dot_width || self.dot_height == 0 {
            return;
        }
        let start = y0.min(y1).min(self.dot_height - 1);
        let end = y0.max(y1).min(self.dot_height - 1);
        for y in start..=end {
            self.set(x, y);
        }
    }

    pub fn is_set(&self, x: usize, y: usize) -> bool {
        if x >= self.dot_width || y >= self.dot_height {
            return false;
        }
        self.dots[y * self.dot_width + x]
    }

    #[allow(clippy::needless_range_loop)]
    pub fn char_pattern(&self, char_col: usize, char_row: usize) -> u8 {
        let mut pattern: u8 = 0;
        for dx in 0..2 {
            for dy in 0..4 {
                let dot_x = char_col * 2 + dx;
                let dot_y = char_row * 4 + dy;
                if self.is_set(dot_x, dot_y) {
                    pattern |= 1 << BRAILLE_DOT_OFFSETS[dx][dy];
                }
            }
        }
        pattern
    }
}

/// Convert a braille bit pattern to a Unicode braille character.
pub fn braille_char(pattern: u8) -> char {
    char::from_u32(0x2800 + pattern as u32).unwrap_or(' ')
}

/// Resolve waveform color based on distance from center.
pub fn waveform_gradient_color(frac_from_center: f32, p: &Palette) -> Color {
    let frac = frac_from_center.clamp(0.0, 1.0);
    let gradient = &p.waveform_gradient;
    let idx = ((frac * gradient.len() as f32) as usize).min(gradient.len() - 1);
    gradient[idx]
}

/// Resample any waveform into `columns` bins using peak-abs over each column window.
pub fn resample_peak_abs(samples: &[f32], columns: usize) -> Vec<f32> {
    if columns == 0 {
        return Vec::new();
    }
    if samples.is_empty() {
        return vec![0.0; columns];
    }

    let len = samples.len();
    let mut out = Vec::with_capacity(columns);
    for col in 0..columns {
        let start = col * len / columns;
        let mut end = (col + 1) * len / columns;
        if end <= start {
            end = (start + 1).min(len);
        }

        let mut peak = 0.0_f32;
        for &sample in &samples[start..end] {
            peak = peak.max(sample.abs());
        }
        out.push(peak.clamp(0.0, 1.0));
    }

    out
}

/// Draw a braille waveform from peak data into a rectangular area.
///
/// `grid_x`, `grid_y`, `grid_width`, `grid_height` define the character area.
/// Peaks are resampled to fit the dot-resolution width.
/// Colors use the waveform gradient from the palette.
pub fn draw_braille_waveform(
    peaks: &[f32],
    grid_x: u16,
    grid_y: u16,
    grid_width: u16,
    grid_height: u16,
    buf: &mut RenderBuf,
    p: &Palette,
) {
    if grid_width == 0 || grid_height == 0 || peaks.is_empty() {
        return;
    }

    let mut raster = DotRaster::new(grid_width, grid_height);
    if raster.dot_width() == 0 || raster.dot_height() == 0 {
        return;
    }

    let center_dot_y = raster.dot_height() / 2;
    let envelope = resample_peak_abs(peaks, raster.dot_width());

    for (idx, amplitude) in envelope.iter().copied().enumerate() {
        let extent = (amplitude * center_dot_y as f32).round() as usize;
        if extent == 0 {
            continue;
        }
        let top = center_dot_y.saturating_sub(extent);
        let bottom = (center_dot_y + extent).min(raster.dot_height() - 1);
        raster.fill_vertical(idx, top, bottom);
    }

    let rows = grid_height as usize;
    for char_row in 0..rows {
        for char_col in 0..grid_width as usize {
            let pattern = raster.char_pattern(char_col, char_row);
            if pattern == 0 {
                continue;
            }
            let center = rows as f32 / 2.0;
            let dist = (char_row as f32 + 0.5 - center).abs();
            let frac = if center <= 0.0 { 0.0 } else { dist / center };
            let color = waveform_gradient_color(frac, p);
            buf.set_cell(
                grid_x + char_col as u16,
                grid_y + char_row as u16,
                braille_char(pattern),
                Style::new().fg(color),
            );
        }
    }
}

/// Draw a vertical playhead line at a normalized position (0.0-1.0).
pub fn draw_playhead(
    progress: f32,
    grid_x: u16,
    grid_y: u16,
    grid_width: u16,
    grid_height: u16,
    buf: &mut RenderBuf,
    color: Color,
) {
    if grid_width == 0 || grid_height == 0 {
        return;
    }
    let frac = progress.clamp(0.0, 1.0);
    let x_offset = ((grid_width.saturating_sub(1) as f32) * frac).round() as u16;
    let x = grid_x + x_offset;
    let style = Style::new().fg(color).bold();
    for y in 0..grid_height {
        buf.set_cell(x, grid_y + y, '\u{2502}', style);
    }
}

/// Draw a center line (dimmed horizontal line at the vertical midpoint).
pub fn draw_center_line(
    grid_x: u16,
    grid_y: u16,
    grid_width: u16,
    grid_height: u16,
    buf: &mut RenderBuf,
    p: &Palette,
) {
    let center_y = grid_y + grid_height / 2;
    let dim_style = Style::new().fg(p.dim);
    let rect = Rect::new(grid_x, center_y, grid_width, 1);
    let dashes = "╌".repeat(grid_width as usize);
    buf.draw_line(rect, &[(&dashes, dim_style)]);
}
