use plotters::prelude::*;

pub struct ShipComponentStats {
    pub transmitted: Vec<(f64, f64)>,
    pub sent: Vec<(f64, f64)>,
}

impl ShipComponentStats {
    pub fn new() -> Self {
        let mut transmitted: Vec<(f64, f64)> = Vec::new();
        let mut sent: Vec<(f64, f64)> = Vec::new();

        sent.push((0.0, 0.0));
        transmitted.push((0.0, 0.0));

        ShipComponentStats { transmitted, sent }
    }

    pub fn plot_stats(&self, component_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let image_name = format!("{}-stats.png", component_name);
        let root = BitMapBackend::new(&image_name, (1011, 758)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("L7 Switch bitrate performance - send", ("sans-serif", 30))
            .margin(5)
            .x_label_area_size(100)
            .y_label_area_size(100)
            .build_cartesian_2d(0f64..10000f64, 0f64..10000f64)?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(LineSeries::new(self.sent.clone(), &RED))?
            .label("sas")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .draw()?;

        Ok(())
    }
}
