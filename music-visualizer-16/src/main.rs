use plotters::prelude::*;
use rodio::{Decoder, source::Source};
use std::fs::File;

fn abs_max(samples: &[f32]) -> f32 {
    samples.iter().map(|x| x.abs()).fold(0.0_f32, f32::max)
}

fn main() {
    let groups = 50;

    let file = File::open("output3.wav").unwrap();

    let source = Decoder::try_from(file).unwrap();

    println!("Channels: {}", source.channels());
    println!("Sample rate: {}", source.sample_rate());
    println!("Duration: {:#?}", source.total_duration().unwrap());
    println!("{}", source.current_span_len().unwrap());

    let samples: Vec<f32> = source.collect();
    println!("len before {}", samples.len());
    let mut minmax = vec![];
    for chunk in samples.chunks(groups) {
        let max = abs_max(chunk);
        minmax.push(max);
    }

    println!("len after {}", minmax.len());


    // plotting
    let width = 1200;
    let height = 400;
    let root = BitMapBackend::new("mirrored_waveform.png", (width, height)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    // Y-axis from -max_value to +max_value
    let max_value = minmax.iter().copied().fold(0.0_f32, f32::max);

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption("Mirrored Waveform", ("sans-serif", 30))
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(
            0..minmax.len(),
            -max_value..max_value, // Mirrored range
        )
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    // Draw mirrored waveform as a line
    chart
        .draw_series(LineSeries::new(
            minmax.iter().enumerate().map(|(i, v)| (i, *v)),
            &BLUE,
        ))
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            minmax.iter().enumerate().map(|(i, v)| (i, -*v)), // mirrored
            &BLUE,
        ))
        .unwrap();

    root.present().unwrap();
}