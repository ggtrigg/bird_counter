extern crate gtk;

pub mod images;
pub mod charts;

use crate::Animal;
use gtk::prelude::*;
use gtk::{MessageDialog, Window, Box, Orientation, Stack, GesturePan, DrawingArea};
use directories_next::ProjectDirs;
use std::path::Path;
use std::path::PathBuf;

pub struct Gui {
  pub images: gtk::Box,
  pub charts: gtk::Box,
  pub drawing_area: gtk::DrawingArea
}

impl Gui {
  pub fn new() -> Gui {
    let g = Gui {
      images: Box::new(Orientation::Vertical, 5),
      charts: Box::new(Orientation::Horizontal, 5),
      drawing_area: DrawingArea::new()
    };
    
    g.images.set_homogeneous(true);
    g.charts.set_homogeneous(true);
    
    g
  }
  
  pub fn build (&self, animals: Vec<Animal>) -> gtk::Widget {
    let stack = Stack::new();
    
    charts::update_chart(&self.drawing_area);
    self.charts.add(&self.drawing_area);

    stack.add_named(&self.images, "birds");
    stack.add_named(&self.charts, "charts");
    stack.set_homogeneous(true);
    stack.set_transition_type(gtk::StackTransitionType::SlideLeft);
    
    let gesture = GesturePan::new(&stack, gtk::Orientation::Horizontal);
    gesture.set_propagation_phase(gtk::PropagationPhase::Capture);
    gesture.connect_pan(move |gesture, direction, offset| {
        if offset > 50.0 {
            if let Some(widget) = gesture.get_widget() {
                if let Ok(nbook) = widget.downcast::<gtk::Stack>() {
                    match direction {
                        gtk::PanDirection::Left => nbook.set_visible_child_full("charts", gtk::StackTransitionType::SlideLeft),
                        gtk::PanDirection::Right => nbook.set_visible_child_full("birds", gtk::StackTransitionType::SlideRight),
                        _ => ()
                    }
                }
            }
        }
    });
    unsafe { stack.set_data("gesture", gesture); }

    self.load_images(&animals);
    
    // glib::timeout_add_seconds_local(300, move || {
    //     refresh_images(&self.images);
    //     charts::update_chart(&drawing_area);
    //     glib::Continue(true)
    // });

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
      message
  );
  alert.run();
  alert.hide();
}