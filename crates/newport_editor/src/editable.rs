use crate::*;

use widgets::color_picker::{ color_edit_button_srgba, Alpha };

use newport_math as math;

pub trait Editable {
    fn edit(&mut self, name: &str, ui: &mut Ui);
}

impl Editable for math::Vector2 {
    fn edit(&mut self, name: &str, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.separator();
            ui.add(DragValue::new(&mut self.x));
            ui.add(DragValue::new(&mut self.y));
        });
    }
}

impl Editable for math::Vector3 {
    fn edit(&mut self, name: &str, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.separator();
            ui.add(DragValue::new(&mut self.x));
            ui.add(DragValue::new(&mut self.y));
            ui.add(DragValue::new(&mut self.z));
        });
    }
}

impl Editable for math::Vector4 {
    fn edit(&mut self, name: &str, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.separator();
            ui.add(DragValue::new(&mut self.x));
            ui.add(DragValue::new(&mut self.y));
            ui.add(DragValue::new(&mut self.z));
            ui.add(DragValue::new(&mut self.w));
        });
    }
}

impl Editable for math::Color {
    fn edit(&mut self, name: &str, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.separator();

            let r = (self.r * 255.0) as u8 / 255;
            let g = (self.g * 255.0) as u8 / 255;
            let b = (self.b * 255.0) as u8 / 255;
            let a = (self.a * 255.0) as u8 / 255;

            let mut srgba = Color32::from_rgba_unmultiplied(r, g, b, a);
            color_edit_button_srgba(ui, &mut srgba, Alpha::OnlyBlend);

            self.r = srgba.r() as f32 / 255.0;
            self.g = srgba.g() as f32 / 255.0;
            self.b = srgba.b() as f32 / 255.0;
            self.a = srgba.a() as f32 / 255.0;
        });
    }
}

impl Editable for math::Quaternion {
    fn edit(&mut self, name: &str, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.separator();
            ui.add(DragValue::new(&mut self.x));
            ui.add(DragValue::new(&mut self.y));
            ui.add(DragValue::new(&mut self.z));
            ui.add(DragValue::new(&mut self.w));
        });
    }
}

impl Editable for String {
    fn edit(&mut self, name: &str, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(name);
            ui.separator();
            ui.text_edit_singleline(self);
        });
    }
}

// TODO: Maybe the math library should have a numeric trait
macro_rules! impl_editable_numberic {
    ($t:ident) => {
        impl Editable for $t {
            fn edit(&mut self, name: &str, ui: &mut Ui) {
                ui.horizontal(|ui| {
                    ui.label(name);
                    ui.separator();
                    ui.add(DragValue::new(self));
                });
            }
        }
    };
}

impl_editable_numberic!(f32);
impl_editable_numberic!(f64);
impl_editable_numberic!(i8);
impl_editable_numberic!(u8);
impl_editable_numberic!(i16);
impl_editable_numberic!(u16);
impl_editable_numberic!(i32);
impl_editable_numberic!(u32);
impl_editable_numberic!(i64);
impl_editable_numberic!(u64);
impl_editable_numberic!(isize);
impl_editable_numberic!(usize);