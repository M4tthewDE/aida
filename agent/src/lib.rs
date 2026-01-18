use ipc_channel::ipc::IpcSender;
use std::{ffi::CStr, os::raw::c_int, sync::OnceLock};
use tracing::debug;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[allow(warnings)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

static SENDER: OnceLock<IpcSender<shared::AgentMessage>> = OnceLock::new();

#[unsafe(export_name = "Agent_OnLoad")]
pub extern "C" fn agent_on_load(
    jvm: *mut bindings::JavaVM,
    options: *mut i8,
    _reserved: *mut std::ffi::c_void,
) -> c_int {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()
        .ok();

    unsafe {
        let server_name = CStr::from_ptr(options).to_str().unwrap();
        let tx: IpcSender<shared::AgentMessage> =
            IpcSender::connect(server_name.to_string()).unwrap();
        SENDER.set(tx.into()).unwrap();

        let get_env = (*(*jvm)).GetEnv.unwrap();
        let mut env: *mut std::ffi::c_void = std::ptr::null_mut();
        let result = get_env(
            jvm,
            &mut env as *mut *mut std::ffi::c_void,
            bindings::JVMTI_VERSION_1_0 as i32,
        );

        let env = if result == bindings::JNI_OK as i32 {
            env as *mut bindings::jvmtiEnv
        } else {
            panic!("error getting env: {}", result);
        };

        let callbacks = bindings::jvmtiEventCallbacks {
            VMInit: Some(vm_init),
            ..Default::default()
        };

        let result = (*(*env)).SetEventCallbacks.unwrap()(
            env,
            &callbacks as *const bindings::jvmtiEventCallbacks,
            size_of::<bindings::jvmtiEventCallbacks>() as i32,
        );

        assert_eq!(result, 0);

        let result = (*(*env)).SetEventNotificationMode.unwrap()(
            env,
            bindings::jvmtiEventMode_JVMTI_ENABLE,
            bindings::jvmtiEvent_JVMTI_EVENT_VM_INIT,
            std::ptr::null_mut(),
        );

        assert_eq!(result, 0);

        let result = (*(*env)).SetEventNotificationMode.unwrap()(
            env,
            bindings::jvmtiEventMode_JVMTI_ENABLE,
            bindings::jvmtiEvent_JVMTI_EVENT_CLASS_LOAD,
            std::ptr::null_mut(),
        );

        assert_eq!(result, 0);
    }

    debug!("agent loaded");

    SENDER
        .get()
        .unwrap()
        .send(shared::AgentMessage::Load)
        .unwrap();

    0
}

#[unsafe(no_mangle)]
pub extern "C" fn vm_init(
    _jvmti_env: *mut bindings::jvmtiEnv,
    _env: *mut bindings::JNIEnv,
    _jthread: bindings::jthread,
) {
    debug!("vm init");
    SENDER
        .get()
        .unwrap()
        .send(shared::AgentMessage::VmInit)
        .unwrap();
}

#[unsafe(export_name = "Agent_OnUnload")]
pub extern "C" fn agent_on_unload(_vm: *mut bindings::JavaVM) {
    debug!("agent unloaded");
    SENDER
        .get()
        .unwrap()
        .send(shared::AgentMessage::Unload)
        .unwrap();
}
