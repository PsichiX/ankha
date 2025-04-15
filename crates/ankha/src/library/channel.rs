use crate::library::option::AnkhaAsyncOption;
use intuicio_core::{
    IntuicioStruct,
    registry::Registry,
    transformer::{DynamicManagedValueTransformer, ValueTransformer},
};
use intuicio_derive::{IntuicioStruct, intuicio_function, intuicio_method, intuicio_methods};
use std::{
    sync::mpsc::{Receiver, Sender, channel as std_channel},
    time::Duration,
};

pub fn install(registry: &mut Registry) {
    registry.add_type(AnkhaReceiver::define_struct(registry));
    registry.add_type(AnkhaSender::define_struct(registry));
    registry.add_function(AnkhaReceiver::receive__define_function(registry));
    registry.add_function(AnkhaReceiver::receive_blocking__define_function(registry));
    registry.add_function(AnkhaReceiver::receive_timeout__define_function(registry));
    registry.add_function(AnkhaReceiver::flush__define_function(registry));
    registry.add_function(AnkhaReceiver::terminate__define_function(registry));
    registry.add_function(AnkhaSender::send__define_function(registry));
    registry.add_function(AnkhaSender::fork__define_function(registry));
    registry.add_function(AnkhaSender::terminate__define_function(registry));
    // registry.add_function(channel::define_function(registry));
}

#[intuicio_function(module_name = "channel")]
pub fn channel(sender: &mut AnkhaSender, receiver: &mut AnkhaReceiver) {
    let (s, r) = std_channel();
    sender.sender = Some(s);
    receiver.receiver = Some(r);
}

#[derive(IntuicioStruct, Default)]
#[intuicio(name = "Receiver", module_name = "channel")]
pub struct AnkhaReceiver {
    #[intuicio(ignore)]
    receiver: Option<Receiver<AnkhaAsyncOption>>,
}

#[intuicio_methods(module_name = "channel")]
impl AnkhaReceiver {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn receive(&self) -> AnkhaAsyncOption {
        self.receiver
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn receive_blocking(&self) -> AnkhaAsyncOption {
        self.receiver
            .as_ref()
            .and_then(|receiver| receiver.recv().ok())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn receive_timeout(&self, timeout: Duration) -> AnkhaAsyncOption {
        self.receiver
            .as_ref()
            .and_then(|receiver| receiver.recv_timeout(timeout).ok())
            .unwrap_or_default()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn flush(&self) {
        if let Some(receiver) = self.receiver.as_ref() {
            while receiver.try_recv().is_ok() {}
        }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn terminate(&mut self) {
        self.receiver = None;
    }
}

#[derive(IntuicioStruct, Default, Clone)]
#[intuicio(name = "Sender", module_name = "channel")]
pub struct AnkhaSender {
    #[intuicio(ignore)]
    sender: Option<Sender<AnkhaAsyncOption>>,
}

#[intuicio_methods(module_name = "channel")]
impl AnkhaSender {
    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn send(&self, value: AnkhaAsyncOption) {
        if let Some(sender) = self.sender.as_ref() {
            let _ = sender.send(value);
        }
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn fork(&self) -> Self {
        self.clone()
    }

    #[intuicio_method(transformer = "DynamicManagedValueTransformer")]
    pub fn terminate(&mut self) {
        self.sender = None;
    }
}
