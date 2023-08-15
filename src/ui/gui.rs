use egui_sdl2_gl::{with_sdl2, DpiScaling, ShaderVersion, EguiStateHandler, painter::Painter, egui::{CtxRef, color::Hsva, color_picker::Alpha}, sdl2::video::Window};

use crate::{egui_set_theme, FRAPPE};

pub struct Gui {
    pub painter: Painter,
    pub egui_state: EguiStateHandler,
    pub egui_ctx: CtxRef,
    pub values: Values
}

impl Gui {
    pub fn new(window: &Window) -> Result<Self, &'static str> {
        let (painter, egui_state) = with_sdl2(window, ShaderVersion::Default, DpiScaling::Default);
        let egui_ctx = CtxRef::default();
        egui_set_theme(&*egui_ctx, FRAPPE);

        Ok(Gui {
            painter,
            egui_state,
            egui_ctx,
            values: Default::default(),
        })
    }
}

pub struct Values {
    pub bg: Hsva,
    pub alpha: Alpha,
    pub notes: Hsva,
    pub particles: Hsva,
}

impl Default for Values {
    fn default() -> Self {
        Values {
            bg: Hsva { h: 0.0, s: 0.0, v: 0.1, a: 1.0 },
            alpha: Alpha::Opaque,
            notes: Hsva { h: 0.5, s: 0.1, v: 0.1, a: 1.0 },
            particles: Hsva { h: 0.75, s: 0.5, v: 0.5, a: 1.0 },
        }
    }
}