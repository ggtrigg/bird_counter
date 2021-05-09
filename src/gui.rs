extern crate gtk;
extern crate gdk;
extern crate cairo;

use crate::Animal;
use directories_next::ProjectDirs;
use gtk::prelude::*;
use gdk::prelude::*;
use gtk::{
    Box, Entry, EventBox, FileChooserAction, FileChooserDialog, FileFilter, MessageDialog,
    Orientation, ResponseType, Window, DrawingArea, GestureLongPress
};
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
            ebox.connect_event(move |widget, event| {
               let event_type = event.get_event_type();
               if event_type == gdk::EventType::ButtonPress || event_type == gdk::EventType::TouchBegin {
                   let coords = event.get_coords().unwrap_or((0.0, 0.0));
                   unsafe { widget.set_data("last_coords", (coords.0, coords.1, event.get_time())) };
               } else if event_type == gdk::EventType::ButtonRelease || event_type == gdk::EventType::TouchEnd {
                    if let Some(last_coords) = unsafe { widget.get_data::<(f64, f64, u32)>("last_coords") } {
                        let coords = event.get_coords().unwrap_or((0.0, 0.0));
                        if (last_coords.0 == coords.0) && (last_coords.1 == coords.1) && (event.get_time() - last_coords.2 < 500) {
                            animal_selected(widget);
                        }
                    }
               }
               Inhibit(false)
            });
            hbox.pack_start(&ebox, true, true, 0);
            
            // Add long press gesture to clear selection.
            let gesture = GestureLongPress::new(&ebox);
            gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
            gesture.connect_pressed(|gesture, _x, _y| {
               if let Some(widget) = gesture.get_widget() {
                   if let Ok(eventbox) = widget.downcast::<gtk::EventBox>() {
                       if let Some(animal_id) = unsafe { eventbox.get_data::<i64>("animal") } {
                           crate::clear_sighting(*animal_id);
                       }
                       if let Some(widget) = eventbox.get_parent() {
                           if let Some(w2) = widget.get_parent() {
                               if let Ok(vbox) = w2.downcast::<gtk::Box>() {
                                   refresh_images(&vbox);
                               }
                           }
                       }
                   }
                }
            });
            unsafe { ebox.set_data("gesture", gesture); }
        }
    }
}

pub fn image_dir() -> PathBuf {
    let image_dir = ProjectDirs::from("org", "glenntrigg", "bird_counter")
    .unwrap()
    .data_dir()
    .to_owned()
    .join(Path::new("images"));

    image_dir
}

fn animal_selected(object: &EventBox) {
    if let Some(animal_id) = unsafe { object.get_data::<i64>("animal") } {
        if *animal_id != 0 {
            if let Some(child) = object.get_child() {
                if let Ok(da) = child.downcast::<gtk::DrawingArea>() {
                    crate::log_sighting(*animal_id);
                    da.queue_draw();
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
                                .map_err(|e| alert(&format!("Error copying file\n{}", e)[..])).ok();
                            let animal = Animal {
                                rowid: None,
                                name: Some(name.get_text().to_string()),
                                filename: Some(
                                    basename.to_str().unwrap_or("").to_string(),
                                ),
                                image: None,
                            };
                            if let Some(oid) = animal
                                .insert()
                                .map_err(|e| alert(&format!("Error adding animal to database\n{}", e)[..])).ok() {

                                unsafe {
                                    object.set_data("animal", oid);
                                }
                                // Update GUI
                                if let Some(child) = object.get_child() {
                                    if let Ok(da) = child.downcast::<gtk::DrawingArea>() {
                                        da.show();
                                    }
                                }
                            }
                        }
                    }
                    break;
                }
                alert("Please enter a species name.");
            }
            dialog.hide();
        }
    }
}

fn load_image(_animal: Option<&Animal>) -> gtk::DrawingArea {
    let da = DrawingArea::new();
    da.set_size_request(180, 180);
    da.connect_draw(draw_image);
    da
}

fn draw_image(da: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
    if let Some(eventbox) = da.get_parent() {
        if let Some(animal_id) = unsafe { eventbox.get_data::<i64>("animal") } {
            let a_width = da.get_allocated_width();
            let a_height = da.get_allocated_height();
            let diff = a_width - a_height;
            let pb = get_animal_pixbuf(animal_id, a_width, a_height);
            let mut x_offset = 0.0;
            let mut y_offset = 0.0;
            if diff < 0 {
                y_offset = -diff as f64 / 2.0;
            } else {
                x_offset = diff as f64 / 2.0;
            }
            context.set_source_pixbuf(&pb, x_offset, y_offset);
            context.paint();
        }
    }
    Inhibit(false)
}

fn get_animal_pixbuf(animal_id: &i64, width: i32, height: i32) -> gdk_pixbuf::Pixbuf {
    let mut pb = gdk_pixbuf::Pixbuf::from_file_at_scale(image_dir().join(Path::new("unknown.png")), width, height, true)
                .map_err(|e| alert(&format!("Couldn't load pixbuf from file\n{}", e)[..])).ok()
                .unwrap_or(gdk_pixbuf::Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, 180, 180).unwrap());

    if let Ok(animal) = select!(Animal "where rowid = ?", animal_id) {
        let res = match &animal.filename {
            Some(filename) => gdk_pixbuf::Pixbuf::from_file_at_scale(image_dir().join(Path::new(filename)), width, height, true),
            None => gdk_pixbuf::Pixbuf::from_file_at_scale(image_dir().join(Path::new("unknown.png")), width, height, true)
        };
        pb = res
            .map_err(|e| alert(&format!("Couldn't load pixbuf from file\n{}", e)[..])).ok()
            .unwrap_or(gdk_pixbuf::Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, 180, 180).unwrap());
        if let Ok(today_count) = select!(i64 "count(*) from sighting where animal_id = ? and date(seen_at, \"unixepoch\", \"localtime\") = date(\"now\", \"localtime\")",
                animal_id) {
            if today_count > 0 {
                add_tick(&pb);
            }
        }
    }
    pb
}

fn add_tick(dest_pb: &gdk_pixbuf::Pixbuf) {
    if let Some(tick_pb) = gdk_pixbuf::Pixbuf::from_file(&image_dir().join(Path::new("tick.png")))
            .map_err(|e| alert(&format!("Couldn't load pixbuf from file\n{}", e)[..])).ok() {
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
    }
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

pub fn alert(message: &str) -> () {
    let alert = MessageDialog::new::<Window>(
        None,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        gtk::MessageType::Error,
        gtk::ButtonsType::Ok,
        message
    );
    alert.run();
    alert.hide();
}