use rtrb::{Consumer, Producer};

#[derive(Debug)]
pub enum PortError {
    NotConnected,
    AlreadyConnected,
    BufferFull,
    BufferEmpty,
}

pub struct OutputPort {
    producer: Option<Producer<f32>>,
}

impl OutputPort {
    pub fn new() -> Self {
        OutputPort { producer: None }
    }

    pub fn set_producer(&mut self, producer: Producer<f32>) -> Result<(), PortError> {
        if self.producer.is_some() {
            return Err(PortError::AlreadyConnected);
        }

        self.producer = Some(producer);

        Ok(())
    }

    pub fn remove_producer(&mut self) -> Result<(), PortError> {
        if self.producer.is_none() {
            return Err(PortError::NotConnected);
        }

        self.producer = None;

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.producer.is_some()
    }

    pub fn write_if_connected(&mut self, value: f32) -> Result<(), PortError> {
        if self.is_connected() {
            self.write(value)
        } else {
            Ok(())
        }
    }

    fn write(&mut self, value: f32) -> Result<(), PortError> {
        match self.producer.as_mut() {
            None => Err(PortError::NotConnected),
            Some(p) => {
                p.push(value).map_err(|_| PortError::BufferFull)?;
                Ok(())
            }
        }
    }
}

pub struct InputPort {
    consumer: Option<Consumer<f32>>,
}

impl InputPort {
    pub fn new() -> Self {
        InputPort { consumer: None }
    }

    pub fn set_consumer(&mut self, consumer: Consumer<f32>) -> Result<(), PortError> {
        if self.consumer.is_some() {
            return Err(PortError::AlreadyConnected);
        }

        self.consumer = Some(consumer);

        Ok(())
    }

    pub fn remove_consumer(&mut self) -> Result<(), PortError> {
        if self.consumer.is_none() {
            return Err(PortError::NotConnected);
        }

        self.consumer = None;

        Ok(())
    }

    pub fn read(&mut self) -> Result<f32, PortError> {
        match self.consumer.as_mut() {
            None => Err(PortError::NotConnected),
            Some(p) => {
                let data = p.pop().map_err(|_| PortError::BufferEmpty)?;
                Ok(data)
            }
        }
    }
}

#[cfg(test)]
mod output_port_tests {
    use super::*;
    use rtrb::RingBuffer;

    #[test]
    fn set_producer_stores_producer_in_port() {
        let (producer, _): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let mut sut = OutputPort::new();

        let _ = sut.set_producer(producer);

        assert!(sut.producer.is_some());
    }

    #[test]
    fn remove_producer_removes_producer_in_port() {
        let (producer, _): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let mut sut = OutputPort::new();

        let _ = sut.set_producer(producer);
        let _ = sut.remove_producer();

        assert!(sut.producer.is_none());
    }

    #[test]
    fn set_producer_returns_error_if_already_set() {
        let (producer1, _): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let (producer2, _): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let mut sut = OutputPort::new();

        let _ = sut.set_producer(producer1);
        let result = sut.set_producer(producer2);

        assert!(matches!(result, Err(PortError::AlreadyConnected)));
    }

    #[test]
    fn write_to_unconnected_port_returns_error() {
        let mut sut = OutputPort::new();

        let result = sut.write(42.0);

        assert!(matches!(result, Err(PortError::NotConnected)));
    }

    #[test]
    fn write_writes_to_buffer() {
        let (producer1, mut consumer): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let mut sut = OutputPort::new();

        let _ = sut.set_producer(producer1);
        sut.write(42.0).expect("write failed unexpectedly");

        let consumed_data = consumer.pop().unwrap();

        assert_eq!(consumed_data, 42.0);
    }

    #[test]
    fn write_to_full_buffer_returns_error() {
        let (producer1, _): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let mut sut = OutputPort::new();

        sut.set_producer(producer1)
            .expect("set_producer failed unexpectedly");
        sut.write(42.0).expect("write failed unexpectedly");

        assert!(matches!(sut.write(43.0), Err(PortError::BufferFull)));
    }
}

#[cfg(test)]
mod input_port_tests {
    use super::*;
    use rtrb::RingBuffer;

    #[test]
    fn set_consumer_stores_consumer_in_port() {
        let (_, consumer): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let mut sut = InputPort::new();

        let _ = sut.set_consumer(consumer);

        assert!(sut.consumer.is_some());
    }

    #[test]
    fn remove_consumer_removes_consumer_in_port() {
        let (_, consumer): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let mut sut = InputPort::new();

        let _ = sut.set_consumer(consumer);
        let _ = sut.remove_consumer();

        assert!(sut.consumer.is_none());
    }

    #[test]
    fn set_producer_returns_error_if_already_set() {
        let (_, consumer1): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let (_, consumer2): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let mut sut = InputPort::new();

        let _ = sut.set_consumer(consumer1);
        let result = sut.set_consumer(consumer2);

        assert!(matches!(result, Err(PortError::AlreadyConnected)));
    }

    #[test]
    fn read_from_unconnected_port_returns_error() {
        let mut sut = InputPort::new();

        let result = sut.read();

        assert!(matches!(result, Err(PortError::NotConnected)));
    }

    #[test]
    fn read_reads_from_buffer() {
        let (mut producer, consumer): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let mut sut = InputPort::new();

        sut.set_consumer(consumer)
            .expect("set consumer failed unexpectedly");

        let _ = producer.push(42.0);
        let result = sut.read().expect("read failed unexpectedly");

        assert_eq!(42.0, result);
    }

    #[test]
    fn read_from_empty_buffer_returns_error() {
        let (_, consumer): (Producer<f32>, Consumer<f32>) = RingBuffer::new(1);
        let mut sut = InputPort::new();

        sut.set_consumer(consumer)
            .expect("set consumer failed unexpectedly");

        assert!(matches!(sut.read(), Err(PortError::BufferEmpty)));
    }
}
