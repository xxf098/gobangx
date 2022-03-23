use crate::event::Key;
use crossterm::event;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

#[derive(Debug, Clone, Copy)]
pub struct EventConfig {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for EventConfig {
    fn default() -> EventConfig {
        EventConfig {
            exit_key: Key::Ctrl('c'),
            tick_rate: Duration::from_millis(250),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Event {
    Input(Key),
    RedrawDatabase(bool),
    RedrawTable(bool),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event>,
    _tx: mpsc::Sender<Event>,
}

impl Events {
    pub fn new(tick_rate: u64) -> Events {
        Events::with_config(EventConfig {
            tick_rate: Duration::from_millis(tick_rate),
            ..Default::default()
        })
    }

    pub fn with_config(config: EventConfig) -> Events {
        let (tx, rx) = mpsc::channel(1024);

        let event_tx = tx.clone();
        tokio::spawn(async move  {
            let tick_rate = config.tick_rate.as_millis() as u64;
            let sleep = time::sleep(Duration::from_millis(tick_rate));
            tokio::pin!(sleep);

            loop {
                tokio::select! {
                    () = &mut sleep => {
                        if event::poll(config.tick_rate).unwrap() {
                            if let event::Event::Key(key) = event::read().unwrap() {
                                let key = Key::from(key);
            
                                event_tx.send(Event::Input(key)).await.unwrap();
                                continue;
                            }
                        }
                        if let Err(_) = event_tx.send(Event::Tick).await {
                            break;
                        }
                        // sleep.as_mut().reset(Instant::now() + Duration::from_millis(tick_rate));
                    },
                }
            }
        });

        Events { rx, _tx: tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn sender(&self) -> mpsc::Sender<Event> {
        self._tx.clone()
    }
}


#[derive(Clone)]
pub struct Store {
    pub sender: mpsc::Sender<Event>,
}


impl Default for Store {
    fn default() -> Self {
        let event = Events::new(200);
        Self::new(event.sender())
    }
}

impl Store {

    pub fn new(sender: mpsc::Sender<Event>) -> Self {
        Self { sender }
    }

    pub async fn dispatch(&self, event: Event) -> anyhow::Result<()> {
        self.sender.send(event).await.map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn dispatch_each(&self, events: Vec<Event>) -> anyhow::Result<()> {
        for event in events.into_iter() {
            if let Err(e) = self.sender.send(event).await.map_err(|e| anyhow::anyhow!(e)) {
                return Err(e)
            }
        }
        Ok(())
    }

    // pub async fn dispatch_all(&self, events: Vec<Event>) -> anyhow::Result<()> {

    // }

}