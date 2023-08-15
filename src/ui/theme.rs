use egui_sdl2_gl::egui::{self, epaint, style, Color32};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Theme {
    pub rosewater: Color32,
    pub flamingo: Color32,
    pub pink: Color32,
    pub mauve: Color32,
    pub red: Color32,
    pub maroon: Color32,
    pub peach: Color32,
    pub yellow: Color32,
    pub green: Color32,
    pub teal: Color32,
    pub sky: Color32,
    pub sapphire: Color32,
    pub blue: Color32,
    pub lavender: Color32,
    pub text: Color32,
    pub subtext1: Color32,
    pub subtext0: Color32,
    pub overlay2: Color32,
    pub overlay1: Color32,
    pub overlay0: Color32,
    pub surface2: Color32,
    pub surface1: Color32,
    pub surface0: Color32,
    pub base: Color32,
    pub mantle: Color32,
    pub crust: Color32,
}

// Catppuccin color theme
pub const FRAPPE: Theme = Theme {
    rosewater: Color32::from_rgb(242, 213, 207),
    flamingo: Color32::from_rgb(238, 190, 190),
    pink: Color32::from_rgb(244, 184, 228),
    mauve: Color32::from_rgb(202, 158, 230),
    red: Color32::from_rgb(231, 130, 132),
    maroon: Color32::from_rgb(234, 153, 156),
    peach: Color32::from_rgb(239, 159, 118),
    yellow: Color32::from_rgb(229, 200, 144),
    green: Color32::from_rgb(166, 209, 137),
    teal: Color32::from_rgb(129, 200, 190),
    sky: Color32::from_rgb(153, 209, 219),
    sapphire: Color32::from_rgb(133, 193, 220),
    blue: Color32::from_rgb(140, 170, 238),
    lavender: Color32::from_rgb(186, 187, 241),
    text: Color32::from_rgb(198, 208, 245),
    subtext1: Color32::from_rgb(181, 191, 226),
    subtext0: Color32::from_rgb(165, 173, 206),
    overlay2: Color32::from_rgb(148, 156, 187),
    overlay1: Color32::from_rgb(131, 139, 167),
    overlay0: Color32::from_rgb(115, 121, 148),
    surface2: Color32::from_rgb(98, 104, 128),
    surface1: Color32::from_rgb(81, 87, 109),
    surface0: Color32::from_rgb(65, 69, 89),
    base: Color32::from_rgb(48, 52, 70),
    mantle: Color32::from_rgb(41, 44, 60),
    crust: Color32::from_rgb(35, 38, 52),
};

fn make_widget_visual(
    old: style::WidgetVisuals,
    theme: &Theme,
    bg_fill: egui::Color32,
) -> style::WidgetVisuals {
    style::WidgetVisuals {
        bg_fill,
        bg_stroke: egui::Stroke {
            color: theme.overlay1,
            ..old.bg_stroke
        },
        fg_stroke: egui::Stroke {
            color: theme.text,
            ..old.fg_stroke
        },
        ..old
    }
}

pub fn egui_set_theme(ctx: &egui::Context, theme: Theme) {
    let old = ctx.style().visuals.clone();
    ctx.set_visuals(egui::Visuals {
        override_text_color: Some(theme.text),
        hyperlink_color: theme.rosewater,
        faint_bg_color: theme.surface0,
        extreme_bg_color: theme.crust,
        code_bg_color: theme.mantle,
        widgets: style::Widgets {
            noninteractive: make_widget_visual(old.widgets.noninteractive, &theme, theme.base),
            inactive: make_widget_visual(old.widgets.inactive, &theme, theme.surface0),
            hovered: make_widget_visual(old.widgets.hovered, &theme, theme.surface2),
            active: make_widget_visual(old.widgets.active, &theme, theme.surface1),
            open: make_widget_visual(old.widgets.open, &theme, theme.surface0),
        },
        selection: style::Selection {
            bg_fill: theme
                .blue
                .linear_multiply(0.2),
            stroke: egui::Stroke {
                color: theme.overlay1,
                ..old.selection.stroke
            },
        },
        window_shadow: epaint::Shadow {
            color: theme.base,
            ..old.window_shadow
        },
        popup_shadow: epaint::Shadow {
            color: theme.base,
            ..old.popup_shadow
        },
        ..old
    });
}