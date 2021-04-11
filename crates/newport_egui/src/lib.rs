use newport_engine::*;

pub use egui::*;

use std::sync::Mutex;
use std::cell::RefCell;

pub struct EGUI {
    context:  Mutex<CtxRef>,
    pub func: Option<fn(&mut CtxRef)>,

    input: RefCell<RawInput>,
}

impl ModuleCompileTime for EGUI {
    fn new() -> Self {
        Self{
            context: Mutex::new(CtxRef::default()),
            func:    None,

            input: RefCell::new(RawInput::default()),
        }
    }
}

impl ModuleRuntime for EGUI {
    fn process_input(&self, _event: &WindowEvent) -> bool {
        false
    }

    fn on_tick(&self, dt: f32) {
        let func = self.func.unwrap();
        let mut lock = self.context.lock().unwrap();

        let engine = Engine::as_ref();
        let size = engine.window().size();

        let mut input = self.input.borrow_mut();
        input.screen_rect = Some(
            Rect::from_min_max(
                Default::default(), 
                pos2(size.0 as f32, size.1 as f32)
            )
        );
        input.predicted_dt = dt;

        lock.begin_frame(input.clone());
        input.events.clear();    
        func(&mut lock);
        let (output, shapes) = lock.end_frame();
        let clipped_meshes = lock.tessellate(shapes);
        
    }
}