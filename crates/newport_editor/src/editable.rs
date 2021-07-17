use crate::*;

pub trait Editable {
    fn edit(&mut self, name: &str, ui: &mut Builder);
}

// impl Editable for math::Vector2 {
//     fn edit(&mut self, name: &str, ui: &mut Ui) {
//         ui.horizontal(|ui| {
//             ui.label(name);
//             ui.separator();
//             ui.add(DragValue::new(&mut self.x));
//             ui.add(DragValue::new(&mut self.y));
//         });
//     }
// }

// impl Editable for math::Vector3 {
//     fn edit(&mut self, name: &str, ui: &mut Ui) {
//         ui.horizontal(|ui| {
//             ui.label(name);
//             ui.separator();
//             ui.add(DragValue::new(&mut self.x));
//             ui.add(DragValue::new(&mut self.y));
//             ui.add(DragValue::new(&mut self.z));
//         });
//     }
// }

// impl Editable for math::Vector4 {
//     fn edit(&mut self, name: &str, ui: &mut Ui) {
//         ui.horizontal(|ui| {
//             ui.label(name);
//             ui.separator();
//             ui.add(DragValue::new(&mut self.x));
//             ui.add(DragValue::new(&mut self.y));
//             ui.add(DragValue::new(&mut self.z));
//             ui.add(DragValue::new(&mut self.w));
//         });
//     }
// }

// impl Editable for math::Color {
//     fn edit(&mut self, name: &str, ui: &mut Ui) {
//         ui.horizontal(|ui| {
//             ui.label(name);
//             ui.separator();
//         });
//     }
// }

// impl Editable for math::Quaternion {
//     fn edit(&mut self, name: &str, ui: &mut Ui) {
//         ui.horizontal(|ui| {
//             ui.label(name);
//             ui.separator();
//             ui.add(DragValue::new(&mut self.x));
//             ui.add(DragValue::new(&mut self.y));
//             ui.add(DragValue::new(&mut self.z));
//             ui.add(DragValue::new(&mut self.w));
//         });
//     }
// }

// impl Editable for String {
//     fn edit(&mut self, name: &str, ui: &mut Ui) {
//         ui.horizontal(|ui| {
//             ui.label(name);
//             ui.separator();
//             ui.text_edit_singleline(self);
//         });
//     }
// }

// // TODO: Maybe the math library should have a numeric trait
// macro_rules! impl_editable_numberic {
//     ($t:ident) => {
//         impl Editable for $t {
//             fn edit(&mut self, name: &str, ui: &mut Ui) {
//                 ui.horizontal(|ui| {
//                     ui.label(name);
//                     ui.separator();
//                     ui.add(DragValue::new(self));
//                 });
//             }
//         }
//     };
// }

// impl_editable_numberic!(f32);
// impl_editable_numberic!(f64);
// impl_editable_numberic!(i8);
// impl_editable_numberic!(u8);
// impl_editable_numberic!(i16);
// impl_editable_numberic!(u16);
// impl_editable_numberic!(i32);
// impl_editable_numberic!(u32);
// impl_editable_numberic!(i64);
// impl_editable_numberic!(u64);
// impl_editable_numberic!(isize);
// impl_editable_numberic!(usize);
