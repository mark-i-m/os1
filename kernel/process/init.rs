use super::{Process};

// The init process
pub struct Init {
    name: &'static str,
}

impl Process for Init {
    fn new (name: &'static str) -> Init {
        Init { name: name }
    }

    fn run () {
        use super::super::window::{Window, Color};

        // clear the screen
        Window::clear_screen();

        let mut w0 = Window::new(40, 10, (4, 10));

        w0.set_bg_color(Color::LightBlue);
        w0.paint();

        w0.set_cursor((1, 1));
        w0.set_bg_color(Color::LightGreen);
        w0.set_fg_color(Color::Red);
        w0.put_str("Hello World!");
    }
}
