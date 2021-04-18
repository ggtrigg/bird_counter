extern crate gtk;
extern crate gdk;

use crate::Animal;
use directories_next::ProjectDirs;
use gtk::prelude::*;
use gtk::{
    Box, Entry, EventBox, FileChooserAction, FileChooserDialog, FileFilter, Image, MessageDialog,
    Orientation, ResponseType, Window,
};
use gdk::EventConfigure;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use turbosql::select;


pub fn load_images(vbox: &Box, animals: &Vec<Animal>) {
    for y in 0..2 {
        let hbox = Box::new(Orientation::Horizontal, 5);
        hbox.set_homogeneous(true);

        vbox.add(&hbox);
        for i in 0..4 {
            let animal_r = animals.get((y * 4) + i);
            let img = load_image(animal_r);
            let ebox = EventBox::new();
            unsafe {
                ebox.set_data(
                    "animal",
                    match animal_r {
                        Some(animal) => animal.rowid.unwrap_or(0),
                        None => 0,
                    },
                );
            }
            ebox.add(&img);
            ebox.set_above_child(true);
            ebox.connect_button_release_event(animal_selected);
            hbox.pack_start(&ebox, true, true, 0);
        }
    }
}

fn image_dir() -> PathBuf {
    let image_dir = ProjectDirs::from("org", "glenntrigg", "bird_counter")
    .unwrap()
    .data_dir()
    .to_owned()
    .join(Path::new("images"));

    image_dir
}

fn animal_selected(object: &EventBox, _event: &gdk::EventButton) -> gtk::Inhibit {
    if let Some(animal_id) = unsafe { object.get_data::<i64>("animal") } {
        if *animal_id != 0 {
            if let Some(child) = object.get_child() {
                if let Ok(image) = child.downcast::<gtk::Image>() {
                    crate::log_sighting(*animal_id);
                    add_tick(&image);
                }
            }
        } else {
            let dialog = FileChooserDialog::with_buttons::<Window>(
                Some("Select Bird Image File"),
                None,
                FileChooserAction::Open,
                &[
                    ("_Cancel", ResponseType::Cancel),
                    ("_Open", ResponseType::Accept),
                ],
            );
            let file_filter = FileFilter::new();
            file_filter.add_pixbuf_formats();
            dialog.add_filter(&file_filter);
            let name = Entry::new();
            name.set_placeholder_text(Some("Enter bird species name"));
            dialog.set_extra_widget(&name);
            loop {
                let response = dialog.run();
                if response == ResponseType::Cancel || name.get_text() != "" {
                    if response == ResponseType::Accept {
                        let source_file =
                            dialog.get_filename().unwrap_or(PathBuf::new());
                        if let Some(basename) = source_file.file_name() {
                            let dest_file = image_dir().join(basename);
                            fs::copy(&source_file, &dest_file)
                                .expect("Error copying file");
                            let animal = Animal {
                                rowid: None,
                                name: Some(name.get_text().to_string()),
                                filename: Some(
                                    basename.to_str().unwrap_or("").to_string(),
                                ),
                                image: None,
                            };
                            let oid = animal
                                .insert()
                                .expect("error adding animal to database");
                            // Update GUI
                            if let Some(child) = object.get_child() {
                                if let Ok(image) = child.downcast::<gtk::Image>() {
                                    let pb = gdk_pixbuf::Pixbuf::from_file_at_scale(
                                        dest_file, 180, 180, true,
                                    )
                                    .expect("couldn't load pixbuf from file");
                                    image.set_from_pixbuf(Some(&pb));
                                    // Set eventbox data
                                    unsafe {
                                        object.set_data("animal", oid);
                                    }
                                }
                            }
                        }
                    }
                    break;
                }
                let alert = MessageDialog::new(
                    Some(&dialog),
                    gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
                    gtk::MessageType::Error,
                    gtk::ButtonsType::Ok,
                    "Please enter a species name.",
                );
                alert.run();
                alert.hide();
            }
            dialog.hide();
        }
    }
    Inhibit(true)
}

fn load_image(animal: Option<&Animal>) -> gtk::Image {
    let img = match animal {
        Some(animal) => match &animal.filename {
            Some(filename) => Image::from_file(image_dir().join(Path::new(filename))),
            None => Image::from_file(image_dir().join(Path::new("unknown.png"))),
        },
        None => Image::from_file(image_dir().join(Path::new("add_new.png"))),
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
        img.set_from_pixbuf(
            source_pb
                .scale_simple(dw, dh, gdk_pixbuf::InterpType::Bilinear)
                .as_ref(),
        );
    }
    let today_count = select!(i64 "count(*) from sighting where animal_id = ? and date(seen_at, \"unixepoch\", \"localtime\") = date(\"now\")",
    match animal {
        Some(animal) => animal.rowid.unwrap_or(0),
        None => 0
    }).unwrap_or(0);
    if today_count > 0 {
        add_tick(&img);
    }
    img
}

fn add_tick(image: &gtk::Image) {
    let tick_pb = gdk_pixbuf::Pixbuf::from_file(&image_dir().join(Path::new("tick.png")))
        .expect("Can't load tick image.");
    let dest_pb = image.get_pixbuf().unwrap();
    let dw = dest_pb.get_width();
    let dh = dest_pb.get_height();
    let tw = tick_pb.get_width();
    let th = tick_pb.get_height();
    let ow = dw - tw - 10;
    let oh = dh - th - 10;
    tick_pb.composite(
        &dest_pb,
        ow,
        oh,
        tw,
        th,
        ow as f64,
        oh as f64,
        1.0,
        1.0,
        gdk_pixbuf::InterpType::Bilinear,
        255,
    );
    image.set_from_pixbuf(Some(&dest_pb));
}

pub fn refresh_images(vbox: &gtk::Box) -> glib::Continue {
    vbox.foreach(|child| {
        if let Some(hbox) = child.downcast_ref::<gtk::Box>() {
            hbox.foreach(|child| {
                if let Some(eventbox) = child.downcast_ref::<gtk::EventBox>() {
                    if let Some(animal_id) = unsafe { eventbox.get_data::<i64>("animal") } {
                        if animal_id > &0 {
                            if let Some(img) = eventbox.get_child() {
                                unsafe { img.destroy() };
                            }
                            if let Ok(animal) = select!(Animal "where rowid = ?", animal_id) {
                                let img = load_image(Some(&animal));
                                eventbox.add(&img);
                            }
                        }
                    }
                }
            });
        }
    });
    vbox.show_all();
    glib::Continue(true)
}

fn _detect_resize(_widget: &Image, event: &EventConfigure) -> bool {
    println!("In detect_resize(), event: {:?}", event.get_size());
    false
}