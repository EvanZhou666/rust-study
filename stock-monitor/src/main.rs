mod config;
mod stock;

use std::{
    collections::HashMap,
    fs,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use config::{load_config, save_config, AppConfig};
use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, Pos2, Rect, RichText, Sense, Stroke, Vec2,
};
use stock::{Candle, SinaProvider, StockQuote};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([300.0, 122.0])
            .with_min_inner_size([280.0, 122.0])
            .with_always_on_top()
            .with_decorations(false)
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "Stock Monitor",
        options,
        Box::new(|cc| {
            configure_chinese_fonts(&cc.egui_ctx);
            Box::new(StockMonitorApp::new(cc))
        }),
    )
}

struct StockMonitorApp {
    config: AppConfig,
    quotes: HashMap<String, StockQuote>,
    candles: HashMap<String, Vec<Candle>>,
    input_symbol: String,
    current_index: usize,
    last_rotation: Instant,
    last_quote_refresh: Instant,
    pending_candle_symbol: Option<String>,
    error: Option<String>,
    tx: Sender<WorkerRequest>,
    rx: Receiver<WorkerEvent>,
}

impl StockMonitorApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = load_config();
        let (tx, worker_rx) = mpsc::channel();
        let (worker_tx, rx) = mpsc::channel();

        spawn_worker(worker_rx, worker_tx);

        let app = Self {
            config,
            quotes: HashMap::new(),
            candles: HashMap::new(),
            input_symbol: String::new(),
            current_index: 0,
            last_rotation: Instant::now(),
            last_quote_refresh: Instant::now() - Duration::from_secs(60),
            pending_candle_symbol: None,
            error: None,
            tx,
            rx,
        };

        app.request_quotes();
        app
    }

    fn request_quotes(&self) {
        let _ = self
            .tx
            .send(WorkerRequest::FetchQuotes(self.config.symbols.clone()));
    }

    fn request_candles(&mut self, symbol: String) {
        if self.pending_candle_symbol.as_ref() == Some(&symbol)
            || self.candles.contains_key(&symbol)
        {
            return;
        }

        self.pending_candle_symbol = Some(symbol.clone());
        let _ = self.tx.send(WorkerRequest::FetchCandles(symbol));
    }

    fn drain_worker_events(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                WorkerEvent::Quotes(result) => match result {
                    Ok(quotes) => {
                        self.error = None;
                        for quote in quotes {
                            self.quotes.insert(quote.symbol.clone(), quote);
                        }
                    }
                    Err(err) => self.error = Some(err),
                },
                WorkerEvent::Candles { symbol, result } => {
                    self.pending_candle_symbol = None;
                    match result {
                        Ok(candles) => {
                            self.error = None;
                            self.candles.insert(symbol, candles);
                        }
                        Err(err) => self.error = Some(err),
                    }
                }
            }
        }
    }

    fn tick(&mut self, ctx: &egui::Context) {
        self.drain_worker_events();

        if self.last_quote_refresh.elapsed()
            >= Duration::from_secs(self.config.quote_refresh_secs.max(3))
        {
            self.last_quote_refresh = Instant::now();
            self.request_quotes();
        }

        if !self.config.symbols.is_empty()
            && self.last_rotation.elapsed()
                >= Duration::from_secs(self.config.rotation_interval_secs.max(2))
        {
            self.last_rotation = Instant::now();
            self.current_index = (self.current_index + 1) % self.config.symbols.len();
        }

        ctx.request_repaint_after(Duration::from_millis(250));
    }

    fn current_symbol(&self) -> Option<&str> {
        if self.config.symbols.is_empty() {
            return None;
        }
        self.config
            .symbols
            .get(self.current_index % self.config.symbols.len())
            .map(String::as_str)
    }

    fn add_symbol(&mut self) {
        let symbol = normalize_symbol(&self.input_symbol);
        if symbol.is_empty() || self.config.symbols.iter().any(|item| item == &symbol) {
            return;
        }

        self.config.symbols.push(symbol);
        self.input_symbol.clear();
        self.persist_and_refresh();
    }

    fn remove_symbol(&mut self, symbol: &str) {
        self.config.symbols.retain(|item| item != symbol);
        self.quotes.remove(symbol);
        self.candles.remove(symbol);
        self.current_index = self
            .current_index
            .min(self.config.symbols.len().saturating_sub(1));
        self.persist_and_refresh();
    }

    fn persist_and_refresh(&mut self) {
        if let Err(err) = save_config(&self.config) {
            self.error = Some(err.to_string());
        }
        self.request_quotes();
    }
}

