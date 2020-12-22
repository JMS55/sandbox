use crate::particle::ParticleType;
use crate::ui::{ptype_ui_color, ptype_ui_text_color};
use winit::platform::unix::{ARGBColor, Button, ButtonState, Element, Theme};

pub struct WaylandCSDTheme {
    pub selected_particle: Option<ParticleType>,
}

impl Theme for WaylandCSDTheme {
    fn element_color(&self, element: Element, window_active: bool) -> ARGBColor {
        let [mut r, mut g, mut b] = match element {
            Element::Bar => ptype_ui_color(self.selected_particle),
            Element::Separator => [0, 0, 0],
            Element::Text => ptype_ui_text_color(self.selected_particle),
        };
        if !window_active && element != Element::Separator {
            r = r.saturating_add(30);
            g = g.saturating_add(30);
            b = b.saturating_add(30);
        }
        ARGBColor { a: 255, r, g, b }
    }

    fn button_color(
        &self,
        _: Button,
        state: ButtonState,
        foreground: bool,
        window_active: bool,
    ) -> ARGBColor {
        let [mut r, mut g, mut b] = match foreground {
            true => ptype_ui_text_color(self.selected_particle),
            false => ptype_ui_color(self.selected_particle),
        };
        match state {
            ButtonState::Hovered | ButtonState::Disabled => {
                r = r.saturating_sub(30);
                g = g.saturating_sub(30);
                b = b.saturating_sub(30);
            }
            _ => {}
        }
        if !window_active {
            r = r.saturating_add(30);
            g = g.saturating_add(30);
            b = b.saturating_add(30);
        }
        ARGBColor { a: 255, r, g, b }
    }
}
