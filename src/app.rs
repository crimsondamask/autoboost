use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver, Sender};

use eframe::egui_glow::check_for_gl_error;
use egui::style::Selection;
use egui::{Color32, CornerRadius, FontData, Stroke, TextEdit, Vec2, Visuals};
use rseip::client::ab_eip::*;
use rseip::precludes::*;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.

enum ReadChannelMsg {
    Value(f32),
    Error(String),
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    received_msg: ReadChannelMsg,
    #[serde(skip)] // This how you opt-out of serialization of a field
    write_value_channel: Option<Sender<f32>>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    read_value_channel: Option<Receiver<ReadChannelMsg>>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    is_first_frame: bool,
    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            received_msg: ReadChannelMsg::Error("Starting".to_owned()),
            label: "Hello World!".to_owned(),

            write_value_channel: None,
            read_value_channel: None,
            is_first_frame: true,
            value: 2.7,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "custom_font".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/plex.ttf"
            ))),
            //egui::FontData::from_static(include_bytes!("../assets/dejavu.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "custom_font".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        // Configuring visuals.

        let mut visuals = Visuals::light();
        visuals.selection = Selection {
            bg_fill: Color32::from_rgb(81, 129, 154),
            stroke: Stroke::new(1.0, Color32::WHITE),
        };

        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(180, 180, 180);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(200, 200, 200);
        visuals.widgets.inactive.corner_radius = CornerRadius::ZERO;
        visuals.widgets.noninteractive.corner_radius = CornerRadius::ZERO;
        visuals.widgets.active.corner_radius = CornerRadius::ZERO;
        visuals.widgets.hovered.corner_radius = CornerRadius::ZERO;
        visuals.window_corner_radius = CornerRadius::ZERO;
        visuals.window_fill = Color32::from_rgb(197, 197, 197);
        visuals.menu_corner_radius = CornerRadius::ZERO;
        visuals.panel_fill = Color32::from_rgb(200, 200, 200);
        visuals.striped = true;
        visuals.slider_trailing_fill = true;

        cc.egui_ctx.set_visuals(visuals);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        ctx.request_repaint();

        if self.is_first_frame {
            let (sender, receiver): (Sender<ReadChannelMsg>, Receiver<ReadChannelMsg>) =
                unbounded();
            self.read_value_channel = Some(receiver);
            std::thread::spawn(move || {
                let mut count = 333.3;
                loop {
                    println!("Tick!");

                    sender.send(ReadChannelMsg::Value(count)).unwrap();
                    count += 1.0;
                    std::thread::sleep(Duration::from_secs(2));
                }
            });

            self.is_first_frame = false
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            if let Some(receiver) = &self.read_value_channel {
                if let Ok(channel_msg) = receiver.try_recv() {
                    self.received_msg = channel_msg;
                }
            }
            match &self.received_msg {
                ReadChannelMsg::Value(v) => {
                    ui.heading(format!("{}", v));
                }
                ReadChannelMsg::Error(e) => {
                    ui.heading(e);
                }
            }

            ui.add(
                TextEdit::singleline(&mut self.label)
                    .background_color(Color32::BLACK)
                    .text_color(Color32::WHITE)
                    .min_size(Vec2::new(50.0, 50.0)),
            );
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

fn ab_connect_to_controller(ip: &str) -> Result<rseip::client::Connection<AbEipDriver>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let connection = rt.block_on(AbEipConnection::new_host_lookup(ip, OpenOptions::default()))?;

    Ok(connection)
}

fn ab_read_tag_f32(connection: &mut Connection<AbEipDriver>, tag: &str) -> Result<f32> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let tag = EPath::parse_tag(tag)?;
    let value: TagValue<f32> = rt.block_on(connection.read_tag(tag))?;

    Ok(value.value)
}
fn ab_write_tag_f32(connection: &mut Connection<AbEipDriver>, tag: &str, value: f32) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let tag = EPath::parse_tag(tag)?;
    let value = TagValue {
        tag_type: TagType::Real,
        value,
    };
    rt.block_on(connection.write_tag(tag, value))?;

    Ok(())
}
