use tokio::sync::broadcast;


/// Listens for the server shutdown signal.
#[derive(Debug)]
pub(crate) struct Shutdown {
	shutdown: bool,
	notify: broadcast::Receiver<()>,
}

impl Shutdown {
	pub(crate) fn new(notify: broadcast::Receiver<()>) -> Shutdown {
		Shutdown {
			shutdown: false,
			notify,
		}
	}

	pub(crate) fn is_shutdown(&self) -> bool {
		self.shutdown
	}

	pub(crate) async fn recv(&mut self){
		if self.shutdown {
			return;
		}

		// Cannot receive a "lag error" as only one value is ever sent.
		let _ = self.notify.recv().await;

		// Remember that the signal has been received.
		self.shutdown = true;
	}
}