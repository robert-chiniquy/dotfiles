use super::{c, is_sep, split_row, ACCENT, RESET};
use std::env;
use std::io::IsTerminal;

const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
// Neon-grit chart palette — rust / gold / dark grit, not vivid neon green.
// Surfaces: transparent field (terminal wallpaper shows through), rust grid.
// Axis copy uses the card accent so the chart reads as part of the same system.
const GRID: [u8; 3] = [78, 42, 24]; // dark rust
const BPP: usize = 4; // RGBA so the chart field is transparent in iTerm
const AXIS_TEXT_SCALE: i32 = 1;

// Multi-stop grit ramp (gold → oxide). With one chart the full ramp is used;
// with N charts the ramp is sliced top→bottom so chart 0 holds the cool/gold
// edge and chart N-1 holds the hot/oxide edge.
const RAMP_STOPS: [[u8; 3]; 5] = [
    [196, 148, 42], // gold
    [168, 92, 36],  // rust amber
    [132, 64, 32],  // dark rust
    [106, 48, 28],  // tar-rust
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LabelKind {
    Category,
    NumericRange,
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
    label_kind: LabelKind,
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
            label_kind: LabelKind::NumericRange,
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
        label_kind: LabelKind::Category,
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
        ChartKind::Histogram if chart.label_kind == LabelKind::NumericRange => {
            " · x:range y:count".to_string()
        }
        ChartKind::Histogram => chart
            .series
            .first()
            .map(|series| format!(" · x:category y:{}", series.name))
            .unwrap_or_else(|| " · x:category y:value".to_string()),
        ChartKind::Sparkline | ChartKind::TimeSeries => match numeric_bounds(chart) {
            Some((min, max)) => format!(" · y {}–{}", format_number(min), format_number(max)),
            None => String::new(),
        },
    }
}

// Linear RGB color ramp: `ramp[0]` at p=0, `ramp[1]` at p=1 (p is clamped).
fn ramp_at(ramp: [[u8; 3]; 2], p: f64) -> [u8; 3] {
    let p = p.clamp(0.0, 1.0);
    [0, 1, 2]
        .map(|k| (ramp[0][k] as f64 + (ramp[1][k] as f64 - ramp[0][k] as f64) * p).round() as u8)
}

// Sample the multi-stop grit ramp at global p ∈ [0, 1].
fn multi_ramp_at(p: f64) -> [u8; 3] {
    let stops = &RAMP_STOPS;
    if stops.len() == 1 {
        return stops[0];
    }
    let p = p.clamp(0.0, 1.0);
    let scaled = p * (stops.len() - 1) as f64;
    let i = (scaled.floor() as usize).min(stops.len() - 2);
    let t = scaled - i as f64;
    ramp_at([stops[i], stops[i + 1]], t)
}

// Map a local [0, 1] within chart `index` of `count` onto the global ramp.
// Chart 0 uses the low edge; the last chart uses the high edge. With count=1
// this is the identity (full ramp).
fn chart_global_p(chart_index: usize, chart_count: usize, local_p: f64) -> f64 {
    let n = chart_count.max(1) as f64;
    let i = (chart_index.min(chart_count.saturating_sub(1))) as f64;
    (i + local_p.clamp(0.0, 1.0)) / n
}

// Color for series `series_index` of `series_count` on chart `chart_index`.
// Series span that chart's slice of the global ramp so consecutive charts
// sit on different parts of the palette.
fn series_color(
    chart_index: usize,
    chart_count: usize,
    series_index: usize,
    series_count: usize,
) -> [u8; 3] {
    let m = series_count.max(1);
    let local = if m == 1 {
        0.5
    } else {
        series_index as f64 / (m - 1) as f64
    };
    multi_ramp_at(chart_global_p(chart_index, chart_count, local))
}

// Per-cell gradient for a histogram bar, drawn from this chart's slice of the
// multi-chart grit ramp. Ramps across this bar's own length.
fn gradient_bar(count: usize, chart_index: usize, chart_count: usize) -> String {
    let mut bar = String::new();
    for i in 0..count {
        let local = if count > 1 {
            i as f64 / (count - 1) as f64
        } else {
            0.0
        };
        let col = multi_ramp_at(chart_global_p(chart_index, chart_count, local));
        bar.push_str(&format!("{}█{}", c(col), RESET));
    }
    bar
}

