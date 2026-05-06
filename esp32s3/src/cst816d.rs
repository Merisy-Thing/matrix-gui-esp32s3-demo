use embedded_hal::i2c::{I2c, SevenBitAddress};

const DEFAULT_CST816D_ADDRESS: u8 = 0x15;

const CST816D_ID_REG: u8 = 0xA7;
const CST816D_TOUCH_NUM_REG: u8 = 0x02;
const CST816D_TOUCH_XH_REG: u8 = 0x03;
// const CST816D_TOUCH_XL_REG: u8 = 0x04;
// const CST816D_TOUCH_YH_REG: u8 = 0x05;
// const CST816D_TOUCH_YL_REG: u8 = 0x06;

const TOUCH_REGISTER_IRQ_CTL: u8 = 0xFA; //interrupt control
//const TOUCH_IRQ_EN_IRQ_TEST: u8 = 0x80; //Interrupt pin test, automatically send out low pulse periodically after enabled
const TOUCH_IRQ_EN_TOUCH: u8 = 0x40; //Periodically pulses low when a touch is detected	//gives a lot of events, in itself only provide touch (TOUCH_CONTACT) info. also include gesture info
//const TOUCH_IRQ_EN_CHANGE: u8 = 0x20; //When a change in touch status is detected, a low pulse is emitted	//gives or adds the release (TOUCH_UP) info
//const TOUCH_IRQ_EN_MOTION: u8 = 0x10; //When a gesture is detected, pulse low	//seems to add the GESTURE_TOUCH_BUTTON events, add long press-while-still-touched gestures
//const TOUCH_IRQ_EN_LONGPRESS: u8 = 0x01; //The long press gesture only emits a low pulse signal	//seems to do nothing..?	//note: document inconsist

/// Represents the dimensions of the device
#[derive(Copy, Clone, Debug)]
pub struct Dimension {
    pub _height: u16,
    pub _width: u16,
}

/// Current state of the driver
#[derive(Copy, Clone, Debug)]
pub enum TouchState {
    Pressed(TouchPoint),
    Released(TouchPoint),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
}

impl TouchPoint {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn rotate(self, rotate: bool) -> Self {
        if rotate {
            TouchPoint {
                x: super::HOR_RES - self.x,
                y: super::VER_RES - self.y,
            }
        } else {
            self
        }
    }
}

pub struct CST816D<I2C>
where
    I2C: I2c<SevenBitAddress>,
{
    address: u8,
    i2c: I2C,
    size: Dimension,
    last_tp: TouchPoint,
    rotate: bool,
}

impl<I2C> CST816D<I2C>
where
    I2C: I2c<SevenBitAddress>,
{
    pub fn new(i2c: I2C) -> Self {
        Self {
            address: DEFAULT_CST816D_ADDRESS,
            i2c,
            size: Dimension {
                _height: 240,
                _width: 280,
            },
            last_tp: TouchPoint::new(0, 0),
            rotate: false,
        }
    }

    pub fn rotate(&mut self, rotate: bool) {
        self.rotate = rotate;
    }

    pub fn init(&mut self) -> Result<(), I2C::Error> {
        let mut rx_buf: [u8; 1] = [0xFF];

        self.i2c
            .write_read(self.address, &[CST816D_ID_REG], &mut rx_buf)?;
        if rx_buf[0] != 0xB6 {
            log::warn!("CST816D not found! 0x{:02X}", rx_buf[0]);
        } else {
            log::info!("CST816D found! 0x{:02X}", rx_buf[0]);

            let irq_en = [TOUCH_REGISTER_IRQ_CTL, TOUCH_IRQ_EN_TOUCH];
            if let Err(e) = self.i2c.write(self.address, &irq_en) {
                log::warn!("Failed to enable touch IRQ: {:?}", e);
            } else {
                log::info!("Touch IRQ enabled");
            }
        }

        Ok(())
    }

    pub fn set_size(&mut self, _width: u16, _height: u16) {
        self.size = Dimension { _height, _width };
    }

    pub fn read_touch(&mut self) -> Result<TouchState, I2C::Error> {
        let mut touch_num: [u8; 1] = [0x0];

        self.i2c
            .write_read(self.address, &[CST816D_TOUCH_NUM_REG], &mut touch_num)?;
        if touch_num[0] == 0 {
            return Ok(TouchState::Released(self.last_tp.rotate(self.rotate)));
        }

        let mut rx_buf: [u8; 4] = [0; 4];

        self.i2c
            .write_read(self.address, &[CST816D_TOUCH_XH_REG], &mut rx_buf)?;

        let mut x: u16 = rx_buf[1] as u16 + (((rx_buf[0] as u16) & 0x0F) << 8);
        let mut y: u16 = rx_buf[3] as u16 + (((rx_buf[2] as u16) & 0x0F) << 8);

        let temp = x;
        x = self.size._width - y;
        y = temp;

        //log::info!("========== x = {:?}    y = {:?} ==========", x, y);

        let tp = TouchPoint::new(x, y);
        self.last_tp = tp;

        Ok(TouchState::Pressed(tp.rotate(self.rotate)))
    }
}
