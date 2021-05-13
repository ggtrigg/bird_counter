extern crate gtk;

pub mod charts;
pub mod images;

use crate::Animal;
use directories_next::ProjectDirs;
use gdk::prelude::*;
use gtk::prelude::*;
use gtk::{
    Box, DrawingArea, GesturePan, GestureZoom, MessageDialog, Orientation,
    Stack, Window,
};
use std::path::Path;
use std::path::PathBuf;

pub struct Gui {
    pub window: gtk::ApplicationWindow,
    pub images: gtk::Box,
    pub charts: gtk::Box,
    pub drawing_area: gtk::DrawingArea,
}

impl Gui {
    pub fn new(window: gtk::ApplicationWindow) -> Gui {
        let g = Gui {
            window,
            images: Box::new(Orientation::Vertical, 5),
            charts: Box::new(Orientation::Horizontal, 5),
            drawing_area: DrawingArea::new(),
        };

        g.images.set_homogeneous(true);
        g.charts.set_homogeneous(true);

        g
    }

    pub fn build(&self, animals: Vec<Animal>) -> gtk::Widget {
        let stack = Stack::new();

        charts::update_chart(&self.drawing_area);
        self.charts.add(&self.drawing_area);

        stack.add_named(&self.images, "birds");
        stack.add_named(&self.charts, "charts");
        stack.set_homogeneous(true);
        stack.set_transition_type(gtk::StackTransitionType::SlideLeft);

        let pan = GesturePan::new(&stack, gtk::Orientation::Horizontal);
        pan.set_propagation_phase(gtk::PropagationPhase::Capture);
        pan.connect_pan(move |gesture, direction, offset| {
            if offset > 50.0 {
                if let Some(widget) = gesture.get_widget() {
                    if let Ok(nbook) = widget.downcast::<gtk::Stack>() {
                        match direction {
                            gtk::PanDirection::Left => nbook.set_visible_child_full(
                                "charts",
                                gtk::StackTransitionType::SlideLeft,
                            ),
                            gtk::PanDirection::Right => nbook.set_visible_child_full(
                                "birds",
                                gtk::StackTransitionType::SlideRight,
                            ),
                            _ => (),
                        }
                    }
                }
            }
        });
        unsafe {
            stack.set_data("pan_gesture", pan);
        }

        let window = self.window.clone();
        let zoom = GestureZoom::new(&stack);
        zoom.set_propagation_phase(gtk::PropagationPhase::Capture);
        zoom.connect_scale_changed(move |_gesture, zoom_amount| {
            if zoom_amount > 1.8 {
                window.fullscreen();
            } else if zoom_amount < 0.7 {
                window.unfullscreen();
            }
        });
        unsafe {
            stack.set_data("zoom_gesture", zoom);
        }

        self.load_images(&animals);

        stack.upcast::<gtk::Widget>()
    }

    pub fn refresh(&self) -> () {
        self.refresh_images();
        charts::update_chart(&self.drawing_area);
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

pub fn alert(message: &str) -> () {
    let alert = MessageDialog::new::<Window>(
        None,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        gtk::MessageType::Error,
        gtk::ButtonsType::Ok,
        message,
    );
    alert.run();
    alert.hide();
}
