use super::impl_module;
use crate::{
    client::{Client, RawHandler},
    proto::{
        v1::{Receiver, V1},
        Deserialize, ProtoPush, Raw,
    },
    util::{chan::Tx, host2byte},
    Result,
};

pub const V1_HOST: Option<Receiver> = Some(host2byte(3, 0));
const HANDLER_NAME: &str = "v1::Uart";

pub mod proto;
use proto::cmd::MsgSend;
pub use proto::{
    cmd::{BaudRate, DataBit, OddEven, SetParam, StopBit},
    sub::SerialData,
};

impl_module!(Uart);

impl<C: Client<V1>> Uart<V1, C> {
    pub fn set_param(
        &mut self,
        baud_rate: Option<BaudRate>,
        data_bit: Option<DataBit>,
        odd_even: Option<OddEven>,
        stop_bit: Option<StopBit>,
        rx_size: Option<u16>,
        tx_size: Option<u16>,
    ) -> Result<()> {
        // see: https://github.com/dji-sdk/RoboMaster-SDK/blob/8f301fd1bd3038f51c403614c52abbf9e9f5103c/src/robomaster/uart.py#L105-L131
        let cmd = SetParam {
            baud_rate: baud_rate.unwrap_or_default(),
            data_bit: data_bit.unwrap_or_default(),
            odd_even: odd_even.unwrap_or_default(),
            stop_bit: stop_bit.unwrap_or_default(),
            rx_enabled: true,
            tx_enabled: true,
            rx_size: rx_size.unwrap_or(50),
            tx_size: tx_size.unwrap_or(50),
        };

        self.client.send_cmd_sync(Some(host2byte(3, 6)), cmd)?;

        Ok(())
    }

    pub fn send_msg(&mut self, data: Vec<u8>) -> Result<()> {
        let cmd = MsgSend::new(data);
        self.client.send_cmd_sync(Some(host2byte(3, 6)), cmd)?;
        Ok(())
    }

    pub fn sub_serial_data(&mut self, tx: Tx<SerialData>) -> Result<()> {
        let raw_hdl = UartHandler(tx);
        self.client.register_raw_handler(HANDLER_NAME, raw_hdl)?;
        Ok(())
    }
}

struct UartHandler(Tx<SerialData>);

impl RawHandler<V1> for UartHandler {
    fn recv(&self, raw: &Raw<V1>) -> Result<bool> {
        if raw.is_ack {
            return Ok(false);
        }

        if raw.id != <SerialData as ProtoPush<V1>>::IDENT {
            return Ok(false);
        }

        if self.0.is_closed() {
            return Ok(false);
        }

        let data = <SerialData as Deserialize<V1>>::de(&raw.raw_data)?;
        // TODO: some tricks here to avoid clone?
        let tx = self.0.clone();
        tx.send(data)?;
        Ok(true)
    }

    fn gc(&self) -> Result<()> {
        Ok(())
    }
}
