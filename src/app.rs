use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    input: String,
    output: String,
    input_xml: String,
    output_xml: String,
}

impl Default for App {
    fn default() -> Self {
        let input = "Hello World!".to_owned();
        let output = randomize_string(&input);
        let input_xml = "<Customers><Customer><Number>1</Number><FirstName>Fred</FirstName><LastName>Landis</LastName><Address><Street>Oakstreet</Street><City>Boston</City><ZIP>23320</ZIP><State>MA</State></Address></Customer><Customer><Number>2</Number><FirstName>Michelle</FirstName><LastName>Butler</LastName><Address><Street>First Avenue</Street><City>San-Francisco</City><ZIP>44324</ZIP><State>CA</State></Address></Customer><Customer><Number>3</Number><FirstName>Ted</FirstName><LastName>Little</LastName><Address><Street>Long Way</Street><City>Los-Angeles</City><ZIP>34424</ZIP><State>CA</State></Address></Customer></Customers>".to_owned();
        let output_xml = prettify_xml(&input);
        Self {
            input,
            output,
            input_xml,
            output_xml,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

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

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label("Input String: ");
                ui.add(egui::TextEdit::singleline(&mut self.input).hint_text("Enter text here"));
                ui.label("Output String: ");
                ui.add(
                    egui::TextEdit::singleline(&mut self.output)
                        .hint_text("Randomized text will appear here"),
                );
            });

            ui.horizontal(|ui| {
                ui.centered_and_justified(|ui| {
                    if ui.button("Randomize!").clicked() {
                        self.output = randomize_string(&self.input);
                    }

                    if ui.button("Copy Output").clicked() {
                        ui.output_mut(|o| o.copied_text.clone_from(&self.output));
                    }
                });
            });

            ui.separator();

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label("Input XML:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.input_xml)
                        .hint_text("Enter XML here")
                        .desired_rows(10)
                        .desired_width(f32::INFINITY),
                );
                ui.label("Output XML:");
                ui.add(
                    egui::TextEdit::multiline(&mut self.output_xml)
                        .hint_text("Formatted XML will appear here")
                        .desired_rows(10)
                        .desired_width(f32::INFINITY),
                );
            });

            ui.horizontal(|ui| {
                ui.centered_and_justified(|ui| {
                    if ui.button("Format!").clicked() {
                        self.output_xml = prettify_xml(&self.input_xml);
                    }

                    if ui.button("Copy Output").clicked() {
                        ui.output_mut(|o| o.copied_text.clone_from(&self.output_xml));
                    }
                });
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                source_and_powered_by(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn source_and_powered_by(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(". ");
        ui.add(egui::github_link_file!(
            "https://github.com/aalhendi/string-randomizer",
            "Source code (app)."
        ));
    });
}

fn randomize_string(input: &str) -> String {
    let mut chars: Vec<char> = input.chars().collect();
    let mut rng = thread_rng();
    chars.shuffle(&mut rng);
    chars.into_iter().collect()
}

/// from: lwilli/prettify_xml.rs
/// Prettify XML by adding proper new lines and indentation.
///
/// This uses the quick_xml library (https://github.com/tafia/quick-xml) to read/parse the given
/// XML string and then write it to a string as indented XML. This isn't perfect and most likely
/// not the most efficient.
///
/// One strange behavior is that a closing element tag is not put on a new line if it follows a
/// comment and text, for example:
/// ```
/// <tag2>
///   <!--Comment-->Text</tag2>
/// ```
///
/// Performance:
///   On a large 66K line minified XML document, this takes about 2700ms.
///   For small XMLs, the time is negligible.
pub fn prettify_xml(xml: &str) -> String {
    // let mut buf = Vec::new();

    let mut reader = Reader::from_str(xml);

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        let ev = reader.read_event();

        match ev {
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Ok(event) => writer.write_event(event),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
        .expect("Failed to parse XML");
    }

    let result = std::str::from_utf8(&writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string();

    result
}