impl eframe::App for StockMonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick(ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(22, 24, 28, 232))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(64, 70, 78)))
                    .rounding(8.0)
                    .inner_margin(egui::Margin::same(10.0)),
            )
            .show(ctx, |ui| {
                let hovered = ui.rect_contains_pointer(ui.max_rect());
                if hovered {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([440.0, 352.0].into()));
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([300.0, 122.0].into()));
                }

                self.show_title_bar(ui, ctx);
                ui.add_space(4.0);
                self.show_quote(ui);

                if hovered {
                    ui.add_space(8.0);
                    self.show_chart(ui);
                    ui.separator();
                    self.show_manager(ui);
                }
            });
    }
}

impl StockMonitorApp {
    fn show_title_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            let drag_width = (ui.available_width() - 30.0).max(80.0);
            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(drag_width, 24.0), Sense::click_and_drag());

            ui.painter().text(
                rect.left_center(),
                egui::Align2::LEFT_CENTER,
                "A 股悬浮看板",
                egui::FontId::proportional(13.0),
                Color32::from_rgb(180, 188, 198),
            );

            if response.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            let close = ui.add_sized(
                [24.0, 24.0],
                egui::Button::new(RichText::new("x").color(Color32::from_rgb(220, 226, 232))),
            );
            if close.clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    fn show_quote(&mut self, ui: &mut egui::Ui) {
        let Some(symbol) = self.current_symbol() else {
            ui.label(RichText::new("添加一只股票开始").color(Color32::WHITE));
            return;
        };

        if let Some(quote) = self.quotes.get(symbol) {
            let up = quote.change() >= 0.0;
            let color = if up {
                Color32::from_rgb(236, 86, 86)
            } else {
                Color32::from_rgb(72, 188, 125)
            };

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&quote.name)
                        .color(Color32::WHITE)
                        .size(18.0)
                        .strong(),
                );
                ui.label(RichText::new(&quote.symbol).color(Color32::GRAY).size(12.0));
            });
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{:.2}", quote.price))
                        .color(color)
                        .size(24.0)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!(
                        "{:+.2}  {:+.2}%",
                        quote.change(),
                        quote.change_percent()
                    ))
                    .color(color)
                    .size(15.0),
                );
            });
            ui.label(
                RichText::new(format!(
                    "{} {}  开 {:.2}  高 {:.2}  低 {:.2}  量 {}  额 {}",
                    quote.date,
                    quote.time,
                    quote.open,
                    quote.high,
                    quote.low,
                    format_large(quote.volume as f64),
                    format_large(quote.amount)
                ))
                .color(Color32::GRAY)
                .size(11.0),
            );
        } else {
            ui.label(RichText::new(symbol).color(Color32::WHITE).size(18.0));
            ui.label(RichText::new("正在获取新浪行情...").color(Color32::GRAY));
        }
    }

    fn show_chart(&mut self, ui: &mut egui::Ui) {
        let Some(symbol) = self.current_symbol().map(str::to_owned) else {
            return;
        };

        self.request_candles(symbol.clone());

        if let Some(candles) = self.candles.get(&symbol) {
            if candles.is_empty() {
                ui.label(RichText::new("暂无当日走势").color(Color32::GRAY));
                return;
            }

            draw_candles(ui, candles);

            if let (Some(first), Some(last)) = (candles.first(), candles.last()) {
                ui.label(
                    RichText::new(format!(
                        "{} -> {}  收 {:.2}  量 {}",
                        first.time,
                        last.time,
                        last.close,
                        format_large(last.volume as f64)
                    ))
                    .color(Color32::GRAY)
                    .size(11.0),
                );
            }
        } else {
            ui.label(RichText::new("正在加载当日 K 线...").color(Color32::GRAY));
        }
    }

    fn show_manager(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut self.input_symbol);
            if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                self.add_symbol();
            }
            if ui.button("添加").clicked() {
                self.add_symbol();
            }
        });

        let symbols = self.config.symbols.clone();
        ui.horizontal_wrapped(|ui| {
            for symbol in symbols {
                if ui.small_button(format!("{symbol} x")).clicked() {
                    self.remove_symbol(&symbol);
                }
            }
        });

        if let Some(error) = &self.error {
            ui.label(
                RichText::new(error)
                    .color(Color32::from_rgb(255, 172, 72))
                    .size(11.0),
            );
        }
    }
}

enum WorkerRequest {
    FetchQuotes(Vec<String>),
    FetchCandles(String),
}

enum WorkerEvent {
    Quotes(Result<Vec<StockQuote>, String>),
    Candles {
        symbol: String,
        result: Result<Vec<Candle>, String>,
    },
}

