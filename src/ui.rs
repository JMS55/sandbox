use crate::particle::ParticleType;
use crate::sandbox::Sandbox;
use imgui::{Condition, Context, FontSource, Slider, StyleColor, StyleVar, Window as ImWindow};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use pixels::wgpu::*;
use puffin::GlobalProfiler;
use puffin_imgui::ProfilerUi;
use std::time::Duration;
use winit::event::Event;
use winit::window::Window;

pub struct UI {
    imgui: Context,
    imgui_platform: WinitPlatform,
    imgui_renderer: Renderer,

    should_display_ui: bool,
    should_display_fps: bool,
    should_display_profiler: bool,

    was_paused_before_popup: bool,
    profiler_ui: ProfilerUi,
}

impl UI {
    pub fn new(window: &Window, device: &Device, queue: &Queue) -> Self {
        let mut imgui = Context::create();
        let mut imgui_platform = WinitPlatform::init(&mut imgui);
        imgui_platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
        imgui.io_mut().font_global_scale = (1.0 / imgui_platform.hidpi_factor()) as f32;
        imgui.fonts().add_font(&[FontSource::TtfData {
            data: include_bytes!("../Inter-SemiBold.otf"),
            size_pixels: (16.0 * imgui_platform.hidpi_factor()) as f32,
            config: None,
        }]);
        imgui.set_ini_filename(None);
        let imgui_renderer = Renderer::new(
            &mut imgui,
            device,
            queue,
            RendererConfig {
                texture_format: TextureFormat::Bgra8UnormSrgb,
                ..Default::default()
            },
        );

        Self {
            imgui,
            imgui_platform,
            imgui_renderer,

            should_display_ui: true,
            should_display_fps: cfg!(debug_assertions),
            should_display_profiler: false,

            was_paused_before_popup: false,
            profiler_ui: ProfilerUi::default(),
        }
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64, device: &Device, queue: &Queue) {
        self.imgui.io_mut().font_global_scale = (1.0 / scale_factor) as f32;
        self.imgui.fonts().clear_input_data();
        self.imgui.fonts().add_font(&[FontSource::TtfData {
            data: include_bytes!("../Inter-SemiBold.otf"),
            size_pixels: (16.0 * scale_factor) as f32,
            config: None,
        }]);
        self.imgui_renderer
            .reload_font_texture(&mut self.imgui, device, queue);
    }

    pub fn toggle_display_ui(&mut self) {
        self.should_display_ui = !self.should_display_ui;
    }

    pub fn toggle_display_fps(&mut self) {
        self.should_display_fps = !self.should_display_fps;
    }

    pub fn toggle_display_profiler(&mut self) {
        self.should_display_profiler = !self.should_display_profiler;
        puffin::set_scopes_on(self.should_display_profiler);
    }

    pub fn start_of_frame(&mut self, time_since_last_frame: Duration) {
        if self.should_display_profiler {
            GlobalProfiler::lock().new_frame();
        }

        self.imgui.io_mut().update_delta_time(time_since_last_frame);
    }

    pub fn prepare_render(&mut self, window: &Window) {
        self.imgui_platform
            .prepare_frame(self.imgui.io_mut(), window)
            .expect("Failed to prepare UI frame");
    }

