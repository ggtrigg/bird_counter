extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;

mod gui;

use gio::prelude::*;
use gtk::prelude::*;
pub use gui::*;
pub use gui::images::*;
pub use gui::charts::*;
use std::env::args;
use std::result::Result;
use turbosql::{Blob, Turbosql, execute};

use chrono::Local;
use gtk::{Application, ApplicationWindow};

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
        window.set_icon_from_file(gui::image_dir().join("bird_icon.png"))
            .map_err(|error| { println!("Error loading icon from file.\nError: {}", error) }).ok();
        if *is_fullscreen {
            window.fullscreen();
        }
        let gui = gui::Gui::new(window);
        gui.window.add(&gui.build(animals));
        let mainwin = gui.window.clone();
        
        glib::timeout_add_seconds_local(300, move || {
            gui.refresh();
            glib::Continue(true)
        });

        mainwin.show_all();
    });

    let args: Vec<String> = args().collect();
    application.run(&args);
}

fn log_sighting(animal_id: i64) {
    let s = Sighting::new(animal_id);
    if let Err(error) = s.insert() {
        gui::alert(&format!("Error logging sighting - {}", error));
    }
}

pub fn clear_sighting(animal_id: i64) {
    execute!(r#"DELETE FROM sighting WHERE animal_id = ? AND date(seen_at, "unixepoch", "localtime") = date("now", "localtime")"#, animal_id).ok();
}

fn handle_local_options(app: &gtk::Application, opts: &glib::VariantDict) -> i32 {
    unsafe { app.set_data("fullscreen", opts.contains("fullscreen")) }
    -1
}

// To get daily bird sighting data from db...
// select distinct animal.name as name, date(seen_at, "unixepoch", "localtime") as date from sighting left join animal on animal.rowid = sighting.animal_id order by name,date;
// To get a weekly breakdown...
// select animal.name, count(distinct date(seen_at, "unixepoch", "localtime")), strftime("%Y%W", seen_at, "unixepoch", "localtime") as week from sighting left join animal on animal.rowid = sighting.animal_id group by animal_id,week;