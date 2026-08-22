use cpal::{
    Device, Stream, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rtrb::{Consumer, Producer, RingBuffer};

/// The AudioDevice owns a stream to the actual audio device
/// it reads samples from a Consumer per channel and interleaves
/// these while writing to the audio stream
pub struct AudioDevice {
    pub sample_rate: u32,
    device: Device,
    config: SupportedStreamConfig,
    channels: usize,
    stream: Option<Stream>,
    consumers: Vec<Consumer<f32>>,
}

impl AudioDevice {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");

        let config = device
            .default_output_config()
            .expect("failed to get default output config");

        let channels = config.channels() as usize;
        let sample_rate = config.sample_rate();

        Self {
            device: device,
            config: config,
            channels: channels,
            sample_rate: sample_rate,
            stream: None,
            consumers: vec![],
        }
    }

    /// Creates a RingBuffer and returns the Producer for that RingBuffer
    /// This function should called for each output channel
    pub fn take_producer(&mut self) -> Producer<f32> {
        let buffer_size = self.sample_rate as usize * 10; // 10 s buffer
        let (producer, consumer): (Producer<f32>, Consumer<f32>) = RingBuffer::new(buffer_size);
        self.consumers.push(consumer);
        return producer;
    }

    /// Opens the audio stream and attached the closure that writes the data
    pub fn play(&mut self) {
        self.stream = Some(self.create_stream_and_play());
    }

    fn create_stream_and_play(&mut self) -> Stream {
        let channels = self.channels;
        let mut consumers = std::mem::take(&mut self.consumers);
        let mut values = vec![0.0; consumers.len()];
        let stream = self
            .device
            .build_output_stream(
                self.config.into(),
                move |output: &mut [f32], _| {
                    for frame in output.chunks_mut(channels) {
                        // make sure to consume all the buffers
                        for i in 0..values.len() {
                            values[i] = consumers[i].pop().unwrap_or(0.0);
                        }

                        let mut channel = 0;
                        for sample in frame {
                            if channel < consumers.len() {
                                *sample = values[channel];
                            }
                            channel += 1;
                        }
                    }
                },
                |err| eprintln!("{err}"),
                None,
            )
            .expect("error");
        stream.play().expect("error");

        stream
    }
}
