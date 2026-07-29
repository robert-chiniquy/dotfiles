use super::{c, is_sep, split_row, RESET};
use std::env;
use std::io::IsTerminal;

const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
// Neon-grit chart palette — rust / gold / dark grit, not vivid neon green.
// Surfaces: transparent field (terminal wallpaper shows through), rust grid,
// aged-paper axes. Accents stay warm and oxidized.
const GRID: [u8; 3] = [78, 42, 24]; // dark rust
const AXIS: [u8; 3] = [176, 152, 112]; // dirty gold / aged paper
const BPP: usize = 4; // RGBA so the chart field is transparent in iTerm
// Histogram ramp: dull brass (floor) → oxide red (ceiling).
const RAMP_LO: [u8; 3] = [156, 118, 48]; // brass / dull gold
const RAMP_HI: [u8; 3] = [148, 36, 24]; // oxide red
const COLORS: [[u8; 3]; 5] = [
    [196, 148, 42], // gold
    [168, 92, 36],  // rust amber
    [132, 64, 32],  // dark rust
    [106, 48, 28],  // tar-rust (VAPORWAVE Rust #6a3d2e-ish)
    [148, 36, 24],  // oxide red
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChartMode {
    Auto,
    Image,
    Text,
    Off,
}

impl ChartMode {
    pub(crate) fn from(value: &str) -> ChartMode {
        match value.trim().to_ascii_lowercase().as_str() {
            "image" | "img" | "png" => ChartMode::Image,
            "text" | "unicode" => ChartMode::Text,
            "off" | "none" | "0" => ChartMode::Off,
            _ => ChartMode::Auto,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ChartKind {
    Sparkline,
    Histogram,
    TimeSeries,
}

impl ChartKind {
    fn from(value: &str) -> ChartKind {
        match value.trim().to_ascii_lowercase().as_str() {
            "histogram" | "hist" | "bars" | "bar" => ChartKind::Histogram,
            "time-series" | "timeseries" | "time_series" | "line" => ChartKind::TimeSeries,
            _ => ChartKind::Sparkline,
        }
    }

    fn label(self) -> &'static str {
        match self {
            ChartKind::Sparkline => "sparkline",
            ChartKind::Histogram => "histogram",
            ChartKind::TimeSeries => "time series",
        }
    }
}

#[derive(Clone, Debug)]
struct Series {
    name: String,
    values: Vec<Option<f64>>,
}

#[derive(Clone, Debug)]
pub(crate) struct Chart {
    title: String,
    kind: ChartKind,
    labels: Vec<String>,
    series: Vec<Series>,
}

struct Builder {
    title: String,
    kind: ChartKind,
    header: Option<Vec<String>>,
    rows: Vec<Vec<String>>,
}

fn chart_title(heading: &str) -> Option<String> {
    let heading = heading.trim();
    if heading.eq_ignore_ascii_case("chart") {
        return Some("Chart".into());
    }
    let (prefix, title) = heading.split_once(':')?;
    if prefix.trim().eq_ignore_ascii_case("chart") && !title.trim().is_empty() {
        Some(title.trim().to_string())
    } else {
        None
    }
}

pub(crate) fn is_chart_heading(heading: &str) -> bool {
    chart_title(heading).is_some()
}

fn parse_number(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.is_empty()
        || matches!(
            value.to_ascii_lowercase().as_str(),
            "n/a" | "na" | "null" | "-"
        )
    {
        return None;
    }
    let negative = value.starts_with('(') && value.ends_with(')');
    let cleaned = value
        .trim_matches(|ch| ch == '(' || ch == ')')
        .trim_end_matches('%')
        .replace(',', "");
    cleaned
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
        .map(|number| if negative { -number } else { number })
}

fn finish(builder: Builder) -> Option<Chart> {
    let header = builder.header?;
    if builder.rows.is_empty() {
        return None;
    }
    if header.len() == 1 && builder.kind == ChartKind::Histogram {
        let observations: Vec<f64> = builder
            .rows
            .iter()
            .filter_map(|row| row.first().and_then(|value| parse_number(value)))
            .collect();
        let (&min, &max) = (
            observations.iter().min_by(|a, b| a.total_cmp(b))?,
            observations.iter().max_by(|a, b| a.total_cmp(b))?,
        );
        let bins = if (max - min).abs() < f64::EPSILON {
            1
        } else {
            (observations.len() as f64).sqrt().ceil().clamp(1.0, 12.0) as usize
        };
        let span = if bins == 1 {
            1.0
        } else {
            (max - min) / bins as f64
        };
        let mut counts = vec![0.0; bins];
        for value in observations {
            let index = if bins == 1 {
                0
            } else {
                (((value - min) / span).floor() as usize).min(bins - 1)
            };
            counts[index] += 1.0;
        }
        let labels = (0..bins)
            .map(|index| {
                if bins == 1 {
                    format_number(min)
                } else {
                    format!(
                        "{}–{}",
                        format_number(min + span * index as f64),
                        format_number(min + span * (index + 1) as f64)
                    )
                }
            })
            .collect();
        return Some(Chart {
            title: builder.title,
            kind: builder.kind,
            labels,
            series: vec![Series {
                name: "count".into(),
                values: counts.into_iter().map(Some).collect(),
            }],
        });
    }
    if header.len() < 2 {
        return None;
    }
    let labels: Vec<String> = builder
        .rows
        .iter()
        .map(|row| row.first().cloned().unwrap_or_default())
        .collect();
    let mut series = Vec::new();
    for (column, name) in header.iter().enumerate().skip(1) {
        let values: Vec<Option<f64>> = builder
            .rows
            .iter()
            .map(|row| row.get(column).and_then(|value| parse_number(value)))
            .collect();
        if values.iter().any(Option::is_some) {
            series.push(Series {
                name: name.clone(),
                values,
            });
        }
    }
    (!series.is_empty()).then_some(Chart {
        title: builder.title,
        kind: builder.kind,
        labels,
        series,
    })
}

pub(crate) fn parse(input: &str) -> Vec<Chart> {
    let mut charts = Vec::new();
    let mut current: Option<Builder> = None;
    for raw in input.lines() {
        let line = raw.trim();
        if let Some(heading) = line.strip_prefix("## ") {
            if let Some(builder) = current.take() {
                if let Some(chart) = finish(builder) {
                    charts.push(chart);
                }
            }
            current = chart_title(heading).map(|title| Builder {
                title,
                kind: ChartKind::Sparkline,
                header: None,
                rows: Vec::new(),
            });
            continue;
        }
        let Some(builder) = current.as_mut() else {
            continue;
        };
        if let Some((key, value)) = line.split_once(':') {
            if key.trim().eq_ignore_ascii_case("type") {
                builder.kind = ChartKind::from(value);
                continue;
            }
        }
        if line.starts_with('|') {
            let cells = split_row(line);
            if cells.is_empty() || is_sep(&cells) {
                continue;
            }
            if builder.header.is_none() {
                builder.header = Some(cells);
            } else {
                builder.rows.push(cells);
            }
        }
    }
    if let Some(builder) = current {
        if let Some(chart) = finish(builder) {
            charts.push(chart);
        }
    }
    charts
}

fn bounds(values: &[Option<f64>]) -> Option<(f64, f64)> {
    let mut numbers = values.iter().flatten().copied();
    let first = numbers.next()?;
    let mut min = first;
    let mut max = first;
    for value in numbers {
        min = min.min(value);
        max = max.max(value);
    }
    Some((min, max))
}

fn sparkline(values: &[Option<f64>]) -> String {
    let Some((min, max)) = bounds(values) else {
        return "·".repeat(values.len());
    };
    let flat = (max - min).abs() < f64::EPSILON;
    values
        .iter()
        .map(|value| match value {
            None => '·',
            Some(_) if flat => BLOCKS[3],
            Some(value) => {
                let ratio = ((value - min) / (max - min)).clamp(0.0, 1.0);
                BLOCKS[(ratio * (BLOCKS.len() - 1) as f64).floor() as usize]
            }
        })
        .collect()
}

fn trunc_text(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_string();
    }
    if width < 2 {
        return "…".chars().take(width).collect();
    }
    format!(
        "{}…",
        value.chars().take(width - 1).collect::<String>().trim_end()
    )
}

fn format_number(value: f64) -> String {
    if (value - value.round()).abs() < 0.000_001 {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn text_rows(chart: &Chart) -> usize {
    1 + match chart.kind {
        ChartKind::Histogram => chart.labels.len().min(6),
        ChartKind::Sparkline | ChartKind::TimeSeries => chart.series.len().min(5),
    }
}

// Axis annotation appended to a chart's header line. A histogram's horizontal
// bars map bin range -> count; for value charts we surface the value-axis range
// the sparkline/line otherwise hides.
fn axis_caption(chart: &Chart) -> String {
    match chart.kind {
        ChartKind::Histogram => " · x:range y:count".to_string(),
        ChartKind::Sparkline | ChartKind::TimeSeries => match numeric_bounds(chart) {
            Some((min, max)) => format!(" · y {}–{}", format_number(min), format_number(max)),
            None => String::new(),
        },
    }
}

// Per-cell brass→oxide gradient for a histogram bar (dark grit ramp).
// Ramps across this bar's own length, not a shared scale, so a short bar and a
// long bar both run the full range.
fn gradient_bar(count: usize) -> String {
    let mut bar = String::new();
    for i in 0..count {
        let p = if count > 1 {
            i as f64 / (count - 1) as f64
        } else {
            0.0
        };
        let col = ramp_at([RAMP_LO, RAMP_HI], p);
        bar.push_str(&format!("{}█{}", c(col), RESET));
    }
    bar
}

fn render_text(chart: &Chart, width: usize, budget: usize) -> String {
    if budget == 0 {
        return String::new();
    }
    let mut lines = vec![trunc_text(
        &format!(
            "CHART {} · {}{}",
            chart.title,
            chart.kind.label(),
            axis_caption(chart)
        ),
        width,
    )];
    let available = budget.saturating_sub(1);
    match chart.kind {
        ChartKind::Histogram => {
            let series = &chart.series[0];
            let max = series
                .values
                .iter()
                .flatten()
                .copied()
                .map(f64::abs)
                .fold(0.0, f64::max);
            let label_width = chart
                .labels
                .iter()
                .map(|label| label.chars().count())
                .max()
                .unwrap_or(1)
                .min(16);
            for (label, value) in chart
                .labels
                .iter()
                .zip(&series.values)
                .take(available.min(6))
            {
                let number = value.map(format_number).unwrap_or_else(|| "n/a".into());
                let bar_width = width.saturating_sub(label_width + number.chars().count() + 3);
                let count = value
                    .map(|value| {
                        if max == 0.0 {
                            1
                        } else {
                            ((value.abs() / max) * bar_width as f64).round() as usize
                        }
                    })
                    .unwrap_or(0)
                    .min(bar_width);
                lines.push(format!(
                    "{:<label_width$} {} {}",
                    trunc_text(label, label_width),
                    gradient_bar(count),
                    number,
                    label_width = label_width
                ));
            }
        }
        ChartKind::Sparkline | ChartKind::TimeSeries => {
            let name_width = chart
                .series
                .iter()
                .map(|series| series.name.chars().count())
                .max()
                .unwrap_or(1)
                .min(16);
            for series in chart.series.iter().take(available.min(5)) {
                let graph_width = width.saturating_sub(name_width + 1);
                let graph = trunc_text(&sparkline(&series.values), graph_width);
                lines.push(format!(
                    "{:<name_width$} {}",
                    trunc_text(&series.name, name_width),
                    graph,
                    name_width = name_width
                ));
            }
        }
    }
    lines.join("\n") + "\n"
}

#[derive(Clone)]
struct Raster {
    width: usize,
    height: usize,
    /// Packed RGBA (alpha 0 = transparent field; drawn ink is opaque).
    pixels: Vec<u8>,
}

impl Raster {
    fn new(width: usize, height: usize) -> Raster {
        // Fully transparent field so iTerm's wallpaper / true terminal bg shows
        // through instead of a solid black rectangle.
        Raster {
            width,
            height,
            pixels: vec![0; width * height * BPP],
        }
    }

    fn pixel(&mut self, x: i32, y: i32, color: [u8; 3]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let offset = (y as usize * self.width + x as usize) * BPP;
        self.pixels[offset..offset + 3].copy_from_slice(&color);
        self.pixels[offset + 3] = 255;
    }

    fn rgb_eq(px: &[u8], color: [u8; 3]) -> bool {
        px.len() >= 3 && px[0] == color[0] && px[1] == color[1] && px[2] == color[2] && px.get(3) != Some(&0)
    }

    fn line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: [u8; 3]) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            self.pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let twice = error * 2;
            if twice >= dy {
                error += dy;
                x0 += sx;
            }
            if twice <= dx {
                error += dx;
                y0 += sy;
            }
        }
    }

    fn rect(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: [u8; 3]) {
        for y in y0.min(self.height)..y1.min(self.height) {
            for x in x0.min(self.width)..x1.min(self.width) {
                self.pixel(x as i32, y as i32, color);
            }
        }
    }

    // Draw a string with the built-in 3x5 numeric font, scaled by `scale`. Only
    // glyphs known to `glyph()` render (digits, '.', '-', space); anything else
    // advances a blank cell so widths stay predictable.
    fn text(&mut self, x: i32, y: i32, text: &str, scale: i32, color: [u8; 3]) {
        let scale = scale.max(1);
        let mut cx = x;
        for ch in text.chars() {
            if let Some(rows) = glyph(ch) {
                for (ry, bits) in rows.iter().enumerate() {
                    for gx in 0..GLYPH_W {
                        if bits & (1 << (GLYPH_W - 1 - gx)) != 0 {
                            for dy in 0..scale {
                                for dx in 0..scale {
                                    self.pixel(cx + gx as i32 * scale + dx, y + ry as i32 * scale + dy, color);
                                }
                            }
                        }
                    }
                }
            }
            cx += (GLYPH_W as i32 + 1) * scale;
        }
    }
}

const GLYPH_W: usize = 3;
const GLYPH_H: usize = 5;

// Pixel width a `Raster::text` call for `s` at `scale` will occupy.
fn text_width(s: &str, scale: i32) -> i32 {
    s.chars().count() as i32 * (GLYPH_W as i32 + 1) * scale.max(1)
}

// 3x5 bitmap glyphs for numeric axis labels (bit2=left, bit1=mid, bit0=right).
// Numeric-only by design: axis ticks and histogram bin bounds are all numbers.
fn glyph(ch: char) -> Option<[u8; GLYPH_H]> {
    Some(match ch {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        '-' | '–' => [0b000, 0b000, 0b111, 0b000, 0b000],
        ' ' => [0; GLYPH_H],
        _ => return None,
    })
}

fn numeric_bounds(chart: &Chart) -> Option<(f64, f64)> {
    let mut numbers = chart
        .series
        .iter()
        .flat_map(|series| series.values.iter().flatten())
        .copied();
    let first = numbers.next()?;
    let (mut min, mut max) = (first, first);
    for number in numbers {
        min = min.min(number);
        max = max.max(number);
    }
    if (max - min).abs() < f64::EPSILON {
        min -= 1.0;
        max += 1.0;
    }
    Some((min, max))
}

// Linear RGB color ramp: `ramp[0]` at p=0, `ramp[1]` at p=1 (p is clamped).
fn ramp_at(ramp: [[u8; 3]; 2], p: f64) -> [u8; 3] {
    let p = p.clamp(0.0, 1.0);
    [0, 1, 2].map(|k| (ramp[0][k] as f64 + (ramp[1][k] as f64 - ramp[0][k] as f64) * p).round() as u8)
}

fn draw_chart(chart: &Chart, width: usize, height: usize) -> Raster {
    let mut raster = Raster::new(width, height);
    if width < 24 || height < 24 {
        return raster;
    }
    // Bounds first: the y-axis tick values derive from them. With no numeric
    // data we still draw an empty grid at the original tight margins.
    let Some((min, max)) = numeric_bounds(chart) else {
        let (left, right, top, bottom) = (12usize, width - 10, 10usize, height - 10);
        for step in 0..=4 {
            let y = top + (bottom - top) * step / 4;
            raster.line(left as i32, y as i32, right as i32, y as i32, GRID);
        }
        return raster;
    };

    // Y-axis tick values, top (max) down to bottom (min) — one per grid line.
    let scale = if height >= 120 { 2 } else { 1 };
    let y_ticks: [String; 5] =
        std::array::from_fn(|s| format_number(max - (max - min) * s as f64 / 4.0));
    let y_label_px = y_ticks.iter().map(|t| text_width(t, scale)).max().unwrap_or(0);
    let glyph_px = GLYPH_H as i32 * scale;

    // Reserve margins for labels when there is room; otherwise fall back to the
    // original tight margins so tiny thumbnails still draw the plot.
    let want_left = 6 + y_label_px as usize + 3;
    let want_bottom = glyph_px as usize + 3;
    let labels_fit = width > want_left + 24 && height > want_bottom + 28;
    let left = if labels_fit { want_left } else { 12 };
    let right = width - 10;
    let top = 10usize;
    let bottom = if labels_fit { height - 10 - want_bottom } else { height - 10 };

    for (step, label) in y_ticks.iter().enumerate() {
        let y = top + (bottom - top) * step / 4;
        raster.line(left as i32, y as i32, right as i32, y as i32, GRID);
        if labels_fit {
            let lx = left as i32 - text_width(label, scale) - 3;
            raster.text(lx.max(0), y as i32 - glyph_px / 2, label, scale, AXIS);
        }
    }

    let y_for = |value: f64| bottom as f64 - ((value - min) / (max - min)) * (bottom - top) as f64;

    match chart.kind {
        ChartKind::Histogram => {
            let series = &chart.series[0];
            let count = series.values.len().max(1);
            let slot = ((right - left) / count).max(1);
            let zero = y_for(0.0_f64.clamp(min, max)).round() as usize;
            for (index, value) in series.values.iter().enumerate() {
                let Some(value) = value else {
                    continue;
                };
                let x0 = left + index * slot + slot / 6;
                let x1 = (left + (index + 1) * slot)
                    .saturating_sub(slot / 6)
                    .max(x0 + 1);
                let value_y = y_for(*value).round().clamp(top as f64, bottom as f64) as usize;
                // Gradient keyed to the plot height and shared across every
                // bar: brass at the baseline, oxide red at the top of the
                // container. A bar's color at a given height is the same
                // regardless of its own height, so short bars stay gold and
                // only the tallest reach red.
                let field = zero.saturating_sub(top).max(1) as f64;
                for y in value_y.min(zero)..value_y.max(zero) + 1 {
                    let p = zero.saturating_sub(y) as f64 / field;
                    raster.rect(x0, y, x1, y + 1, ramp_at([RAMP_LO, RAMP_HI], p));
                }
            }
            // X-axis: the observed data range, read from the first/last bin labels.
            if labels_fit {
                if let (Some(first), Some(last)) = (chart.labels.first(), chart.labels.last()) {
                    let lo = first.split('–').next().unwrap_or(first).trim();
                    let hi = last.rsplit('–').next().unwrap_or(last).trim();
                    let ty = (bottom as i32 + 3).min(height as i32 - glyph_px - 1);
                    raster.text(left as i32, ty, lo, scale, AXIS);
                    raster.text(right as i32 - text_width(hi, scale), ty, hi, scale, AXIS);
                }
            }
        }
        ChartKind::Sparkline | ChartKind::TimeSeries => {
            for (series_index, series) in chart.series.iter().take(COLORS.len()).enumerate() {
                let color = COLORS[series_index % COLORS.len()];
                let count = series.values.len();
                let mut previous: Option<(i32, i32)> = None;
                for (index, value) in series.values.iter().enumerate() {
                    let Some(value) = value else {
                        previous = None;
                        continue;
                    };
                    let x = if count <= 1 {
                        (left + right) / 2
                    } else {
                        left + (right - left) * index / (count - 1)
                    };
                    let y = y_for(*value).round().clamp(top as f64, bottom as f64) as usize;
                    if let Some((px, py)) = previous {
                        raster.line(px, py, x as i32, y as i32, color);
                        raster.line(px, py + 1, x as i32, y as i32 + 1, color);
                    }
                    raster.rect(
                        x.saturating_sub(1),
                        y.saturating_sub(1),
                        x + 2,
                        y + 2,
                        color,
                    );
                    previous = Some((x as i32, y as i32));
                }
            }
        }
    }
    raster
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for byte in bytes {
        a = (a + *byte as u32) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

fn png_chunk(output: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(kind);
    output.extend_from_slice(data);
    let mut crc_data = Vec::with_capacity(kind.len() + data.len());
    crc_data.extend_from_slice(kind);
    crc_data.extend_from_slice(data);
    output.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

fn render_png(chart: &Chart, width: usize, height: usize) -> Vec<u8> {
    let width = width.clamp(32, 960);
    let height = height.clamp(24, 240);
    let raster = draw_chart(chart, width, height);
    let mut raw = Vec::with_capacity((width * BPP + 1) * height);
    for row in raster.pixels.chunks_exact(width * BPP) {
        raw.push(0); // filter: None
        raw.extend_from_slice(row);
    }

    let mut zlib = vec![0x78, 0x01];
    let chunks = raw.chunks(65_535);
    let count = chunks.len();
    for (index, block) in chunks.enumerate() {
        zlib.push(if index + 1 == count { 1 } else { 0 });
        let length = block.len() as u16;
        zlib.extend_from_slice(&length.to_le_bytes());
        zlib.extend_from_slice(&(!length).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&(width as u32).to_be_bytes());
    header.extend_from_slice(&(height as u32).to_be_bytes());
    // 8-bit RGBA (color type 6) — transparent field over iTerm wallpaper
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    png_chunk(&mut png, b"IHDR", &header);
    png_chunk(&mut png, b"IDAT", &zlib);
    png_chunk(&mut png, b"IEND", &[]);
    png
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let value = ((chunk[0] as u32) << 16)
            | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8)
            | chunk.get(2).copied().unwrap_or(0) as u32;
        output.push(ALPHABET[((value >> 18) & 63) as usize] as char);
        output.push(ALPHABET[((value >> 12) & 63) as usize] as char);
        output.push(if chunk.len() > 1 {
            ALPHABET[((value >> 6) & 63) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            ALPHABET[(value & 63) as usize] as char
        } else {
            '='
        });
    }
    output
}

fn iterm_image(png: &[u8], title: &str, width: usize, height: usize) -> String {
    let name = base64(format!("{}.png", title).as_bytes());
    format!(
        "\x1b]1337;File=name={};size={};width={};height={};preserveAspectRatio=0;inline=1:{}\x07\n",
        name,
        png.len(),
        width,
        height,
        base64(png)
    )
}

fn resolve_mode(
    requested: ChartMode,
    stdout_is_tty: bool,
    term_program: Option<&str>,
    lc_terminal: Option<&str>,
    in_tmux: bool,
) -> ChartMode {
    if requested != ChartMode::Auto {
        return requested;
    }
    let is_iterm = term_program == Some("iTerm.app") || lc_terminal == Some("iTerm2");
    if stdout_is_tty && is_iterm && !in_tmux {
        ChartMode::Image
    } else {
        ChartMode::Text
    }
}

pub(crate) fn configured_mode(cli: Option<ChartMode>) -> ChartMode {
    let requested = cli
        .or_else(|| {
            env::var("SCORECARD_CHARTS")
                .ok()
                .map(|value| ChartMode::from(&value))
        })
        .unwrap_or(ChartMode::Auto);
    resolve_mode(
        requested,
        std::io::stdout().is_terminal(),
        env::var("TERM_PROGRAM").ok().as_deref(),
        env::var("LC_TERMINAL").ok().as_deref(),
        env::var_os("TMUX").is_some(),
    )
}

pub(crate) fn desired_rows(charts: &[Chart], mode: ChartMode) -> usize {
    match mode {
        ChartMode::Off => 0,
        ChartMode::Image => charts.len() * 9,
        ChartMode::Text | ChartMode::Auto => charts.iter().map(text_rows).sum(),
    }
}

pub(crate) fn render(charts: &[Chart], mode: ChartMode, width: usize, mut budget: usize) -> String {
    let mut output = String::new();
    for chart in charts {
        if budget < 2 {
            break;
        }
        match mode {
            ChartMode::Off => break,
            ChartMode::Text | ChartMode::Auto => {
                let rows = text_rows(chart).min(budget);
                output.push_str(&render_text(chart, width, rows));
                budget -= rows;
            }
            ChartMode::Image => {
                let image_rows = budget.saturating_sub(1).min(8);
                if image_rows < 3 {
                    break;
                }
                output.push_str(&format!(
                    "CHART {} · {}{}\n",
                    chart.title,
                    chart.kind.label(),
                    axis_caption(chart)
                ));
                let pixel_width = (width * 8).clamp(320, 960);
                let pixel_height = (image_rows * 20).clamp(60, 240);
                let png = render_png(chart, pixel_width, pixel_height);
                output.push_str(&iterm_image(&png, &chart.title, width, image_rows));
                budget -= image_rows + 1;
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    // The histogram gradient ramp: base color at p=0, tip at p=1, linear and
    // clamped in between. Failure hypothesis: unclamped p pushing a channel out
    // of range, or the endpoints swapped.
    #[test]
    fn ramp_at_interpolates_and_clamps() {
        assert_eq!(ramp_at([RAMP_LO, RAMP_HI], 0.0), RAMP_LO);
        assert_eq!(ramp_at([RAMP_LO, RAMP_HI], 1.0), RAMP_HI);
        assert_eq!(ramp_at([RAMP_LO, RAMP_HI], -1.0), RAMP_LO, "p below 0 clamps to base");
        assert_eq!(ramp_at([RAMP_LO, RAMP_HI], 2.0), RAMP_HI, "p above 1 clamps to tip");
        let mid = ramp_at([RAMP_LO, RAMP_HI], 0.5);
        for k in 0..3 {
            let (lo, hi) = (RAMP_LO[k].min(RAMP_HI[k]), RAMP_LO[k].max(RAMP_HI[k]));
            assert!(mid[k] >= lo && mid[k] <= hi, "midpoint channel {k} out of range");
        }
    }

    const TABLE: &str = "\
## Chart: Reveal latency
type: time-series
| day | p50 | p95 |
| --- | ---: | ---: |
| Mon | 10 | 20 |
| Tue | 15 | 24 |
| Wed | 12 | 31 |
";

    #[test]
    fn parses_chart_tables_into_typed_series() {
        let charts = parse(TABLE);
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].title, "Reveal latency");
        assert_eq!(charts[0].kind, ChartKind::TimeSeries);
        assert_eq!(charts[0].labels, vec!["Mon", "Tue", "Wed"]);
        assert_eq!(charts[0].series[0].name, "p50");
        assert_eq!(
            charts[0].series[1].values,
            vec![Some(20.0), Some(24.0), Some(31.0)]
        );
    }

    #[test]
    fn malformed_cells_are_missing_data_not_zero() {
        let charts = parse(
            "## Chart: Queue\n\
             type: sparkline\n\
             | day | depth |\n\
             | --- | ---: |\n\
             | Mon | 4 |\n\
             | Tue | n/a |\n\
             | Wed | 8 |\n",
        );
        assert_eq!(charts[0].series[0].values, vec![Some(4.0), None, Some(8.0)]);
    }

    #[test]
    fn one_column_histograms_bin_raw_observations() {
        let charts = parse(
            "## Chart: Latency distribution\n\
             type: histogram\n\
             | milliseconds |\n\
             | ---: |\n\
             | 10 |\n\
             | 12 |\n\
             | 18 |\n\
             | 31 |\n",
        );
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].kind, ChartKind::Histogram);
        assert_eq!(
            charts[0].series[0].values.iter().flatten().sum::<f64>(),
            4.0
        );
        assert_eq!(charts[0].labels.len(), charts[0].series[0].values.len());
    }

    #[test]
    fn sparkline_normalizes_without_losing_missing_points() {
        assert_eq!(sparkline(&[Some(0.0), Some(5.0), None, Some(10.0)]), "▁▄·█");
        assert_eq!(sparkline(&[Some(7.0), Some(7.0)]), "▄▄");
    }

    #[test]
    fn png_encoder_produces_a_bounded_valid_shape() {
        let chart = &parse(TABLE)[0];
        let png = render_png(chart, 320, 96);
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 320);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 96);
        // IHDR: bit depth 8, color type 6 (RGBA) so the field can be transparent
        assert_eq!(png[24], 8);
        assert_eq!(png[25], 6);
        // store filter + RGBA per pixel, store blocks, chunk framing — well under 8 B/px
        assert!(png.len() < 320 * 96 * 8);
    }

    #[test]
    fn iterm_image_sequence_embeds_png_with_dimensions() {
        let png = render_png(&parse(TABLE)[0], 320, 96);
        let sequence = iterm_image(&png, "Reveal latency", 80, 7);
        assert!(sequence.starts_with("\u{1b}]1337;File="));
        assert!(sequence.contains("inline=1"));
        assert!(sequence.contains("width=80"));
        assert!(sequence.contains("height=7"));
        assert!(sequence.ends_with("\u{7}\n"));
    }

    #[test]
    fn auto_mode_uses_images_only_on_an_iterm_tty_outside_tmux() {
        assert_eq!(
            resolve_mode(ChartMode::Auto, true, Some("iTerm.app"), None, false),
            ChartMode::Image
        );
        assert_eq!(
            resolve_mode(ChartMode::Auto, false, Some("iTerm.app"), None, false),
            ChartMode::Text
        );
        assert_eq!(
            resolve_mode(ChartMode::Auto, true, Some("iTerm.app"), None, true),
            ChartMode::Text
        );
        assert_eq!(
            resolve_mode(ChartMode::Image, false, None, None, true),
            ChartMode::Image
        );
    }

    #[test]
    fn text_fallback_keeps_every_series_named() {
        let rendered = render_text(&parse(TABLE)[0], 60, 5);
        assert!(rendered.contains("Reveal latency"));
        assert!(rendered.contains("p50"));
        assert!(rendered.contains("p95"));
    }

    #[test]
    fn axis_font_renders_digits_and_skips_unknown_glyphs() {
        assert!(glyph('0').is_some());
        assert!(glyph('.').is_some());
        assert!(glyph('-').is_some());
        assert!(glyph('x').is_none());
        assert_eq!(text_width("12", 1), 2 * (GLYPH_W as i32 + 1));
        let mut raster = Raster::new(24, 8);
        raster.text(0, 0, "12", 1, AXIS);
        let drawn = raster
            .pixels
            .chunks_exact(BPP)
            .filter(|px| Raster::rgb_eq(px, AXIS))
            .count();
        assert!(drawn > 0, "digits should light up axis-colored pixels");
    }

    #[test]
    fn text_histogram_labels_both_axes() {
        let charts = parse(
            "## Chart: Latency distribution\n\
             type: histogram\n\
             | milliseconds |\n\
             | ---: |\n\
             | 10 |\n| 12 |\n| 18 |\n| 31 |\n",
        );
        let rendered = render_text(&charts[0], 60, 8);
        assert!(rendered.contains("x:range"));
        assert!(rendered.contains("y:count"));
    }

    #[test]
    fn text_value_chart_caption_shows_the_value_axis_range() {
        let rendered = render_text(&parse(TABLE)[0], 72, 5);
        assert!(rendered.contains("· y "), "value charts must name the y range");
        assert!(rendered.contains("10")); // min across p50/p95
        assert!(rendered.contains("31")); // max across p50/p95
    }

    #[test]
    fn image_chart_draws_axis_label_pixels() {
        let raster = draw_chart(&parse(TABLE)[0], 320, 160);
        let axis_px = raster
            .pixels
            .chunks_exact(BPP)
            .filter(|px| Raster::rgb_eq(px, AXIS))
            .count();
        assert!(axis_px > 0, "image chart should render axis-label pixels");
        let transparent = raster
            .pixels
            .chunks_exact(BPP)
            .filter(|px| px[3] == 0)
            .count();
        assert!(
            transparent > 0,
            "chart field must be transparent so the terminal bg shows through"
        );
    }
}
