use core::slice::IterMut;

use esp_hal::{
    clock::Clocks,
    rmt::{PulseCode, TxChannel},
};

const SK68XX_CODE_PERIOD: u32 = 1200;
const SK68XX_T0H_NS: u32 = 320;
const SK68XX_T0L_NS: u32 = SK68XX_CODE_PERIOD - SK68XX_T0H_NS;
const SK68XX_T1H_NS: u32 = 640;
const SK68XX_T1L_NS: u32 = SK68XX_CODE_PERIOD - SK68XX_T1H_NS;

#[derive(Debug)]
pub enum ARGBError {
    BufferSizeExceeded,
    Transmit(esp_hal::rmt::Error),
}

pub struct ARGB<TX: TxChannel> {
    pulses: (u32, u32),
    channel: Option<TX>,
}

impl<TX: TxChannel> ARGB<TX> {
    pub fn new(channel: TX, clocks: &Clocks) -> Self {
        let src_clock = clocks.apb_clock.to_MHz();

        Self {
            channel: Some(channel),
            pulses: (
                u32::from(PulseCode {
                    level1: true,
                    length1: ((SK68XX_T0H_NS * src_clock) / 1000) as u16,
                    level2: false,
                    length2: ((SK68XX_T0L_NS * src_clock) / 1000) as u16,
                }),
                u32::from(PulseCode {
                    level1: true,
                    length1: ((SK68XX_T1H_NS * src_clock) / 1000) as u16,
                    level2: false,
                    length2: ((SK68XX_T1L_NS * src_clock) / 1000) as u16,
                }),
            ),
        }
    }

    pub fn convert_rgb_channel_to_pulses(
        &self,
        channel_value: u8,
        mut_iter: &mut IterMut<u32>,
    ) -> Result<(), ARGBError> {
        for position in [128, 64, 32, 16, 8, 4, 2, 1] {
            *mut_iter.next().ok_or(ARGBError::BufferSizeExceeded)? = match channel_value & position
            {
                0 => self.pulses.0,
                _ => self.pulses.1,
            }
        }

        Ok(())
    }

    pub fn send(&mut self, buffer: &[u32]) -> Result<(), ARGBError> {
        match self.channel.take().unwrap()
                    .transmit(buffer)
                    .wait() {
            Ok(chan) => {
                self.channel = Some(chan);
            },
            Err((err, chan)) => {
                self.channel = Some(chan);
                return Err(ARGBError::Transmit(err));
            },
        }

        Ok(())
    }
}
