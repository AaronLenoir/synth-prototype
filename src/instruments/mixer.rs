use std::collections::HashMap;

use rtrb::{Consumer, Producer, RingBuffer};

use crate::{
    core::instrument::{
        instrument::Instrument,
        instrument_error::InstrumentError,
        instrument_info::InstrumentInfo,
        instrument_ports::{InstrumentPorts, PortId, PortResolver},
    },
    sequencer::event::RackEvent,
};

pub struct MixerInPorts;

// 16 tracks with two channels in
impl MixerInPorts {
    pub const IN_LEFT_01: PortId = 0;
    pub const IN_RIGHT_01: PortId = 1;

    pub const IN_LEFT_02: PortId = 2;
    pub const IN_RIGHT_02: PortId = 3;

    pub const IN_LEFT_03: PortId = 4;
    pub const IN_RIGHT_03: PortId = 5;

    pub const IN_LEFT_04: PortId = 6;
    pub const IN_RIGHT_04: PortId = 7;

    pub const IN_LEFT_05: PortId = 8;
    pub const IN_RIGHT_05: PortId = 9;

    pub const IN_LEFT_06: PortId = 10;
    pub const IN_RIGHT_06: PortId = 11;

    pub const IN_LEFT_07: PortId = 12;
    pub const IN_RIGHT_07: PortId = 13;

    pub const IN_LEFT_08: PortId = 14;
    pub const IN_RIGHT_08: PortId = 15;

    pub const IN_LEFT_09: PortId = 16;
    pub const IN_RIGHT_09: PortId = 17;

    pub const IN_LEFT_10: PortId = 18;
    pub const IN_RIGHT_10: PortId = 19;

    pub const IN_LEFT_11: PortId = 20;
    pub const IN_RIGHT_11: PortId = 21;

    pub const IN_LEFT_12: PortId = 22;
    pub const IN_RIGHT_12: PortId = 23;

    pub const IN_LEFT_13: PortId = 24;
    pub const IN_RIGHT_13: PortId = 25;

    pub const IN_LEFT_14: PortId = 26;
    pub const IN_RIGHT_14: PortId = 27;

    pub const IN_LEFT_15: PortId = 28;
    pub const IN_RIGHT_15: PortId = 29;

    pub const IN_LEFT_16: PortId = 30;
    pub const IN_RIGHT_16: PortId = 31;
}

pub struct MixerOutPorts;

impl MixerOutPorts {
    pub const OUT_LEFT: PortId = 0;
    pub const OUT_RIGHT: PortId = 1;
}

impl PortResolver for Mixer {
    fn output_port(&self, name: &str) -> Option<PortId> {
        match name {
            "OUT_LEFT" => Some(MixerOutPorts::OUT_LEFT),
            "OUT_RIGHT" => Some(MixerOutPorts::OUT_RIGHT),
            _ => None,
        }
    }

    fn input_port(&self, name: &str) -> Option<PortId> {
        match name {
            "IN_LEFT_01" => Some(MixerInPorts::IN_LEFT_01),
            "IN_LEFT_02" => Some(MixerInPorts::IN_LEFT_02),
            "IN_LEFT_03" => Some(MixerInPorts::IN_LEFT_03),
            "IN_LEFT_04" => Some(MixerInPorts::IN_LEFT_04),
            "IN_LEFT_05" => Some(MixerInPorts::IN_LEFT_05),
            "IN_LEFT_06" => Some(MixerInPorts::IN_LEFT_06),
            "IN_LEFT_07" => Some(MixerInPorts::IN_LEFT_07),
            "IN_LEFT_08" => Some(MixerInPorts::IN_LEFT_08),
            "IN_LEFT_09" => Some(MixerInPorts::IN_LEFT_09),
            "IN_LEFT_10" => Some(MixerInPorts::IN_LEFT_10),
            "IN_LEFT_11" => Some(MixerInPorts::IN_LEFT_11),
            "IN_LEFT_12" => Some(MixerInPorts::IN_LEFT_12),
            "IN_LEFT_13" => Some(MixerInPorts::IN_LEFT_13),
            "IN_LEFT_14" => Some(MixerInPorts::IN_LEFT_14),
            "IN_LEFT_15" => Some(MixerInPorts::IN_LEFT_15),
            "IN_LEFT_16" => Some(MixerInPorts::IN_LEFT_16),
            "IN_RIGHT_01" => Some(MixerInPorts::IN_RIGHT_01),
            "IN_RIGHT_02" => Some(MixerInPorts::IN_RIGHT_02),
            "IN_RIGHT_03" => Some(MixerInPorts::IN_RIGHT_03),
            "IN_RIGHT_04" => Some(MixerInPorts::IN_RIGHT_04),
            "IN_RIGHT_05" => Some(MixerInPorts::IN_RIGHT_05),
            "IN_RIGHT_06" => Some(MixerInPorts::IN_RIGHT_06),
            "IN_RIGHT_07" => Some(MixerInPorts::IN_RIGHT_07),
            "IN_RIGHT_08" => Some(MixerInPorts::IN_RIGHT_08),
            "IN_RIGHT_09" => Some(MixerInPorts::IN_RIGHT_09),
            "IN_RIGHT_10" => Some(MixerInPorts::IN_RIGHT_10),
            "IN_RIGHT_11" => Some(MixerInPorts::IN_RIGHT_11),
            "IN_RIGHT_12" => Some(MixerInPorts::IN_RIGHT_12),
            "IN_RIGHT_13" => Some(MixerInPorts::IN_RIGHT_13),
            "IN_RIGHT_14" => Some(MixerInPorts::IN_RIGHT_14),
            "IN_RIGHT_15" => Some(MixerInPorts::IN_RIGHT_15),
            "IN_RIGHT_16" => Some(MixerInPorts::IN_RIGHT_16),
            _ => None,
        }
    }
}