fn render_text(
    chart: &Chart,
    width: usize,
    budget: usize,
    chart_index: usize,
    chart_count: usize,
) -> String {
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
                    gradient_bar(count, chart_index, chart_count),
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
            let series_take = available.min(5).min(chart.series.len());
            for (series_index, series) in chart.series.iter().take(series_take).enumerate() {
                let graph_width = width.saturating_sub(name_width + 1);
                let graph = trunc_text(&sparkline(&series.values), graph_width);
                let col = series_color(chart_index, chart_count, series_index, series_take);
                lines.push(format!(
                    "{}{:<name_width$}{} {}",
                    c(col),
                    trunc_text(&series.name, name_width),
                    RESET,
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

    // Draw a string with the built-in 5x7 axis font, scaled by `scale`. Only
    // glyphs known to `glyph()` render; anything else advances a blank cell so
    // widths stay predictable.
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
                                    self.pixel(
                                        cx + gx as i32 * scale + dx,
                                        y + ry as i32 * scale + dy,
                                        color,
                                    );
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

const GLYPH_W: usize = 5;
const GLYPH_H: usize = 7;

// Pixel width a `Raster::text` call for `s` at `scale` will occupy.
fn text_width(s: &str, scale: i32) -> i32 {
    s.chars().count() as i32 * (GLYPH_W as i32 + 1) * scale.max(1)
}

// Compact 5x7 bitmap glyphs for numeric and categorical axis labels
// (bit4=left through bit0=right). Lowercase has its own forms so repository
// names keep their natural visual rhythm instead of reading like display type.
fn glyph(ch: char) -> Option<[u8; GLYPH_H]> {
    Some(match ch {
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'J' => [
            0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        'a' => [0, 0, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111],
        'b' => [
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b11110,
        ],
        'c' => [0, 0, 0b01110, 0b10001, 0b10000, 0b10001, 0b01110],
        'd' => [
            0b00001, 0b00001, 0b01101, 0b10011, 0b10001, 0b10001, 0b01111,
        ],
        'e' => [0, 0, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110],
        'f' => [
            0b00110, 0b01001, 0b01000, 0b11100, 0b01000, 0b01000, 0b01000,
        ],
        'g' => [0, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110],
        'h' => [
            0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001,
        ],
        'i' => [0b00100, 0, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110],
        'j' => [0b00010, 0, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100],
        'k' => [
            0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010,
        ],
        'l' => [
            0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'm' => [0, 0, 0b11010, 0b10101, 0b10101, 0b10101, 0b10101],
        'n' => [0, 0, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001],
        'o' => [0, 0, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110],
        'p' => [0, 0, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000],
        'q' => [0, 0, 0b01111, 0b10001, 0b01111, 0b00001, 0b00001],
        'r' => [0, 0, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000],
        's' => [0, 0, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110],
        't' => [
            0b01000, 0b01000, 0b11100, 0b01000, 0b01000, 0b01001, 0b00110,
        ],
        'u' => [0, 0, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101],
        'v' => [0, 0, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'w' => [0, 0, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'x' => [0, 0, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001],
        'y' => [0, 0, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110],
        'z' => [0, 0, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111],
        '.' => [0, 0, 0, 0, 0, 0b00110, 0b00110],
        '-' | '–' => [0, 0, 0, 0b11111, 0, 0, 0],
        '_' => [0, 0, 0, 0, 0, 0, 0b11111],
        '/' => [0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0, 0],
        '…' => [0, 0, 0, 0, 0, 0b10101, 0b10101],
        ' ' => [0; GLYPH_H],
        _ => return None,
    })
}

fn fitted_axis_label(label: &str, max_width: i32, scale: i32) -> String {
    let advance = (GLYPH_W as i32 + 1) * scale.max(1);
    let max_chars = (max_width.max(0) / advance) as usize;
    if max_chars == 0 {
        return String::new();
    }
    if label.chars().count() <= max_chars {
        return label.to_string();
    }
    if max_chars == 1 {
        return "…".to_string();
    }
    format!("{}…", label.chars().take(max_chars - 1).collect::<String>())
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
    if chart.kind == ChartKind::Histogram {
        min = min.min(0.0);
        max = max.max(0.0);
    }
    if (max - min).abs() < f64::EPSILON {
        min -= 1.0;
        max += 1.0;
    }
    Some((min, max))
}

fn draw_chart(
    chart: &Chart,
    width: usize,
    height: usize,
    chart_index: usize,
    chart_count: usize,
) -> Raster {
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
    let scale = AXIS_TEXT_SCALE;
    let y_ticks: [String; 5] =
        std::array::from_fn(|s| format_number(max - (max - min) * s as f64 / 4.0));
    let y_label_px = y_ticks
        .iter()
        .map(|t| text_width(t, scale))
        .max()
        .unwrap_or(0);
    let glyph_px = GLYPH_H as i32 * scale;

    // Reserve margins for labels when there is room; otherwise fall back to the
    // original tight margins so tiny thumbnails still draw the plot.
    let want_left = 6 + y_label_px as usize + 3;
    let want_bottom = glyph_px as usize + 3;
    let labels_fit = width > want_left + 24 && height > want_bottom + 28;
    let left = if labels_fit { want_left } else { 12 };
    let right = width - 10;
    let top = 10usize;
    let bottom = if labels_fit {
        height - 10 - want_bottom
    } else {
        height - 10
    };

    for (step, label) in y_ticks.iter().enumerate() {
        let y = top + (bottom - top) * step / 4;
        raster.line(left as i32, y as i32, right as i32, y as i32, GRID);
        if labels_fit {
            let lx = left as i32 - text_width(label, scale) - 3;
            raster.text(lx.max(0), y as i32 - glyph_px / 2, label, scale, ACCENT);
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
                // Gradient keyed to plot height within this chart's slice of
                // the multi-chart grit ramp: baseline = low edge of the slice,
                // top of the container = high edge. Across N charts the slices
                // abut so the stack reads as one continuous gold→oxide ramp.
                let field = zero.saturating_sub(top).max(1) as f64;
                for y in value_y.min(zero)..value_y.max(zero) + 1 {
                    let local = zero.saturating_sub(y) as f64 / field;
                    let col = multi_ramp_at(chart_global_p(chart_index, chart_count, local));
                    raster.rect(x0, y, x1, y + 1, col);
                }
            }
            if labels_fit {
                let ty = (bottom as i32 + 3).min(height as i32 - glyph_px - 1);
                match chart.label_kind {
                    LabelKind::Category => {
                        let max_label_width = slot.saturating_sub(4) as i32;
                        let label_scale = if scale > 1
                            && chart
                                .labels
                                .iter()
                                .any(|label| text_width(label, scale) > max_label_width)
                        {
                            1
                        } else {
                            scale
                        };
                        for (index, label) in chart.labels.iter().take(count).enumerate() {
                            let x0 = left + index * slot;
                            let x1 = (left + (index + 1) * slot).min(right);
                            let fitted = fitted_axis_label(
                                label,
                                x1.saturating_sub(x0).saturating_sub(4) as i32,
                                label_scale,
                            );
                            if fitted.is_empty() {
                                continue;
                            }
                            let label_width = text_width(&fitted, label_scale);
                            let center = (x0 + x1) as i32 / 2;
                            raster.text(center - label_width / 2, ty, &fitted, label_scale, ACCENT);
                        }
                    }
                    LabelKind::NumericRange => {
                        // Raw-observation histograms use numeric bins. Endpoint
                        // labels keep the full range readable without crowding.
                        if let (Some(first), Some(last)) =
                            (chart.labels.first(), chart.labels.last())
                        {
                            let lo = first.split('–').next().unwrap_or(first).trim();
                            let hi = last.rsplit('–').next().unwrap_or(last).trim();
                            raster.text(left as i32, ty, lo, scale, ACCENT);
                            raster.text(
                                right as i32 - text_width(hi, scale),
                                ty,
                                hi,
                                scale,
                                ACCENT,
                            );
                        }
                    }
                }
            }
        }
        ChartKind::Sparkline | ChartKind::TimeSeries => {
            let series_take = chart.series.len().min(RAMP_STOPS.len());
            for (series_index, series) in chart.series.iter().take(series_take).enumerate() {
                let color = series_color(chart_index, chart_count, series_index, series_take);
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

fn render_png(
    chart: &Chart,
    width: usize,
    height: usize,
    chart_index: usize,
    chart_count: usize,
) -> Vec<u8> {
    let width = width.clamp(32, 960);
    let height = height.clamp(24, 240);
    let raster = draw_chart(chart, width, height, chart_index, chart_count);
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

fn iterm_image_sequence(png: &[u8], title: &str, width: usize, height: usize) -> String {
    let name = base64(format!("{}.png", title).as_bytes());
    format!(
        "\x1b]1337;File=name={};size={};width={};height={};preserveAspectRatio=0;inline=1:{}\x07",
        name,
        png.len(),
        width,
        height,
        base64(png)
    )
}

fn iterm_image(png: &[u8], title: &str, width: usize, height: usize) -> String {
    format!("{}\n", iterm_image_sequence(png, title, width, height))
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

pub(crate) fn render_image_sidecar_rows(
    charts: &[Chart],
    width: usize,
    mut budget: usize,
) -> Vec<String> {
    let mut chart_count = 0usize;
    let mut remaining = budget;
    for _ in charts {
        let image_rows = remaining.saturating_sub(1).min(8);
        if image_rows < 3 {
            break;
        }
        chart_count += 1;
        remaining = remaining.saturating_sub(image_rows + 1);
    }

    let mut rows = Vec::new();
    for (chart_index, chart) in charts.iter().take(chart_count).enumerate() {
        let image_rows = budget.saturating_sub(1).min(8);
        rows.push(trunc_text(
            &format!(
                "CHART {} · {}{}",
                chart.title,
                chart.kind.label(),
                axis_caption(chart)
            ),
            width,
        ));
        let pixel_width = (width * 8).clamp(320, 960);
        let pixel_height = (image_rows * 20).clamp(60, 240);
        let png = render_png(chart, pixel_width, pixel_height, chart_index, chart_count);
        rows.push(format!(
            "\x1b7{}\x1b8",
            iterm_image_sequence(&png, &chart.title, width, image_rows)
        ));
        rows.resize(rows.len() + image_rows.saturating_sub(1), String::new());
        budget = budget.saturating_sub(image_rows + 1);
    }
    rows
}

pub(crate) fn render(charts: &[Chart], mode: ChartMode, width: usize, mut budget: usize) -> String {
    let mut output = String::new();
    // Count only charts we can actually paint under the budget so the ramp
    // stretch matches what the user sees (not hidden overflow charts).
    let chart_count = match mode {
        ChartMode::Off => 0,
        ChartMode::Image => {
            let mut n = 0usize;
            let mut b = budget;
            for _ in charts {
                let image_rows = b.saturating_sub(1).min(8);
                if image_rows < 3 {
                    break;
                }
                n += 1;
                b = b.saturating_sub(image_rows + 1);
            }
            n.max(1)
        }
        ChartMode::Text | ChartMode::Auto => {
            let mut n = 0usize;
            let mut b = budget;
            for chart in charts {
                if b < 2 {
                    break;
                }
                let rows = text_rows(chart).min(b);
                if rows == 0 {
                    break;
                }
                n += 1;
                b = b.saturating_sub(rows);
            }
            n.max(1)
        }
    };
    for (chart_index, chart) in charts.iter().enumerate() {
        if budget < 2 {
            break;
        }
        match mode {
            ChartMode::Off => break,
            ChartMode::Text | ChartMode::Auto => {
                let rows = text_rows(chart).min(budget);
                output.push_str(&render_text(chart, width, rows, chart_index, chart_count));
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
                let png = render_png(chart, pixel_width, pixel_height, chart_index, chart_count);
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

    fn rgb_eq(px: &[u8], color: [u8; 3]) -> bool {
        px.len() >= 4 && px[0] == color[0] && px[1] == color[1] && px[2] == color[2] && px[3] != 0
    }

    // Two-stop ramp: base at p=0, tip at p=1, linear and clamped.
    #[test]
    fn ramp_at_interpolates_and_clamps() {
        let lo = RAMP_STOPS[0];
        let hi = RAMP_STOPS[RAMP_STOPS.len() - 1];
        assert_eq!(ramp_at([lo, hi], 0.0), lo);
        assert_eq!(ramp_at([lo, hi], 1.0), hi);
        assert_eq!(ramp_at([lo, hi], -1.0), lo, "p below 0 clamps to base");
        assert_eq!(ramp_at([lo, hi], 2.0), hi, "p above 1 clamps to tip");
        let mid = ramp_at([lo, hi], 0.5);
        for k in 0..3 {
            let (a, b) = (lo[k].min(hi[k]), lo[k].max(hi[k]));
            assert!(
                mid[k] >= a && mid[k] <= b,
                "midpoint channel {k} out of range"
            );
        }
    }

    // Multi-chart stretch: chart 0 owns the cool half, chart 1 the hot half.
    // Failure hypothesis: both charts still sample the full ramp, so mid colors
    // collide and consecutive charts look identical.
    #[test]
    fn multi_chart_ramp_slices_do_not_overlap() {
        assert_eq!(multi_ramp_at(0.0), RAMP_STOPS[0]);
        assert_eq!(multi_ramp_at(1.0), RAMP_STOPS[RAMP_STOPS.len() - 1]);
        // chart 0 local=1 abuts chart 1 local=0
        assert!((chart_global_p(0, 2, 1.0) - chart_global_p(1, 2, 0.0)).abs() < 1e-9);
        let top_hi = multi_ramp_at(chart_global_p(0, 2, 1.0));
        let bot_lo = multi_ramp_at(chart_global_p(1, 2, 0.0));
        assert_eq!(top_hi, bot_lo, "adjacent slices must join continuously");
        // mid of chart 0 is cooler (higher gold channel) than mid of chart 1
        let top_mid = multi_ramp_at(chart_global_p(0, 2, 0.5));
        let bot_mid = multi_ramp_at(chart_global_p(1, 2, 0.5));
        assert_ne!(
            top_mid, bot_mid,
            "stacked charts must not share the same mid color"
        );
        // single chart still uses the full ramp
        assert_eq!(multi_ramp_at(chart_global_p(0, 1, 0.0)), RAMP_STOPS[0]);
        assert_eq!(
            multi_ramp_at(chart_global_p(0, 1, 1.0)),
            RAMP_STOPS[RAMP_STOPS.len() - 1]
        );
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
        let png = render_png(chart, 320, 96, 0, 1);
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
        let png = render_png(&parse(TABLE)[0], 320, 96, 0, 1);
        let sequence = iterm_image(&png, "Reveal latency", 80, 7);
        assert!(sequence.starts_with("\u{1b}]1337;File="));
        assert!(sequence.contains("inline=1"));
        assert!(sequence.contains("width=80"));
        assert!(sequence.contains("height=7"));
        assert!(sequence.ends_with("\u{7}\n"));
    }

    #[test]
    fn image_sidecar_rows_reserve_a_caption_and_cursor_safe_image() {
        let rows = render_image_sidecar_rows(&parse(TABLE), 40, 9);

        assert_eq!(rows.len(), 9);
        assert!(rows[0].starts_with("CHART Reveal latency"));
        assert!(rows[1].starts_with("\x1b7\x1b]1337;File="));
        assert!(rows[1].contains("width=40"));
        assert!(rows[1].contains("height=8"));
        assert!(rows[1].ends_with("\x07\x1b8"));
        assert!(rows[2..].iter().all(String::is_empty));
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
        let rendered = render_text(&parse(TABLE)[0], 60, 5, 0, 1);
        assert!(rendered.contains("Reveal latency"));
        assert!(rendered.contains("p50"));
        assert!(rendered.contains("p95"));
    }

    #[test]
    fn axis_font_renders_numeric_and_distinct_mixed_case_glyphs() {
        assert_eq!((GLYPH_W, GLYPH_H, AXIS_TEXT_SCALE), (5, 7, 1));
        assert!(glyph('0').is_some());
        assert!(glyph('.').is_some());
        assert!(glyph('-').is_some());
        assert!(glyph('x').is_some());
        assert!(glyph('_').is_some());
        assert!(glyph('🙂').is_none());
        assert_ne!(
            glyph('x'),
            glyph('X'),
            "repository labels should retain their source case"
        );
        assert_eq!(text_width("12", 1), 2 * (GLYPH_W as i32 + 1));
        let mut raster = Raster::new(24, 8);
        raster.text(0, 0, "12", 1, crate::ACCENT);
        let drawn = raster
            .pixels
            .chunks_exact(BPP)
            .filter(|px| rgb_eq(px, crate::ACCENT))
            .count();
        assert!(drawn > 0, "digits should light up card-accent pixels");
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
        let rendered = render_text(&charts[0], 60, 8, 0, 1);
        assert!(rendered.contains("x:range"));
        assert!(rendered.contains("y:count"));
    }

    #[test]
    fn categorical_histogram_caption_names_category_and_value_axes() {
        let chart = &parse(
            "## Chart: Tokens\n\
             type: histogram\n\
             | repository | tokens (M) |\n\
             | --- | ---: |\n\
             | occult | 2369.5 |\n",
        )[0];
        assert_eq!(axis_caption(chart), " · x:category y:tokens (M)");
    }

    #[test]
    fn histogram_value_axis_includes_zero_baseline() {
        let chart = &parse(
            "## Chart: Tokens\n\
             type: histogram\n\
             | repository | tokens |\n\
             | --- | ---: |\n\
             | occult | 2400 |\n\
             | research | 115 |\n",
        )[0];
        assert_eq!(numeric_bounds(chart), Some((0.0, 2400.0)));
    }

    #[test]
    fn text_value_chart_caption_shows_the_value_axis_range() {
        let rendered = render_text(&parse(TABLE)[0], 72, 5, 0, 1);
        assert!(
            rendered.contains("· y "),
            "value charts must name the y range"
        );
        assert!(rendered.contains("10")); // min across p50/p95
        assert!(rendered.contains("31")); // max across p50/p95
    }

    #[test]
    fn image_chart_draws_y_axis_labels_in_card_accent() {
        let raster = draw_chart(&parse(TABLE)[0], 320, 160, 0, 1);
        let axis_px = raster
            .pixels
            .chunks_exact(BPP)
            .filter(|px| rgb_eq(px, crate::ACCENT))
            .count();
        assert!(
            axis_px > 0,
            "image chart should render y-axis labels in the card accent"
        );
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

    #[test]
    fn image_histogram_labels_every_category_column_in_card_accent() {
        let chart = &parse(
            "## Chart: Tokens by repository\n\
             type: histogram\n\
             | repository | tokens |\n\
             | --- | ---: |\n\
             | occult | 2400 |\n\
             | latchkey-project | 1700 |\n\
             | tpm | 170 |\n\
             | research | 115 |\n",
        )[0];
        let (width, height) = (640usize, 160usize);
        let raster = draw_chart(chart, width, height, 0, 1);

        // Mirror the renderer's plot geometry, then require label ink in every
        // categorical slot below the baseline, including interior columns.
        let (min, max) = numeric_bounds(chart).unwrap();
        let scale = AXIS_TEXT_SCALE;
        let y_ticks: [String; 5] =
            std::array::from_fn(|step| format_number(max - (max - min) * step as f64 / 4.0));
        let y_label_px = y_ticks
            .iter()
            .map(|tick| text_width(tick, scale))
            .max()
            .unwrap();
        let glyph_px = GLYPH_H * scale as usize;
        let left = 6 + y_label_px as usize + 3;
        let right = width - 10;
        let bottom = height - 10 - (glyph_px + 3);
        let label_top = bottom + 3;
        let slot = (right - left) / chart.labels.len();
        for (index, label) in chart.labels.iter().enumerate() {
            let x0 = left + index * slot;
            let x1 = if index + 1 == chart.labels.len() {
                right
            } else {
                left + (index + 1) * slot
            };
            let ink = (label_top..raster.height)
                .flat_map(|y| (x0..x1).map(move |x| (x, y)))
                .filter(|(x, y)| {
                    let offset = (y * raster.width + x) * BPP;
                    rgb_eq(&raster.pixels[offset..offset + BPP], crate::ACCENT)
                })
                .count();
            assert!(ink > 0, "category {label:?} has no x-axis label ink");
        }
        let y_axis_ink = raster
            .pixels
            .chunks_exact(BPP)
            .filter(|px| rgb_eq(px, crate::ACCENT))
            .count();
        assert!(y_axis_ink > 0, "numeric ticks must use the card accent");
    }

    #[test]
    fn image_raw_histogram_range_labels_use_card_accent() {
        let chart = &parse(
            "## Chart: Latency distribution\n\
             type: histogram\n\
             | milliseconds |\n\
             | ---: |\n\
             | 10 |\n| 12 |\n| 18 |\n| 31 |\n",
        )[0];
        let (width, height) = (480usize, 160usize);
        let raster = draw_chart(chart, width, height, 0, 1);

        // The range endpoints occupy the band below the bottom y tick. Scan
        // beneath that tick so its cyan ink cannot satisfy this assertion.
        let glyph_px = GLYPH_H * AXIS_TEXT_SCALE as usize;
        let bottom = height - 10 - (glyph_px + 3);
        let label_top = bottom + 3;
        let accent_ink = ((label_top + 1)..(label_top + glyph_px))
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .filter(|(x, y)| {
                let offset = (y * raster.width + x) * BPP;
                rgb_eq(&raster.pixels[offset..offset + BPP], crate::ACCENT)
            })
            .count();
        assert!(
            accent_ink > 0,
            "numeric range and y-axis labels must use the card accent"
        );
    }
}
