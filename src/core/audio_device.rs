use cpal::{
    Device, Host, Stream, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rtrb::Consumer;

pub struct AudioDevice {
    host: Host,
    device: Device,
    config: SupportedStreamConfig,
    channels: usize,
    stream: Option<Stream>,
}

impl AudioDevice {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");

        let config = device
            .default_output_config()
            .expect("could not create default output");

        let channels = config.channels() as usize;

        Self {
            host: cpal::default_host(),
            device: device,
            config: config,
            channels: channels,
            stream: None,
        }
    }

    pub fn connect_consumer(&mut self, mut consumer: Consumer<f32>) {
        let channels = self.channels;

        self.stream = Some(
            self.device
                .build_output_stream(
                    self.config.into(),
                    move |output: &mut [f32], _| {
                        for frame in output.chunks_mut(channels) {
                            for sample in frame {
                                let value = consumer.pop().unwrap_or(0.0);
                                *sample = value;
                            }
                        }
                    },
                    |err| eprintln!("{err}"),
                    None,
                )
                .expect("error"),
        );
    }

    pub fn play(&mut self) {
        self.stream.as_mut().unwrap().play().expect("error");
        println!("after play");
    }
}