    pub fn render(
        &mut self,
        sandbox: &mut Sandbox,
        selected_particle: &mut Option<ParticleType>,
        brush_size: &mut u8,
        game_paused: &mut bool,

        window: &Window,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        render_texture: &TextureView,
    ) {
        let ui = self.imgui.frame();

        // Function to create particle selection buttons
        let mut button_x = 0.0;
        let mut particle_selector_button = |text: &str, ptype: Option<ParticleType>| {
            ui.set_cursor_pos([
                button_x,
                if ptype == *selected_particle {
                    0.0
                } else {
                    8.0
                },
            ]);
            button_x += if ptype == *selected_particle {
                108.0
            } else {
                93.0
            };

            let button_color = ptype_ui_color(ptype);
            let button_color = [
                button_color[0] as f32 / 255.0,
                button_color[1] as f32 / 255.0,
                button_color[2] as f32 / 255.0,
                0.95,
            ];
            let text_color = ptype_ui_text_color(ptype);
            let text_color = [
                text_color[0] as f32 / 255.0,
                text_color[1] as f32 / 255.0,
                text_color[2] as f32 / 255.0,
                1.0,
            ];
            let style1 = [
                ui.push_style_color(StyleColor::Button, button_color),
                ui.push_style_color(StyleColor::ButtonHovered, button_color),
                ui.push_style_color(StyleColor::ButtonActive, button_color),
                ui.push_style_color(StyleColor::Text, text_color),
            ];
            let style2 = ui.push_style_var(StyleVar::FrameRounding(6.0));
            let size = if ptype == *selected_particle {
                [100.0, 55.0]
            } else {
                [85.0, 40.0]
            };
            if ui.button_with_size(text, size) {
                *selected_particle = ptype;
            }
            for style in style1 {
                style.pop();
            }
            style2.pop();
        };

        // Setup styles
        let foreground_color1 = [
            (230.0 / 255.0f32),
            (230.0 / 255.0f32),
            (230.0 / 255.0f32),
            0.95,
        ];
        let background_color1 = [
            (92.0 / 255.0f32),
            (64.0 / 255.0f32),
            (38.0 / 255.0f32),
            0.95,
        ];
        let foreground_color2 = [
            (80.0 / 255.0f32),
            (80.0 / 255.0f32),
            (80.0 / 255.0f32),
            0.95,
        ];
        let background_color2 = [
            (60.0 / 255.0f32),
            (60.0 / 255.0f32),
            (60.0 / 255.0f32),
            0.95,
        ];
        let style1 = [
            ui.push_style_color(StyleColor::Button, background_color1),
            ui.push_style_color(StyleColor::ButtonActive, background_color1),
            ui.push_style_color(StyleColor::ButtonHovered, background_color1),
            ui.push_style_color(StyleColor::CheckMark, foreground_color1),
            ui.push_style_color(StyleColor::FrameBg, background_color1),
            ui.push_style_color(StyleColor::FrameBgActive, background_color1),
            ui.push_style_color(StyleColor::FrameBgHovered, background_color1),
            ui.push_style_color(StyleColor::ScrollbarBg, background_color2),
            ui.push_style_color(StyleColor::ScrollbarGrab, foreground_color2),
            ui.push_style_color(StyleColor::ScrollbarGrabActive, foreground_color2),
            ui.push_style_color(StyleColor::ScrollbarGrabHovered, foreground_color2),
            ui.push_style_color(StyleColor::SliderGrab, foreground_color1),
            ui.push_style_color(StyleColor::SliderGrabActive, foreground_color1),
        ];
        let mut style2 = vec![
            ui.push_style_var(StyleVar::FrameRounding(4.0)),
            ui.push_style_var(StyleVar::WindowBorderSize(0.0)),
            ui.push_style_var(StyleVar::WindowMinSize([1.0, 1.0])),
            ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0])),
        ];

        // Draw toggle UI checkbox
        let should_display_ui = &mut self.should_display_ui;
        ImWindow::new("toggle_ui_window")
            .position([10.0, 27.0], Condition::Always)
            .title_bar(false)
            .draw_background(false)
            .movable(false)
            .resizable(false)
            .build(&ui, || {
                ui.checkbox("Toggle UI", should_display_ui);
            });

        if self.should_display_ui {
            // Draw particle selection buttons
            ImWindow::new("particle_selection_window")
                .always_auto_resize(true)
                .content_size([1416.0, 55.0])
                .position([108.0, 10.0], Condition::Always)
                .title_bar(false)
                .draw_background(false)
                .movable(false)
                .resizable(false)
                .horizontal_scrollbar(true)
                .build(&ui, || {
                    particle_selector_button("Delete Tool", None);
                    particle_selector_button("Sand", Some(ParticleType::Sand));
                    particle_selector_button("Water", Some(ParticleType::Water));
                    particle_selector_button("Acid", Some(ParticleType::Acid));
                    particle_selector_button("Iridium", Some(ParticleType::Iridium));
                    particle_selector_button("Replicator", Some(ParticleType::Replicator));
                    particle_selector_button("Plant", Some(ParticleType::Plant));
                    particle_selector_button("Cryotheum", Some(ParticleType::Cryotheum));
                    particle_selector_button("Unstable", Some(ParticleType::Unstable));
                    particle_selector_button("Electricity", Some(ParticleType::Electricity));
                    particle_selector_button("Life", Some(ParticleType::Life));
                    particle_selector_button("Fire", Some(ParticleType::Fire));
                    particle_selector_button("Mirror", Some(ParticleType::Mirror));
                    particle_selector_button("Glitch", Some(ParticleType::Glitch));
                });

            let y = if window.inner_size().width < 1416 {
                87.0
            } else {
                75.0
            };
            let was_paused_before_popup = &mut self.was_paused_before_popup;
            ImWindow::new("second_row_window")
                .always_auto_resize(true)
                .position([10.0, y], Condition::Always)
                .title_bar(false)
                .draw_background(false)
                .movable(false)
                .resizable(false)
                .build(&ui, || {
                    // Draw the pause game checkbox
                    ui.set_cursor_pos([0.0, 4.0]);
                    ui.checkbox("Paused", game_paused);
                    // Draw the emoty sandbox button
                    ui.set_cursor_pos([84.0, 1.0]);
                    if ui.button_with_size("Empty Sandbox", [125.0, 27.0]) {
                        *was_paused_before_popup = *game_paused;
                        *game_paused = true;
                        ui.open_popup("empty_sandbox_popup");
                    }
                    // Draw the empty sandbox popup
                    style2.clear();
                    ui.popup_modal("empty_sandbox_popup")
                        .title_bar(false)
                        .movable(false)
                        .resizable(false)
                        .build(&ui, || {
                            ui.text("Empty Sandbox?");
                            if ui.button_with_size("Yes", [60.0, 30.0]) {
                                sandbox.empty_out();
                                ui.close_current_popup();
                                *game_paused = *was_paused_before_popup;
                            }
                            ui.same_line();
                            if ui.button_with_size("No", [60.0, 30.0]) {
                                ui.close_current_popup();
                                *game_paused = *was_paused_before_popup;
                            }
                        });
                    style2 = vec![
                        ui.push_style_var(StyleVar::FrameRounding(4.0)),
                        ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0])),
                        ui.push_style_var(StyleVar::WindowMinSize([1.0, 1.0])),
                    ];
                    // Draw the brush size slider
                    ui.set_cursor_pos([219.0, 4.0]);
                    Slider::new("Brush Size", 1, 10).build(&ui, brush_size);
                });
        }

        // Draw the FPS counter
        if self.should_display_fps {
            let height: f32 = window.inner_size().to_logical(window.scale_factor()).height;
            let y = height - 26.0;
            ImWindow::new("fps_window")
                .always_auto_resize(true)
                .position([10.0, y], Condition::Always)
                .title_bar(false)
                .draw_background(false)
                .movable(false)
                .resizable(false)
                .no_inputs()
                .build(&ui, || ui.text(format!("FPS: {:.0}", ui.io().framerate)));
        }

        for style in style1 {
            style.pop();
        }
        drop(style2);
        // Draw the profiler
        if self.should_display_profiler {
            self.profiler_ui.window(&ui);
        }

        // Render
        self.imgui_platform.prepare_render(&ui, window);
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("ui_render_pass"),
            color_attachments: &[RenderPassColorAttachment {
                view: render_texture,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        self.imgui_renderer
            .render(ui.render(), queue, device, &mut pass)
            .expect("Failed to render UI");
    }

    pub fn handle_event(&mut self, window: &Window, event: &Event<'_, ()>) {
        self.imgui_platform
            .handle_event(self.imgui.io_mut(), window, event);
    }

    pub fn ui_wants_mouse_input(&self) -> bool {
        self.imgui.io().want_capture_mouse
    }
}

