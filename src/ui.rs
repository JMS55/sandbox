use crate::particle::ParticleType;
use crate::sandbox::Sandbox;
use imgui::{
    im_str, Condition, Context, FontSource, ImStr, Slider, StyleColor, StyleVar, Window as ImWindow,
};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use pixels::wgpu::*;
use puffin::GlobalProfiler;
use puffin_imgui::ProfilerUi;
use std::time::Instant;
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
    recent_frames: [Instant; 10],
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
            recent_frames: [Instant::now(); 10],
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

    pub fn start_of_frame(&mut self) {
        if self.should_display_profiler {
            GlobalProfiler::lock().new_frame();
        }

        self.imgui
            .io_mut()
            .update_delta_time(self.recent_frames[self.recent_frames.len() - 1].elapsed());
        self.recent_frames.rotate_left(1);
        self.recent_frames[self.recent_frames.len() - 1] = Instant::now();
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
        let mut particle_selector_button = |text: &ImStr, ptype: Option<ParticleType>| {
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
            let style1 = ui.push_style_colors(&[
                (StyleColor::Button, button_color),
                (StyleColor::ButtonHovered, button_color),
                (StyleColor::ButtonActive, button_color),
                (StyleColor::Text, text_color),
            ]);
            let style2 = ui.push_style_var(StyleVar::FrameRounding(6.0));
            let size = if ptype == *selected_particle {
                [100.0, 55.0]
            } else {
                [85.0, 40.0]
            };
            if ui.button(text, size) {
                *selected_particle = ptype;
            }
            style1.pop(&ui);
            style2.pop(&ui);
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
        let style1 = ui.push_style_colors(&[
            (StyleColor::Button, background_color1),
            (StyleColor::ButtonActive, background_color1),
            (StyleColor::ButtonHovered, background_color1),
            (StyleColor::CheckMark, foreground_color1),
            (StyleColor::FrameBg, background_color1),
            (StyleColor::FrameBgActive, background_color1),
            (StyleColor::FrameBgHovered, background_color1),
            (StyleColor::ScrollbarBg, background_color2),
            (StyleColor::ScrollbarGrab, foreground_color2),
            (StyleColor::ScrollbarGrabActive, foreground_color2),
            (StyleColor::ScrollbarGrabHovered, foreground_color2),
            (StyleColor::SliderGrab, foreground_color1),
            (StyleColor::SliderGrabActive, foreground_color1),
        ]);
        let mut style2 = Some(ui.push_style_vars(&[
            StyleVar::FrameRounding(4.0),
            StyleVar::WindowBorderSize(0.0),
            StyleVar::WindowMinSize([1.0, 1.0]),
            StyleVar::WindowPadding([0.0, 0.0]),
        ]));

        // Draw toggle UI checkbox
        let should_display_ui = &mut self.should_display_ui;
        ImWindow::new(im_str!("toggle_ui_window"))
            .position([10.0, 27.0], Condition::Always)
            .title_bar(false)
            .draw_background(false)
            .movable(false)
            .resizable(false)
            .build(&ui, || {
                ui.checkbox(im_str!("Toggle UI"), should_display_ui);
            });

        if self.should_display_ui {
            // Draw particle selection buttons
            ImWindow::new(im_str!("particle_selection_window"))
                .always_auto_resize(true)
                .content_size([1416.0, 55.0])
                .position([108.0, 10.0], Condition::Always)
                .title_bar(false)
                .draw_background(false)
                .movable(false)
                .resizable(false)
                .horizontal_scrollbar(true)
                .build(&ui, || {
                    particle_selector_button(im_str!("Delete Tool"), None);
                    particle_selector_button(im_str!("Sand"), Some(ParticleType::Sand));
                    particle_selector_button(im_str!("Water"), Some(ParticleType::Water));
                    particle_selector_button(im_str!("Acid"), Some(ParticleType::Acid));
                    particle_selector_button(im_str!("Iridium"), Some(ParticleType::Iridium));
                    particle_selector_button(im_str!("Replicator"), Some(ParticleType::Replicator));
                    particle_selector_button(im_str!("Plant"), Some(ParticleType::Plant));
                    particle_selector_button(im_str!("Cryotheum"), Some(ParticleType::Cryotheum));
                    particle_selector_button(im_str!("Unstable"), Some(ParticleType::Unstable));
                    particle_selector_button(
                        im_str!("Electricity"),
                        Some(ParticleType::Electricity),
                    );
                    particle_selector_button(im_str!("Life"), Some(ParticleType::Life));
                    particle_selector_button(im_str!("Fire"), Some(ParticleType::Fire));
                    particle_selector_button(im_str!("Mirror"), Some(ParticleType::Mirror));
                    particle_selector_button(im_str!("Glitch"), Some(ParticleType::Glitch));
                });

            let y = if window.inner_size().width < 1416 {
                87.0
            } else {
                75.0
            };
            let was_paused_before_popup = &mut self.was_paused_before_popup;
            ImWindow::new(im_str!("second_row_window"))
                .always_auto_resize(true)
                .position([10.0, y], Condition::Always)
                .title_bar(false)
                .draw_background(false)
                .movable(false)
                .resizable(false)
                .build(&ui, || {
                    // Draw the pause game checkbox
                    ui.set_cursor_pos([0.0, 4.0]);
                    ui.checkbox(im_str!("Paused"), game_paused);
                    // Draw the emoty sandbox button
                    ui.set_cursor_pos([84.0, 1.0]);
                    if ui.button(im_str!("Empty Sandbox"), [125.0, 27.0]) {
                        *was_paused_before_popup = *game_paused;
                        *game_paused = true;
                        ui.open_popup(im_str!("empty_sandbox_popup"));
                    }
                    // Draw the empty sandbox popup
                    style2.take().unwrap().pop(&ui);
                    ui.popup_modal(im_str!("empty_sandbox_popup"))
                        .title_bar(false)
                        .movable(false)
                        .resizable(false)
                        .build(|| {
                            ui.text("Empty Sandbox?");
                            if ui.button(im_str!("Yes"), [60.0, 30.0]) {
                                sandbox.empty_out();
                                ui.close_current_popup();
                                *game_paused = *was_paused_before_popup;
                            }
                            ui.same_line(0.0);
                            if ui.button(im_str!("No"), [60.0, 30.0]) {
                                ui.close_current_popup();
                                *game_paused = *was_paused_before_popup;
                            }
                        });
                    style2 = Some(ui.push_style_vars(&[
                        StyleVar::FrameRounding(4.0),
                        StyleVar::WindowPadding([0.0, 0.0]),
                        StyleVar::WindowMinSize([1.0, 1.0]),
                    ]));
                    // Draw the brush size slider
                    ui.set_cursor_pos([219.0, 4.0]);
                    Slider::new(im_str!("Brush Size"))
                        .range(1..=10)
                        .build(&ui, brush_size);
                });
        }

        // Draw the FPS counter
        if self.should_display_fps {
            let height: f32 = window.inner_size().to_logical(window.scale_factor()).height;
            let y = height - 26.0;
            let fps =
                self.recent_frames.len() as f64 / self.recent_frames[0].elapsed().as_secs_f64();
            ImWindow::new(im_str!("fps_window"))
                .always_auto_resize(true)
                .position([10.0, y], Condition::Always)
                .title_bar(false)
                .draw_background(false)
                .movable(false)
                .resizable(false)
                .no_inputs()
                .build(&ui, || ui.text(format!("FPS: {:.0}", fps)));
        }

        style1.pop(&ui);
        style2.unwrap().pop(&ui);

        // Draw the profiler
        if self.should_display_profiler {
            self.profiler_ui.window(&ui);
        }

        // Render
        self.imgui_platform.prepare_render(&ui, window);
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[RenderPassColorAttachmentDescriptor {
                attachment: render_texture,
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

    pub fn handle_event(&mut self, window: &Window, event: &Event<()>) {
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