pub struct Mixer {
    info: InstrumentInfo,
    ports: InstrumentPorts,
}

impl Mixer {
    pub fn new(name: &str) -> Self {
        Self {
            info: InstrumentInfo::new(name),
            ports: InstrumentPorts::new(32, 32),
        }
    }

    fn add_input(
        &mut self,
        left_in: PortId,
        right_in: PortId,
        sample_count: usize,
        left_buffer: &mut Vec<f32>,
        right_buffer: &mut Vec<f32>,
    ) -> Result<(), InstrumentError> {
        let left_in = self.ports.input_port_mut(left_in);

        for i in 0..(sample_count as usize) {
            left_buffer[i] += left_in.read().unwrap_or(0.0);
        }

        let right_in = self.ports.input_port_mut(right_in);

        for i in 0..(sample_count as usize) {
            right_buffer[i] += right_in.read().unwrap_or(0.0);
        }

        Ok(())
    }
}

impl Instrument for Mixer {
    fn info(&self) -> &InstrumentInfo {
        &self.info
    }

    fn ports(&mut self) -> &mut InstrumentPorts {
        &mut self.ports
    }

    fn update(
        &mut self,
        time_window: u128,
        sample_count: u32,
        events: HashMap<u32, Vec<&RackEvent>>,
    ) -> Result<(), InstrumentError> {
        let sample_count_as_usize = sample_count as usize;

        let mut left_buffer = vec![0.0f32; sample_count_as_usize];
        let mut right_buffer = vec![0.0f32; sample_count_as_usize];

        self.add_input(
            MixerInPorts::IN_LEFT_01,
            MixerInPorts::IN_RIGHT_01,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_02,
            MixerInPorts::IN_RIGHT_02,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_03,
            MixerInPorts::IN_RIGHT_03,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_04,
            MixerInPorts::IN_RIGHT_04,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_05,
            MixerInPorts::IN_RIGHT_05,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_06,
            MixerInPorts::IN_RIGHT_06,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_07,
            MixerInPorts::IN_RIGHT_07,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_08,
            MixerInPorts::IN_RIGHT_08,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_09,
            MixerInPorts::IN_RIGHT_09,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_10,
            MixerInPorts::IN_RIGHT_10,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_11,
            MixerInPorts::IN_RIGHT_11,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_12,
            MixerInPorts::IN_RIGHT_12,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_13,
            MixerInPorts::IN_RIGHT_13,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_14,
            MixerInPorts::IN_RIGHT_14,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_15,
            MixerInPorts::IN_RIGHT_15,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;
        self.add_input(
            MixerInPorts::IN_LEFT_16,
            MixerInPorts::IN_RIGHT_16,
            sample_count_as_usize,
            &mut left_buffer,
            &mut right_buffer,
        )?;

        let name = self.info.name().to_owned();
        let out_left = self.ports.output_port_mut(MixerOutPorts::OUT_LEFT);
        for i in 0..sample_count {
            out_left
                .write_if_connected(left_buffer[i as usize])
                .map_err(|e| InstrumentError::from_port_error(&name, e))?;
        }

        let out_right = self.ports.output_port_mut(MixerOutPorts::OUT_RIGHT);
        for i in 0..sample_count {
            out_right
                .write_if_connected(right_buffer[i as usize])
                .map_err(|e| InstrumentError::from_port_error(&name, e))?;
        }

        Ok(())
    }
}