pub fn ptype_ui_color(ptype: Option<ParticleType>) -> [u8; 3] {
    match ptype {
        None => [26, 26, 26],
        Some(ParticleType::Sand) => [196, 192, 135],
        Some(ParticleType::Water) => [26, 91, 165],
        Some(ParticleType::Acid) => [148, 219, 10],
        Some(ParticleType::Iridium) => [100, 100, 100],
        Some(ParticleType::Replicator) => [78, 21, 77],
        Some(ParticleType::Plant) => [6, 89, 9],
        Some(ParticleType::Cryotheum) => [12, 193, 255],
        Some(ParticleType::Unstable) => [94, 78, 55],
        Some(ParticleType::Electricity) => [255, 244, 49],
        Some(ParticleType::Glass) => unreachable!(),
        Some(ParticleType::Life) => [135, 12, 211],
        Some(ParticleType::SuperLife) => unreachable!(),
        Some(ParticleType::Blood) => unreachable!(),
        Some(ParticleType::Smoke) => unreachable!(),
        Some(ParticleType::Fire) => [255, 151, 20],
        Some(ParticleType::Mirror) => [78, 216, 131],
        Some(ParticleType::Steam) => unreachable!(),
        Some(ParticleType::Glitch) => [120, 119, 100],
    }
}

pub fn ptype_ui_text_color(ptype: Option<ParticleType>) -> [u8; 3] {
    let light = match ptype {
        None => true,
        Some(ParticleType::Sand) => false,
        Some(ParticleType::Water) => true,
        Some(ParticleType::Acid) => false,
        Some(ParticleType::Iridium) => true,
        Some(ParticleType::Replicator) => true,
        Some(ParticleType::Plant) => true,
        Some(ParticleType::Cryotheum) => false,
        Some(ParticleType::Unstable) => true,
        Some(ParticleType::Electricity) => false,
        Some(ParticleType::Glass) => unreachable!(),
        Some(ParticleType::Life) => true,
        Some(ParticleType::SuperLife) => unreachable!(),
        Some(ParticleType::Blood) => unreachable!(),
        Some(ParticleType::Smoke) => unreachable!(),
        Some(ParticleType::Fire) => false,
        Some(ParticleType::Mirror) => false,
        Some(ParticleType::Steam) => unreachable!(),
        Some(ParticleType::Glitch) => false,
    };
    if light {
        [204, 204, 204]
    } else {
        [0, 0, 0]
    }
}
