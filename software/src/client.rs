use postcard_rpc::{
    header::VarSeqKind,
    host_client::{HostClient, HostErr},
    standard_icd::{PingEndpoint, WireError, ERROR_PATH},
};
use std::convert::Infallible;
use icd::{
    BadPositionError, GetUniqueIdEndpoint, Rgb8, SetAllLedEndpoint,
    SetSingleLedEndpoint, SingleLed, ToggleLedByPosEndpoint,
};

pub struct WorkbookClient {
    pub client: HostClient<WireError>,
}

#[derive(Debug)]
pub enum WorkbookError<E> {
    Comms(HostErr<WireError>),
    Endpoint(E),
}

impl<E> From<HostErr<WireError>> for WorkbookError<E> {
    fn from(value: HostErr<WireError>) -> Self {
        Self::Comms(value)
    }
}

trait FlattenErr {
    type Good;
    type Bad;
    fn flatten(self) -> Result<Self::Good, WorkbookError<Self::Bad>>;
}

impl<T, E> FlattenErr for Result<T, E> {
    type Good = T;
    type Bad = E;
    fn flatten(self) -> Result<Self::Good, WorkbookError<Self::Bad>> {
        self.map_err(WorkbookError::Endpoint)
    }
}

// ---

impl WorkbookClient {
    pub fn new() -> Self {
        let client = HostClient::new_raw_nusb(
            |d| d.product_string() == Some("flashy"),
            ERROR_PATH,
            8,
            VarSeqKind::Seq2,
        );
        Self { client }
    }

    pub async fn wait_closed(&self) {
        self.client.wait_closed().await;
    }

    pub async fn ping(&self, id: u32) -> Result<u32, WorkbookError<Infallible>> {
        let val = self.client.send_resp::<PingEndpoint>(&id).await?;
        Ok(val)
    }

    pub async fn get_id(&self) -> Result<u64, WorkbookError<Infallible>> {
        let id = self.client.send_resp::<GetUniqueIdEndpoint>(&()).await?;
        Ok(id)
    }

    pub async fn toggle_led_by_pos(
        &self,
        position: u32,
    ) -> Result<(), WorkbookError<BadPositionError>> {
        self.client
            .send_resp::<ToggleLedByPosEndpoint>(&position)
            .await?;
        Ok(())
    }

    pub async fn set_rgb_single(
        &self,
        position: u32,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), WorkbookError<BadPositionError>> {
        self.client
            .send_resp::<SetSingleLedEndpoint>(&SingleLed {
                position,
                rgb: Rgb8 { r, g, b },
            })
            .await?
            .flatten()
    }

    pub async fn set_all_rgb_single(
        &self,
        r: u8,
        g: u8,
        b: u8,
    ) -> Result<(), WorkbookError<Infallible>> {
        self.client
            .send_resp::<SetAllLedEndpoint>(&[Rgb8 { r, g, b }; 24])
            .await?;
        Ok(())
    }

}

impl Default for WorkbookClient {
    fn default() -> Self {
        Self::new()
    }
}