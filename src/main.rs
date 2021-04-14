extern crate gtk;
extern crate gio;
extern crate gdk_pixbuf;

use turbosql::{Turbosql, Blob, select};
use std::result::Result;
use std::path::Path;
use std::fs;
use gtk::prelude::*;
use gio::prelude::*;
use std::path::PathBuf;

use gtk::{
    Application, ApplicationWindow, Box, Image, EventBox, Entry, Orientation, FileChooserDialog, FileChooserAction,
    FileFilter, ResponseType, Window, MessageDialog
};
use chrono::Local;

#[derive(Turbosql, Default)]
struct Animal {
    rowid: Option<i64>,
    name: Option<String>,
    filename: Option<String>,
    image: Option<Blob>
}

#[derive(Turbosql, Default)]
struct Sighting {
    rowid: Option<i64>,
    animal_id: Option<i64>,
    seen_at: Option<i64>
}

impl Sighting {
    fn new(animal_id:i64) -> Sighting {
        let s = Sighting {
            rowid: None,
            animal_id: Some(animal_id),
            seen_at: Some(Local::now().timestamp())
        };
        s
    }
}

fn main() {
    let application = Application::new(
        Some("com.github.gtk-rs.examples.basic"),
        Default::default(),
    ).expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let animals: Vec<Animal> = turbosql::select!(Vec<Animal>).expect("Couldn't retrieve animals");

        let window = ApplicationWindow::new(app);
        window.set_title("Bird Counter");
        window.set_default_size(350, 70);
        
        let vbox = Box::new(Orientation::Vertical, 5);
        vbox.set_homogeneous(true);
        window.add(&vbox);
        
        for y in 0..2 {
            let hbox = Box::new(Orientation::Horizontal, 5);
            hbox.set_homogeneous(true);
            vbox.add(&hbox);
            
            for i in 0..4 {
                let animal_r = animals.get((y * 4) + i);
                let img = match animal_r {
                    Some(animal) => match &animal.filename {
                        Some(filename) => Image::from_file(format!("images/{}", filename)),
                        None => Image::new()
                    },
                    None => Image::from_file("images/add_new.png")
                };
                if let Some(source_pb) = img.get_pixbuf() {
                    let sw = source_pb.get_width();
                    let sh = source_pb.get_height();
                    let mut dw = 180;
                    let mut dh = 180;
                    if sw > sh {
                        dh = 180 * sh / sw;
                    } else if sh > sw {
                        dw = 180 * sw / sh;
                    }
                    img.set_from_pixbuf(source_pb.scale_simple(dw, dh, gdk_pixbuf::InterpType::Bilinear).as_ref());
                }
                let today_count = select!(i64 "count(*) from sighting where animal_id = ? and date(seen_at, \"unixepoch\", \"localtime\") = date(\"now\")",
                    match animal_r {
                        Some(animal) => animal.rowid.unwrap_or(0),
                        None => 0
                    }).unwrap_or(0);
                if today_count > 0 {
                    add_tick(&img);
                }
                let ebox = EventBox::new();
                unsafe { ebox.set_data("animal",
                    match animal_r {
                        Some(animal) => animal.rowid.unwrap_or(0),
                        None => 0
                    });
                }
                ebox.add(&img);
                ebox.connect_button_release_event(|object, _event| {
                    // println!("Button release. {:?}, {:?}", object, event);
                    if let Some(animal_id) = unsafe { object.get_data::<i64>("animal") } {
                        if *animal_id != 0 {
                            if let Some(child) = object.get_child() {
                                if let Ok(image) = child.downcast::<gtk::Image>() {
                                    log_sighting(*animal_id);
                                    add_tick(&image);
                                }
                            }
                        } else {
                            let dialog = FileChooserDialog::with_buttons::<Window>(
                                Some("Select Bird Image File"),
                                None,
                                FileChooserAction::Open,
                                &[("_Cancel", ResponseType::Cancel), ("_Open", ResponseType::Accept)]
                            );
                            let file_filter = FileFilter::new();
                            file_filter.add_pixbuf_formats();
                            dialog.add_filter(&file_filter);
                            let name = Entry::new();
                            name.set_placeholder_text(Some("Enter bird species name"));
                            dialog.set_extra_widget(&name);
                            loop {
                                let response = dialog.run();
                                println!("Response {:?}", response);
                                println!("Filename {}", dialog.get_filename().unwrap_or(PathBuf::new()).to_str().unwrap_or(""));
                                println!("Species name {}", name.get_text());
                                if response == ResponseType::Cancel || name.get_text() != "" {
                                    if response == ResponseType::Accept {
                                        let source_file = dialog.get_filename().unwrap_or(PathBuf::new());
                                        if let Some(basename) = source_file.file_name() {
                                            let dest_file = Path::new("images").join(basename);
                                            fs::copy(&source_file, &dest_file).expect("Error copying file");
                                            let animal = Animal {
                                                rowid: None,
                                                name: Some(name.get_text().to_string()),
                                                filename: Some(basename.to_str().unwrap_or("").to_string()),
                                                image: None
                                            };
                                            let oid = animal.insert().expect("error adding animal to database");
                                            // Update GUI
                                            if let Some(child) = object.get_child() {
                                                if let Ok(image) = child.downcast::<gtk::Image>() {
                                                    let pb = gdk_pixbuf::Pixbuf::from_file_at_scale(dest_file, 180, 180, true).expect("couldn't load pixbuf from file");
                                                    image.set_from_pixbuf(Some(&pb));
                                                    // Set eventbox data
                                                    unsafe { object.set_data("animal", oid); }
                                                }
                                            }
                                        }
                                    }
                                    break;
                                }
                                let alert = MessageDialog::new(Some(&dialog), gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
                                    gtk::MessageType::Error, gtk::ButtonsType::Ok, "Please enter a species name.");
                                alert.run();
                                alert.hide();
                            }
                            dialog.hide();
                        }
                    }
                    Inhibit(false)
                });
                hbox.add(&ebox);
            }
        }

        window.show_all();
    });

    application.run(&[]);
}

fn add_tick(image: &gtk::Image) {
    let tick_pb = gdk_pixbuf::Pixbuf::from_file("images/tick.png").expect("Can't load tick image.");
    let dest_pb = image.get_pixbuf().unwrap();
    let dw = dest_pb.get_width();
    let dh = dest_pb.get_height();
    let tw = tick_pb.get_width();
    let th = tick_pb.get_height();
    let ow = dw - tw - 10;
    let oh = dh - th - 10;
    tick_pb.composite(&dest_pb, ow, oh, tw, th, ow as f64, oh as f64, 1.0, 1.0, gdk_pixbuf::InterpType::Bilinear, 255);
    image.set_from_pixbuf(Some(&dest_pb));
}

fn log_sighting(animal_id: i64) {
    let s = Sighting::new(animal_id);
    let oid = s.insert();
    println!("Sighting: {:?}", oid);
}

// To get daily bird sighting data from db...
// select distinct animal.name, date(seen_at, "unixepoch", "localtime") from sighting left join animal on animal.rowid = sighting.animal_id;