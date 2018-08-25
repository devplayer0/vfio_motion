use ::reqwest;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Reqwest(err: reqwest::Error) {
            from()
            display("http error: {}", err)
        }
        /*BadState(msg: &'static str) {
            description(msg)
        }*/
    }
}
#[derive(Serialize)]
pub struct Device<'a> {
    #[serde(skip)]
    client: &'a reqwest::Client,
    #[serde(skip)]
    host: &'a str,

    domain: &'a str,
    evdev: &'a str,
    #[serde(skip)]
    attached: bool,
}
impl<'a> Device<'a> {
    pub fn new(client: &'a reqwest::Client, host: &'a str, domain: &'a str, evdev: &'a str) -> Device<'a> {
        Device {
            client,
            host,

            domain,
            evdev,
            attached: false,
        }
    }
    pub fn domain(&self) -> &str {
        self.domain
    }
    pub fn evdev(&self) -> &str {
        self.evdev
    }
    pub fn attached(&self) -> bool {
        self.attached
    }

    pub fn attach(&mut self) -> Result<(), Error> {
        if self.attached {
            warn!("device at '{}' is already attached", self.evdev);
        }

        self.client
            .post(self.host)
            .json(self)
            .send()?;

        self.attached = true;
        Ok(())
    }
    pub fn detach(&mut self) -> Result<(), Error> {
        if !self.attached {
            warn!("device at '{}' is already detached", self.evdev);
        }

        self.client
            .delete(self.host)
            .json(self)
            .send()?;

        self.attached = false;
        Ok(())
    }
    pub fn toggle(&mut self) -> Result<(), Error> {
        match self.attached {
            false => self.attach(),
            true => self.detach()
        }
    }
}