fn spawn_worker(rx: Receiver<WorkerRequest>, tx: Sender<WorkerEvent>) {
    thread::spawn(move || {
        let provider = match SinaProvider::new() {
            Ok(provider) => provider,
            Err(err) => {
                let _ = tx.send(WorkerEvent::Quotes(Err(err.to_string())));
                return;
            }
        };

        while let Ok(request) = rx.recv() {
            match request {
                WorkerRequest::FetchQuotes(symbols) => {
                    let result = provider
                        .fetch_quotes(&symbols)
                        .map_err(|err| err.to_string());
                    let _ = tx.send(WorkerEvent::Quotes(result));
                }
                WorkerRequest::FetchCandles(symbol) => {
                    let result = provider
                        .fetch_intraday(&symbol)
                        .map_err(|err| err.to_string());
                    let _ = tx.send(WorkerEvent::Candles { symbol, result });
                }
            }
        }
    });
}

fn normalize_symbol(input: &str) -> String {
    let trimmed = input.trim().to_lowercase();
    if trimmed.starts_with("sh") || trimmed.starts_with("sz") || trimmed.starts_with("bj") {
        return trimmed;
    }

    if trimmed.len() != 6 || !trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return trimmed;
    }

    match trimmed.as_bytes()[0] {
        b'6' | b'5' | b'9' => format!("sh{trimmed}"),
        b'0' | b'1' | b'2' | b'3' => format!("sz{trimmed}"),
        b'4' | b'8' => format!("bj{trimmed}"),
        _ => trimmed,
    }
}

fn configure_chinese_fonts(ctx: &egui::Context) {
    let font_candidates = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ];

    let Some(font_bytes) = font_candidates.iter().find_map(|path| fs::read(path).ok()) else {
        return;
    };

    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("cjk".to_owned(), FontData::from_owned(font_bytes));

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, "cjk".to_owned());
    }

    ctx.set_fonts(fonts);
}

fn draw_candles(ui: &mut egui::Ui, candles: &[Candle]) {
    let desired_size = Vec2::new(ui.available_width(), 140.0);
    let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 4.0, Color32::from_rgba_unmultiplied(14, 16, 20, 180));

    let min_price = candles
        .iter()
        .map(|candle| candle.low)
        .fold(f64::INFINITY, f64::min);
    let max_price = candles
        .iter()
        .map(|candle| candle.high)
        .fold(f64::NEG_INFINITY, f64::max);

    if !min_price.is_finite() || !max_price.is_finite() {
        return;
    }

    let price_range = (max_price - min_price).max(0.01);
    let to_y = |price: f64| {
        let rate = (price - min_price) / price_range;
        rect.bottom() - (rate as f32 * rect.height())
    };

    let candle_width = (rect.width() / candles.len() as f32).clamp(3.0, 8.0);
    let gap = candle_width * 0.35;

    for (index, candle) in candles.iter().enumerate() {
        let center_x =
            rect.left() + (index as f32 + 0.5) * rect.width() / candles.len().max(1) as f32;
        let open_y = to_y(candle.open);
        let close_y = to_y(candle.close);
        let high_y = to_y(candle.high);
        let low_y = to_y(candle.low);
        let rising = candle.close >= candle.open;
        let color = if rising {
            Color32::from_rgb(236, 86, 86)
        } else {
            Color32::from_rgb(72, 188, 125)
        };

        painter.line_segment(
            [Pos2::new(center_x, high_y), Pos2::new(center_x, low_y)],
            Stroke::new(1.0, color),
        );

        let half_width = (candle_width - gap).max(1.5) / 2.0;
        let top = open_y.min(close_y);
        let bottom = open_y.max(close_y).max(top + 1.0);
        let body = Rect::from_min_max(
            Pos2::new(center_x - half_width, top),
            Pos2::new(center_x + half_width, bottom),
        );
        painter.rect_filled(body, 1.0, color);
    }

    painter.text(
        rect.left_top() + Vec2::new(6.0, 5.0),
        egui::Align2::LEFT_TOP,
        format!("{max_price:.2}"),
        egui::FontId::monospace(10.0),
        Color32::GRAY,
    );
    painter.text(
        rect.left_bottom() + Vec2::new(6.0, -16.0),
        egui::Align2::LEFT_TOP,
        format!("{min_price:.2}"),
        egui::FontId::monospace(10.0),
        Color32::GRAY,
    );
}

fn format_large(value: f64) -> String {
    if value >= 100_000_000.0 {
        format!("{:.2}e", value / 100_000_000.0)
    } else if value >= 10_000.0 {
        format!("{:.2}w", value / 10_000.0)
    } else {
        format!("{value:.0}")
    }
}
