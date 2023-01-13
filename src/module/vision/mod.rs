use std::sync::Arc;

use super::impl_module;
use crate::{
    client::Client,
    proto::v1::{Receiver, V1},
    util::{
        chan::{unbounded, Rx, Tx},
        host2byte,
    },
    Result,
};

pub mod proto;
use proto::cmd::{DetectStatus, DetectTypeMask, SetColor};
pub use proto::{
    cmd::{Color, ColorType},
    sub::DetectInfo,
    DetectType,
};

pub const V1_HOST: Option<Receiver> = Some(host2byte(17, 7));

struct DetectInfoChan {
    rx: Arc<Rx<DetectInfo>>,
    tx: Option<Tx<DetectInfo>>,
}

impl Default for DetectInfoChan {
    fn default() -> Self {
        let (tx, rx) = unbounded();

        DetectInfoChan {
            rx: Arc::new(rx),
            tx: Some(tx),
        }
    }
}

impl_module!(Vision, ~detect_mask: DetectTypeMask, ~detect_info_chan: DetectInfoChan);

impl<C: Client<V1>> Vision<V1, C> {
    pub fn reset(&mut self) -> Result<()> {
        self.detect_mask.reset();
        self.update_detection()
    }

    pub fn enable_detection(&mut self, typ: DetectType, color: Option<Color>) -> Result<()> {
        self.sub_detect_info_event()?;

        // TODO: refresh detect status?
        self.detect_mask.add(typ);
        self.update_detection()?;

        if let Some(color) = color {
            if let Some(color_typ) = match typ {
                DetectType::Line => Some(ColorType::Line),

                DetectType::Marker => Some(ColorType::Marker),

                _ => None,
            } {
                self.set_color(color_typ, color)?;
            }
        }

        Ok(())
    }

    pub fn disable_detection(&mut self, typ: DetectType) -> Result<()> {
        // TODO: refresh detect status?
        self.detect_mask.sub(typ);
        self.update_detection()?;

        Ok(())
    }

    pub fn detect_info_rx(&self) -> &Arc<Rx<DetectInfo>> {
        &self.detect_info_chan.rx
    }

    fn sub_detect_info_event(&mut self) -> Result<()> {
        if let Some(tx) = self.detect_info_chan.tx.take() {
            self.client.subscribe_event(tx)?;
        }

        Ok(())
    }

    fn set_color(&mut self, typ: ColorType, color: Color) -> Result<()> {
        let cmd = SetColor { typ, color };

        self.client.send_cmd_sync(V1_HOST, cmd)?;
        Ok(())
    }

    fn update_detection(&self) -> Result<()> {
        self.client.send_cmd_sync(V1_HOST, self.detect_mask)?;
        Ok(())
    }

    pub fn detection_status(&self) -> Result<DetectTypeMask> {
        self.client.send_cmd_sync(V1_HOST, DetectStatus)
    }
}
