use plotters::prelude::*;

pub struct ShipComponentStats {
    pub total_sent: f64,
    pub total_transmitted: f64,
    pub bitrate_transmitted: Vec<(f64, f64)>,
    pub bitrate_sent: Vec<(f64, f64)>,
}

impl ShipComponentStats {
    pub fn new() -> Self {
        // the first element of the tuple is the bitrate, the second is the time when it was measured
        let mut bitrate_transmitted: Vec<(f64, f64)> = Vec::new();
        let mut bitrate_sent: Vec<(f64, f64)> = Vec::new();

        bitrate_sent.push((0.0, 0.0));
        bitrate_transmitted.push((0.0, 0.0));

        ShipComponentStats {
            total_sent: 0.0,
            total_transmitted: 0.0,
            bitrate_transmitted,
            bitrate_sent,
        }
    }

    pub fn plot_sent_stats(&self, component_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let image_name = format!("./test/imgs/{}-stats.png", component_name);
        let root = BitMapBackend::new(&image_name, (1011, 758)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("L7 Switch bitrate performance - send", ("sans-serif", 30))
            .margin(5)
            .x_label_area_size(100)
            .y_label_area_size(100)
            .build_cartesian_2d(0f64..20f64, 0f64..1500f64)?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(self.bitrate_sent.clone(), &RED))?
            .label("Sending Bitrate B/s")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
        chart
            .draw_series(LineSeries::new(self.bitrate_transmitted.clone(), &BLUE))?
            .label("Transmission Bitrate B/s")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .draw()?;

        Ok(())
    }
}
