use std::sync::{Arc, Mutex, RwLock};
use bluedroid::gatt_server::{Characteristic, GLOBAL_GATT_SERVER, Profile, Service};
use bluedroid::utilities::{AttributePermissions, BleUuid, CharacteristicProperties};


use serde_json::to_vec;
use crate::auth0::Auth0;


use crate::kv::{NvsKvStore};
use crate::onboarding::{OnboardingHandler, OnboardingState};
use crate::wifi::{EspMyceliumWifi};

const SERVICE_UUID: &str = "00467768-6228-2272-4663-277478269000";
const STATUS_UUID: &str = "00467768-6228-2272-4663-277478269001";
const RPC_COMMAND_UUID: &str = "00467768-6228-2272-4663-277478269002";

pub fn onboarding(handler: OnboardingHandler) {
    // let handler = Arc::new(Mutex::new(OnboardingHandler::new(wifi, kv, auth)));
    // let state = Arc::new(RwLock::new(OnboardingState::AwaitingSettings));
    // let state_read = state.clone();

    let current_state = Characteristic::new(BleUuid::from_uuid128_string(STATUS_UUID))
        .name("Current state")
        .permissions(AttributePermissions::new().read())
        .properties(CharacteristicProperties::new().read().notify())
        .show_name()
        .on_read(move |_| {
            vec![]
        })
        .build();

    let rpc_command = Characteristic::new(BleUuid::from_uuid128_string(RPC_COMMAND_UUID))
        .name("RPC command handler")
        .permissions(AttributePermissions::new().write().read())
        .properties(CharacteristicProperties::new().write().read())
        .on_write(move |bytes, _| { handler.handle(&bytes) }
        )
        .on_read(|_| vec![])
        .show_name()
        .build();

    let service = Service::new(BleUuid::from_uuid128_string(SERVICE_UUID))
        .name("Mycelium onboarding service")
        .primary()
        .characteristic(&rpc_command)
        .characteristic(&current_state)
        .build();

    let profile = Profile::new(0x0001)
        .name("Default Profile")
        .service(&service)
        .build();

    GLOBAL_GATT_SERVER
        .lock()
        .unwrap()
        .profile(profile)
        .device_name("Mycelium onboarding")
        .appearance(bluedroid::utilities::Appearance::GenericComputer)
        .advertise_service(&service)
        .start();
}