use std::iter::{FromIterator, Iterator};
use animate::Canvas;
use charts::{Chart, LineChart, LineChartOptions};
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

fn create_stream<'a>(namedata: &'a Vec<ChannelData>, weeks: &Vec<u32>, week_labels: &'a Vec<String>)
    -> DataStream<'a, &'a str, i32> {
    let mut metadata = Vec::new();
    for cdata in namedata {
        metadata.push(Channel {
           name: &cdata.name.clone().unwrap()[..],
           tag: cdata.tag.unwrap(),
           visible: true 
        });
    }

    let mut sdata: Vec<Vec<u32>> = vec![vec![0; weeks.len() as usize]; metadata.len()];

    for (i, animal_id) in namedata.iter().enumerate() {
        let sightings = get_sightings(animal_id.tag.unwrap() as u32);
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
           metric: &week_labels[i][..],
           data: imap
        });
    }

    DataStream::new(metadata, frames)
}

pub fn setup_chart() -> gtk::DrawingArea {
    let drawing_area = Box::new(gtk::DrawingArea::new)();
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

    // let default_size = (800.0, 400.0);
    // let padding = 30.0;

    let stream = create_stream(&an, &weeks, &week_labels);

    let mut options: LineChartOptions = Default::default();
    options.channel.labels = Some(Default::default());
    options.channel.fill_opacity = 0.25;
    options.yaxis.min_interval = Some(2.);
    options.title.text = Some("Weekly Bird Sightings");
    options.xaxis.title.text = Some("Weeks Ago");

    let mut chart = LineChart::new(options);
    chart.set_stream(stream);

    drawing_area.connect_draw(move |area, cr| {
        let (rect, _) = area.get_allocated_size();
        let size = (rect.width as f64, rect.height as f64);
        // let chart_area: (f64, f64) = (size.0 - padding * 2.0, size.1 - padding * 2.0);

        chart.resize(size.0, size.1);

        let ctx = Canvas::new(cr); // overhead
        chart.draw(&ctx);

        Inhibit(false)
    });
    
    drawing_area
}