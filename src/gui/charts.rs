use std::iter::{FromIterator, Iterator};
// use crate::gui;
use primitives::colorspace::prelude::*;
use charts::{Chart, BarChart, BarChartOptions, Position, Fill};
use animate::Canvas;
use dataflow::*;
use intmap::IntMap;
use chrono::prelude::*;
use gtk::prelude::*;
use turbosql::select;

#[derive(Debug, Eq, PartialEq, Clone)]
struct ChannelData {
    name: Option<String>,
    tag: Option<u8>
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct WeekAndCountResult {
    week: Option<String>,
    count: Option<u32>,
}


fn get_sightings(animal_id: u32) -> Vec<WeekAndCountResult> {
    let result = select!(Vec<WeekAndCountResult> r#"strftime("%Y%W", seen_at, "unixepoch", "localtime") as week, count(distinct date(seen_at, "unixepoch", "localtime")) as count from sighting where animal_id = ? group by week"#, animal_id);
    match result {
        Ok(rows) => rows,
        Err(_) => Vec::new()
    }
}

fn create_stream() -> DataStream<String, i32> {
    let mut metadata = Vec::new();
    let an = turbosql::select!(Vec<ChannelData> "name, rowid as tag from animal").expect("Couldn't retrieve animals");
    
    // Get the start and end (current) week values, and derive the x-axis week labels
    let first_week: String = select!(String r#"min(strftime("%Y%W", seen_at, "unixepoch", "localtime")) as week from sighting"#).expect("Error accesing db");
    let this_week = Local::now().format("%Y%W").to_string();
    let first_week_n = first_week.parse::<u32>().unwrap();
    let this_week_n = this_week.parse::<u32>().unwrap();
    let weeks = Vec::from_iter(first_week_n..=this_week_n);
    let week_labels: Vec<String> = weeks.iter().enumerate().map(|(i, _)| {
        match i {
            0 => String::from("This week"),
            1 => String::from("Last week"),
            _ => (i as u32 + 1).to_string()
        }
    }).rev().collect();
    
    for cdata in an {
        metadata.push(Channel {
           name: cdata.name.clone().unwrap(),
           tag: cdata.tag.unwrap(),
           visible: true 
        });
    }

    let mut sdata: Vec<Vec<u32>> = vec![vec![0; weeks.len() as usize]; metadata.len()];

    for (i, channel) in metadata.iter().enumerate() {
        let sightings = get_sightings(channel.tag as u32);
        for sighting in sightings {
            if let Some(week) = sighting.week {
                let week_num = week.parse::<u32>().unwrap();
                let j: usize = (week_num - weeks.first().unwrap()) as usize;
                sdata[i][j] = sighting.count.unwrap_or(0);
            }
        }
    }

    let mut frames = Vec::new();
    for (i, _weeknum) in weeks.iter().enumerate() {
        let mut imap: IntMap<i32> = IntMap::with_capacity(metadata.len());
        for bnum in 0..metadata.len() {
            imap.insert(bnum as u64, sdata[bnum][i] as i32);
        }
        frames.push(DataFrame {
           metric: week_labels[i].clone(),
           data: imap
        });
    }

    DataStream::new(metadata, frames)
}

pub fn update_chart(drawing_area: &gtk::DrawingArea) -> () {
    let stream = create_stream();
    let mut options: BarChartOptions = Default::default();
    options.channel.labels = Some(Default::default());
    options.yaxis.min_interval = Some(1.);
    options.title.text = Some("Weekly Bird Sightings".to_string());
    options.xaxis.title.text = Some("Weeks Ago".to_string());
    options.legend.position = Position::Top;
    options.legend.label_formatter = Some(charts::default_label_formatter);
    options.legend.style = Default::default();
    // These colours are from the OpenOffice charts palette.
    options.colors = vec![
        Fill::Solid(Color::rgb(0x00, 0x45, 0x86)),
        Fill::Solid(Color::rgb(0xff, 0x42, 0x0e)),
        Fill::Solid(Color::rgb(0xff, 0xd3, 0x20)),
        Fill::Solid(Color::rgb(0x57, 0x9d, 0x1c)),
        Fill::Solid(Color::rgb(0x7e, 0x00, 0x21)),
        Fill::Solid(Color::rgb(0x83, 0xca, 0xff)),
        Fill::Solid(Color::rgb(0x31, 0x40, 0x04)),
        Fill::Solid(Color::rgb(0xae, 0xcf, 0x00))
    ];

    let mut chart = BarChart::new(options);
    chart.set_stream(stream);
    
    drawing_area.connect_draw(move |area, cr| {
        let (rect, _) = area.get_allocated_size();
        let size = (rect.width as f64, rect.height as f64);

        chart.resize(size.0, size.1);

        let ctx = Canvas::new(cr); // overhead
        chart.draw(&ctx);

        Inhibit(false)
    });
}