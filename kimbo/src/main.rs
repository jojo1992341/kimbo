#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logic;

use eframe::egui::{self, Color32, RichText, Sense, Stroke, Vec2};
use eframe::wgpu;
use logic::{compute, State};
use std::sync::Arc;
use std::time::{Duration, Instant};

const CLAIRE: Color32 = Color32::from_rgb(0xef, 0xe6, 0xc4);
const FONCEE: Color32 = Color32::from_rgb(0xa8, 0x94, 0x60);
const NOMS: [&str; 2] = ["Claire", "Foncée"];

fn couleur(c: u8) -> Color32 {
    if c == 0 { CLAIRE } else { FONCEE }
}

fn carre(ui: &mut egui::Ui, c: u8, taille: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(taille), Sense::hover());
    ui.painter().rect_filled(rect, 4.0, couleur(c));
    ui.painter()
        .rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_black_alpha(90)), egui::StrokeKind::Inside);
}

fn compteur(ui: &mut egui::Ui, val: &mut u32) {
    if ui.button("  +  ").clicked() && *val < 30 {
        *val += 1;
    }
    ui.label(RichText::new(format!("{:>2}", *val)).strong().size(17.0));
    if ui.button("  −  ").clicked() && *val > 0 {
        *val -= 1;
    }
}

/// Rend à Windows les pages mémoire inutilisées (réduit la RAM affichée par le Gestionnaire des tâches).
#[cfg(windows)]
fn liberer_memoire() {
    use windows_sys::Win32::System::{ProcessStatus::K32EmptyWorkingSet, Threading::GetCurrentProcess};
    unsafe {
        K32EmptyWorkingSet(GetCurrentProcess());
    }
}
#[cfg(not(windows))]
fn liberer_memoire() {}

struct App {
    s: State,
    debut: Instant,
    purge_faite: bool,
    avait_focus: bool,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        // Purge mémoire : 1,5 s après le démarrage, puis à chaque perte de focus
        let focus = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if (!self.purge_faite && self.debut.elapsed() > Duration::from_millis(1500))
            || (self.avait_focus && !focus)
        {
            liberer_memoire();
            self.purge_faite = true;
        }
        self.avait_focus = focus;
        if !self.purge_faite {
            ctx.request_repaint_after(Duration::from_millis(1600));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Kimbo : sur quelle couleur de case me placer ?");
            ui.add_space(6.0);

            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.s.idol, "Idole");
                    ui.checkbox(&mut self.s.statue, "Succès Statue (aucun déplacement)");
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Case du Kimbo");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // ordre inversé car right_to_left
                        for c in (0..2u8).rev() {
                            if ui.selectable_label(self.s.k == c, NOMS[c as usize]).clicked() {
                                self.s.k = c;
                            }
                            carre(ui, c, 14.0);
                        }
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Case du Disciple");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for c in (0..2u8).rev() {
                            if ui.selectable_label(self.s.d == c, NOMS[c as usize]).clicked() {
                                self.s.d = c;
                            }
                            carre(ui, c, 14.0);
                        }
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Glyphes déclenchés au tour d’avant ?");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.selectable_label(self.s.o == 0, "Non").clicked() {
                            self.s.o = 0;
                        }
                        if ui.selectable_label(self.s.o == 1, "Oui").clicked() {
                            self.s.o = 1;
                        }
                    });
                });
                if self.s.idol {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Distance moi ↔ Kimbo");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            compteur(ui, &mut self.s.dk);
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Distance moi ↔ Disciple");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            compteur(ui, &mut self.s.dd);
                        });
                    });
                } else {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Ma case");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            for c in (0..2u8).rev() {
                                if ui.selectable_label(self.s.m == c, NOMS[c as usize]).clicked() {
                                    self.s.m = c;
                                }
                                carre(ui, c, 14.0);
                            }
                        });
                    });
                }
            });

            ui.add_space(8.0);
            resultat(ui, &self.s);
        });
    }
}

