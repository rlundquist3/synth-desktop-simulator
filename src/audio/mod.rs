use cpal::{
    BufferSize, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::{cell::RefCell, sync::Mutex};
use synth_core::{SAMPLE_RATE, engines::fm::FMSynth};

use crate::log;

pub async fn audio_handler(engine: &'static Mutex<RefCell<FMSynth>>) {
    let device = cpal::default_host()
        .default_output_device()
        .expect("Should find default audio device");

    let config = StreamConfig {
        channels: 2,
        sample_rate: SAMPLE_RATE,
        buffer_size: BufferSize::Default,
    };

    let stream = device
        .build_output_stream(
            config,
            move |output: &mut [f32], _| {
                let e = engine.lock().unwrap();
                audio_output(&mut e.borrow_mut(), output, config.channels as usize);
            },
            |err| log::push(format!("audio stream error: {err}")),
            None,
        )
        .unwrap_or_else(|err| {
            panic!("Audio device does not support {SAMPLE_RATE}Hz f32 output: {err}")
        });

    stream.play().unwrap();
    log::push(format!("Audio initialized at {SAMPLE_RATE}Hz"));

    // Hold for lifetime of task
    std::future::pending::<()>().await;
}

fn audio_output(engine: &mut FMSynth, output: &mut [f32], channels: usize) {
    output.chunks_mut(channels).for_each(|frame| {
        frame.fill(engine.next().unwrap_or(0.0));
    });
}
