use os::window::{WindowBuilder, WindowEvent};

fn main() {
    let mut window = WindowBuilder::new()
        .title("Hello, world!".to_string())
        .size((1280, 720))
        .spawn()
        .unwrap();
    
    window.set_visible(true);

    'run: loop {
        for event in window.poll_events() {
            match event {
                WindowEvent::Closed => {
                    break 'run;
                },
                _ => { }
            }  
        }
    }
}
