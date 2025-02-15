use plotters::prelude::*;

pub struct ShipComponentStats {
    pub total_sent: f64,
    pub total_received: f64,
    pub bitrate_received: Vec<(f64, f64)>,
    pub bitrate_sent: Vec<(f64, f64)>,
}

impl ShipComponentStats {
    pub fn new() -> Self {
        // the first element of the tuple is the bitrate, the second is the time when it was measured
        let mut bitrate_received: Vec<(f64, f64)> = Vec::new();
        let mut bitrate_sent: Vec<(f64, f64)> = Vec::new();

        bitrate_sent.push((0.0, 0.0));
        bitrate_received.push((0.0, 0.0));

        ShipComponentStats {
            total_sent: 0.0,
            total_received: 0.0,
            bitrate_received,
            bitrate_sent,
        }
    }

    pub fn plot_sent_stats(&self, component_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let image_name = format!("./test/imgs/{}-stats.png", component_name);
        let root = BitMapBackend::new(&image_name, (1011, 758)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("L7 Switch Performance", ("sans-serif", 20))
            .margin(5)
            .x_label_area_size(100)
            .y_label_area_size(100)
            .build_cartesian_2d(0f64..20f64, 0f64..250f64)?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(self.bitrate_sent.clone(), &RED))?
            .label("Sending MB/s")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
        chart
            .draw_series(LineSeries::new(self.bitrate_received.clone(), &BLUE))?
            .label("Transmission MB/s")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .draw()?;

        Ok(())
    }
}
