use plotters::prelude::*;
use rodio::Decoder;
use std::fs::File;

fn abs_max(samples: &[f32]) -> f32 {
    samples.iter().map(|x| x.abs()).fold(0.0_f32, f32::max)
}

fn main() {
    // Ensure output3.wav exists in your project directory
    let file = File::open("output3.wav").expect("Could not open output3.wav");
    let source = Decoder::try_from(file).expect("Could not decode audio file");

    let samples: Vec<f32> = source.collect();
    
    // --- Data Preparation ---

    // 1. Determine resolution: Set exactly how many visual bars we want.
    let num_bars = 150; 
    // Avoid division by zero if file is empty
    if samples.is_empty() { panic!("Audio file is empty"); }
    let groups = (samples.len() / num_bars).max(1);

    let mut minmax = vec![];
    for chunk in samples.chunks(groups) {
        let max = abs_max(chunk);
        minmax.push(max);
    }

    // Ensure we have data to plot
    if minmax.is_empty() { return; }
    let max_value = minmax.iter().copied().fold(0.0_f32, f32::max).max(0.01); // Ensure non-zero range

    // --- Plotting Setup ---

    let width = 1200;
    let height = 400;
    let root = BitMapBackend::new("visualizer.png", (width, height)).into_drawing_area();
    
    // Aesthetic: Dark mode background
    root.fill(&RGBColor(15, 15, 20)).unwrap(); 

    // Define colors
    let neon_cyan = RGBColor(0, 255, 255);
    // A subtle grey for the center line so it doesn't overpower the bars
    let horizon_grey = RGBColor(100, 100, 100); 

    // Clean Chart (No grid lines, no axes displayed)
    let mut chart = ChartBuilder::on(&root)
        // Set X-axis as a float range so we can manage spacing cleanly
        .build_cartesian_2d(0.0..(num_bars as f32), -max_value..max_value) 
        .unwrap();

    // --- Drawing ---

    // 1. Draw Discrete Bars
    for (i, &val) in minmax.iter().enumerate() {
        // Create a gap between the bars (bar takes up from x.1 to x.9)
        let x0 = i as f32 + 0.1; 
        let x1 = i as f32 + 0.9; 

        // Draw a rectangle spanning from positive peak to negative peak (centered on 0)
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(x0, val), (x1, -val)],
                neon_cyan.filled(),
            )))
            .unwrap();
    }

    // 2. Draw Center Horizon Line
    // We draw a line from X=0 to X=end at exactly Y=0.
    chart
        .draw_series(LineSeries::new(
            vec![(0.0, 0.0), (num_bars as f32, 0.0)],
            horizon_grey.stroke_width(1), // A thin, subtle line
        ))
        .unwrap();

    root.present().unwrap();
    println!("Visualizer image created successfully!");
}