fn resultat(ui: &mut egui::Ui, s: &State) {
    let vert = Color32::from_rgb(0x4c, 0xaf, 0x50);
    let rouge = Color32::from_rgb(0xd9, 0x53, 0x4f);
    let orange = Color32::from_rgb(0xe8, 0xa3, 0x3a);

    let Some(r) = compute(s) else {
        egui::Frame::group(ui.style()).stroke(Stroke::new(2.0_f32, orange)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Infos incohérentes").size(22.0).strong());
                ui.weak("Vérifie les couleurs et les distances.");
            });
        });
        return;
    };

    let bien_place = r.me == r.safe;
    let mut bordure = if bien_place { vert } else { rouge };
    if s.statue && r.pulls > 0 {
        bordure = orange;
    }
    let qui = |k: bool, d: bool| {
        format!("{}{}{}", if k { "Kimbo" } else { "" }, if k && d { " et du " } else { "" }, if d { "Disciple" } else { "" })
    };

    egui::Frame::group(ui.style()).stroke(Stroke::new(2.0_f32, bordure)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() / 2.0 - 120.0).max(0.0));
                ui.label(RichText::new(if s.statue { "Reste sur" } else { "Place-toi sur" }).size(22.0).strong());
                carre(ui, r.safe, 28.0);
                ui.label(RichText::new(NOMS[r.safe as usize]).size(22.0).strong());
            });
            if bien_place {
                ui.label(
                    RichText::new(format!(
                        "Tu es bien placé : ne bouge pas{}.",
                        if s.statue { "" } else { " (ou déplace-toi d’un nombre pair de cases)" }
                    ))
                    .color(vert),
                );
            } else {
                ui.label(RichText::new("Déplace-toi d’un nombre IMPAIR de cases.").color(rouge).strong());
            }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.weak("Ta case :");
                carre(ui, r.me, 12.0);
                ui.weak(format!("· Frappe le Kimbo en {}", r.duo));
            });
            ui.horizontal_wrapped(|ui| {
                ui.weak("Après leurs déplacements : Kimbo");
                carre(ui, r.k2, 12.0);
                ui.weak("· Disciple");
                carre(ui, r.d2, 12.0);
            });
            if r.statue_push {
                ui.weak("Statue : pousse le Kimbo d’un nombre impair de cases pour qu’il soit sur un glyphe.");
            }
            if r.pulls > 0 {
                let c = qui(r.pull_k, r.pull_d);
                if s.statue {
                    ui.colored_label(
                        orange,
                        format!("À plus de 8 cases du {c} : l’attirance de l’idole te ferait quitter ta case (succès Statue raté). Reste à 8 cases ou moins."),
                    );
                } else {
                    ui.weak(format!(
                        "Plus de 8 cases du {c} : {} d’une case. {}",
                        if r.pulls > 1 { "chacun t’attire" } else { "il t’attire" },
                        if r.pulls % 2 == 1 {
                            "Ta couleur s’inverse : déjà pris en compte."
                        } else {
                            "Les deux attirances s’annulent : déjà pris en compte."
                        }
                    ));
                }
            }
        });
    });
}

/// Choisit DirectX 12 si un adaptateur est disponible, sinon OpenGL.
fn choisir_backend() -> wgpu::Backends {
    let dx12 = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::DX12,
        ..Default::default()
    });
    if dx12.enumerate_adapters(wgpu::Backends::DX12).is_empty() {
        wgpu::Backends::GL
    } else {
        wgpu::Backends::DX12
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Kimbo : sur quelle case me placer ?")
            .with_inner_size([440.0, 560.0])
            .with_min_inner_size([380.0, 480.0])
            .with_always_on_top() // toujours au-dessus des autres fenêtres
            .with_icon(egui::IconData {
                rgba: include_bytes!("../assets/icon_64.rgba").to_vec(),
                width: 64,
                height: 64,
            }),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: {
            let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::default();
            // DX12 si disponible, sinon OpenGL
            setup.instance_descriptor.backends = choisir_backend();
            // GPU économe (intégré si présent) : moins de mémoire pilote
            setup.power_preference = wgpu::PowerPreference::LowPower;
            // Allocation GPU orientée mémoire plutôt que performance
            setup.device_descriptor = Arc::new(|adapter| {
                let base = if adapter.get_info().backend == wgpu::Backend::Gl {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                };
                wgpu::DeviceDescriptor {
                    label: Some("kimbo device"),
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits { max_texture_dimension_2d: 8192, ..base },
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                }
            });
            eframe::egui_wgpu::WgpuConfiguration {
                wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(setup),
                desired_maximum_frame_latency: Some(1), // moins de buffers d'affichage
                ..Default::default()
            }
        },
        ..Default::default()
    };
    eframe::run_native(
        "Kimbo",
        options,
        Box::new(|_cc| {
            Ok(Box::new(App {
                s: State::default(),
                debut: Instant::now(),
                purge_faite: false,
                avait_focus: true,
            }))
        }),
    )
}
