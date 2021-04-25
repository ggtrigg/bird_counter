use animate::Canvas;
use charts::{Chart, LineChart, LineChartOptions};
use dataflow::*;
use gtk::prelude::*;
use turbosql::select;

#[derive(Debug, Eq, PartialEq, Clone)]
struct NameAndCountResult {
    name: Option<String>,
    week: Option<String>,
    count: Option<i64>,
}

fn get_sightings(_animal_id: Option<i64>) -> Vec<NameAndCountResult> {
    let result = select!(Vec<NameAndCountResult> r#"animal.name as name, strftime("%Y%W", seen_at, "unixepoch", "localtime") as week, count(distinct date(seen_at, "unixepoch", "localtime")) as count from sighting left join animal on animal.rowid = sighting.animal_id group by animal_id,week"#);
    match result {
        Ok(rows) => rows,
        Err(_) => Vec::new()
    }
}

pub fn testing() {
    let rows = get_sightings(None);
    println!("Got {} rows.", rows.len());
    
    for row in &rows {
        let name = row.name.clone().unwrap_or(String::from(""));
        let week = row.week.clone().unwrap_or(String::from(""));
        let count = row.count.unwrap_or(0);
        println!("Row: {} seen on week {}, count = {}", name, week, count);
    }
}


fn create_stream() -> DataStream<'static, &'static str, i32> {
    let metadata = vec![
        Channel {
            name: "Series 1",
            tag: 0,
            visible: true,
        },
        Channel {
            name: "Series 2",
            tag: 1,
            visible: true,
        },
        Channel {
            name: "Series 3",
            tag: 2,
            visible: true,
        },
    ];

    // Zero stream tag is allways metric
    let mut frames = vec![DataFrame {
        metric: "Monday",
        data: [(0, 1), (1, 3), (2, 5)].iter().cloned().collect(),
    }];

    frames.push(DataFrame {
        metric: "Tuesday",
        data: [(0, 3), (1, 4), (2, 6)].iter().cloned().collect(),
    });

    frames.push(DataFrame {
        metric: "Wednesday",
        data: [(0, 4), (1, 3), (2, 1)].iter().cloned().collect(),
    });

    frames.push(DataFrame {
        metric: "Thursday",
        data: [(1, 5), (2, 1)].iter().cloned().collect(),
    });

    frames.push(DataFrame {
        metric: "Friday",
        data: [(0, 3), (1, 4), (2, 2)].iter().cloned().collect(),
    });

    frames.push(DataFrame {
        metric: "Saturday",
        data: [(0, 5), (1, 10), (2, 4)].iter().cloned().collect(),
    });

    frames.push(DataFrame {
        metric: "Sunday",
        data: [(0, 4), (1, 12), (2, 8)].iter().cloned().collect(),
    });

    DataStream::new(metadata, frames)
}

pub fn setup_chart () -> gtk::DrawingArea {
    let drawing_area = Box::new(gtk::DrawingArea::new)();
    // let default_size = (800.0, 400.0);
    // let padding = 30.0;

    let stream = create_stream();

    let mut options: LineChartOptions = Default::default();
    options.channel.labels = Some(Default::default());
    options.channel.fill_opacity = 0.25;
    options.yaxis.min_interval = Some(2.);
    options.title.text = Some("Weekly Bird Sightings");

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