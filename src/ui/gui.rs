use egui_sdl2_gl::{with_sdl2, DpiScaling, ShaderVersion, EguiStateHandler, painter::Painter, egui::CtxRef, sdl2::video::Window};

use crate::{egui_set_theme, FRAPPE};

pub struct Gui {
    pub painter: Painter,
    pub egui_state: EguiStateHandler,
    pub egui_ctx: CtxRef,
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
        })
    }
}