use crate::editor::{
    Context,
    Page
};

struct LevelEditor;

impl Page for LevelEditor {
    fn can_close(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "Level Editor"
    }

    fn show(&mut self, ctx: &mut Context) {
        
    }
}