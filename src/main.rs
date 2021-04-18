extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;

mod gui;

use gio::prelude::*;
use gtk::prelude::*;
pub use gui::*;
use std::env::args;
use std::result::Result;
use turbosql::{Blob, Turbosql};

use chrono::Local;
use gtk::{Application, ApplicationWindow, Box, GestureSwipe, Notebook, Orientation, Label};

#[derive(Turbosql, Default)]
pub struct Animal {
    rowid: Option<i64>,
    name: Option<String>,
    filename: Option<String>,
    image: Option<Blob>,
}

#[derive(Turbosql, Default)]
struct Sighting {
    rowid: Option<i64>,
    animal_id: Option<i64>,
    seen_at: Option<i64>,
}

impl Sighting {
    fn new(animal_id: i64) -> Sighting {
        let s = Sighting {
            rowid: None,
            animal_id: Some(animal_id),
            seen_at: Some(Local::now().timestamp()),
        };
        s
    }
}

fn main() {
    let application =
        Application::new(Some("com.github.ggtrigg.bird_counter"), Default::default())
            .expect("failed to initialize GTK application");

    application.add_main_option(
        "fullscreen",
        glib::Char::new('f').unwrap(),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        "start in fullscreen mode",
        None,
    );
    application.connect_handle_local_options(handle_local_options);

    application.connect_activate(|app| {
        let animals: Vec<Animal> =
            turbosql::select!(Vec<Animal>).expect("Couldn't retrieve animals");
        let is_fullscreen: &bool = unsafe { app.get_data("fullscreen").unwrap_or(&false) };

        let window = ApplicationWindow::new(app);
        window.set_title("Bird Counter");
        window.set_icon_from_file(gui::image_dir().join("bird_icon.png")).map_err(|error| { println!("Error loading icon from file.\nError: {}", error) }).ok();
        if *is_fullscreen {
            window.fullscreen();
        } else {
            // window.set_default_size(350, 70);
        }
        let notebook = Notebook::new();
        window.add(&notebook);
        // window.connect_configure_event(detect_resize);

        let gesture = GestureSwipe::new(&notebook);
        gesture.set_touch_only(true);
        gesture.connect_swipe(handle_swipe);
        let vbox = Box::new(Orientation::Vertical, 5);
        vbox.set_homogeneous(true);

        let abox = Box::new(Orientation::Horizontal, 5);
        let label = Label::new(Some("Sighting chart(s) will go here"));
        abox.add(&label);
        notebook.add(&vbox);
        notebook.add(&abox);
        load_images(&vbox, &animals);
        
        glib::timeout_add_seconds_local(30, move || { refresh_images(&vbox) });

        window.show_all();
    });

    let args: Vec<String> = args().collect();
    application.run(&args);
}

fn log_sighting(animal_id: i64) {
    let s = Sighting::new(animal_id);
    let _oid = s.insert();
}

fn handle_swipe(gesture: &GestureSwipe, vx: f64, vy: f64) {
    println!("Got gesture! {:?}, velocity {}, {}", gesture, vx, vy);
}

fn handle_local_options(app: &gtk::Application, opts: &glib::VariantDict) -> i32 {
    unsafe { app.set_data("fullscreen", opts.contains("fullscreen")) }
    -1
}

// To get daily bird sighting data from db...
// select distinct animal.name, date(seen_at, "unixepoch", "localtime") from sighting left join animal on animal.rowid = sighting.animal_id;